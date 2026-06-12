use std::{
    collections::HashMap,
    fs::{self, File},
    io::{BufWriter, Cursor, Write},
    path::Path,
};

use anyhow::Result;
use chardetng::EncodingDetector;
use mca::{RegionReader, RegionWriter};
use quartz_nbt::{NbtCompound, NbtTag, io::Flavor};
use uuid::Uuid;

use crate::{
    content_replace::swap_uuids_in_string,
    utils::{create_reverse_map, i32s_to_uuid4, uuid4_to_i32s},
};

/// Minecraft 自身的 NBT 嵌套深度上限
const MAX_NBT_DEPTH: usize = 512;

/// 线性扫描文本的最大括号嵌套深度
///
/// quartz_nbt 的 SNBT 解析器是递归实现且没有深度限制，
/// 病态嵌套的文本（例如几十万个连续 `[`）会直接栈溢出，解析前先用它预检
fn max_bracket_depth(text: &str) -> usize {
    let mut depth: usize = 0;
    let mut max_depth = 0;
    for byte in text.bytes() {
        match byte {
            b'[' | b'{' => {
                depth += 1;
                max_depth = max_depth.max(depth);
            }
            b']' | b'}' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    max_depth
}

fn detect_compress_flavor(data: &[u8]) -> Flavor {
    match data {
        [0x1f, 0x8b, ..] => Flavor::GzCompressed,   // gzip
        [0x78, 0x9c, ..] => Flavor::ZlibCompressed, // zlib default
        [0x78, 0x01, ..] => Flavor::ZlibCompressed, // zlib no compression
        [0x78, 0xda, ..] => Flavor::ZlibCompressed, // zlib best compression
        _ => Flavor::Uncompressed,
    }
}

fn process_compound(compound: &mut NbtCompound, uuid_map: &HashMap<Uuid, Uuid>) {
    // 处理 1.16 之前奇怪的 UUID 规则
    let mut new_uuids = HashMap::new();
    let mut uuid_keys_matched = Vec::new();

    // 先找所有 Most 键
    for (most_key, _) in compound
        .into_iter()
        .filter(|&(label, _)| label.ends_with("Most"))
    {
        if let Some(key_prefix) = most_key.strip_suffix("Most") {
            // 拼接出猜测的 Least 键
            let least_key = format!("{}Least", key_prefix);

            uuid_keys_matched.push((most_key.clone(), least_key));
        }
    }

    while let Some((most_key, least_key)) = uuid_keys_matched.pop() {
        // 用这俩去查询，都查到了就是有
        if let Ok(uuid_most) = compound.get::<_, i64>(&most_key)
            && let Ok(uuid_least) = compound.get::<_, i64>(&least_key)
        {
            // 把两个 u64 拼成一个 u128，再转成 UUID
            let old_uuid = Uuid::from_u128((uuid_most as u128) << 64 | (uuid_least as u128));

            if let Some(&other_uuid) = uuid_map.get(&old_uuid) {
                new_uuids.insert(
                    most_key, 
                    NbtTag::Long((other_uuid.as_u128() >> 64) as i64)
                );
                new_uuids.insert(
                    least_key,
                    NbtTag::Long((other_uuid.as_u128() & (u128::MAX >> 64)) as i64),
                );
            }
        }
    }

    for (key, new_tag) in new_uuids {
        compound.insert(key, new_tag);
    }
}

/// 对于每个 NBT，会递归调用此函数以处理可能的所有值
///
/// 触发递归的只有列表和复合标签
fn process_nbt(nbt: &mut NbtCompound, uuid_map: &HashMap<Uuid, Uuid>) {
    let mut stack = Vec::with_capacity(nbt.len());

    process_compound(nbt, uuid_map);
    for (_, tag) in nbt {
        stack.push(tag);
    }

    while let Some(tag) = stack.pop() {
        match tag {
            NbtTag::String(string) => {
                if let Ok(old_uuid) = Uuid::parse_str(string) {
                    if let Some(&other_uuid) = uuid_map.get(&old_uuid) {
                        // println!("Mapping UUID {} to {}", old_uuid, other_uuid);
                        *string = other_uuid.to_string();
                    }
                }
            }
            NbtTag::IntArray(int_array) => {
                if int_array.len() == 4 {
                    let old_uuid = i32s_to_uuid4(int_array);

                    if let Some(&other_uuid) = uuid_map.get(&old_uuid) {
                        // println!("Mapping UUID {} to {}", old_uuid, other_uuid);
                        let new_ints = uuid4_to_i32s(other_uuid);
                        int_array.copy_from_slice(&new_ints);
                    }
                }
            }
            NbtTag::List(list) => {
                for item in list {
                    stack.push(item);
                }
            }
            NbtTag::Compound(compound) => {
                process_compound(compound, uuid_map);
                for (_, tag) in compound {
                    stack.push(tag);
                }
            }
            _ => {}
        }
    }
}

// 需要无环链 uuid map
pub fn process_nbt_file(path: &Path, uuid_map: &HashMap<Uuid, Uuid>) -> Result<()> {
    let bytes = fs::read(path)?;

    let mut detector = EncodingDetector::new();
    detector.feed(&bytes, true);
    let encoding = detector.guess(None, true);
    let (cow, _, had_errors) = encoding.decode(&bytes);
    if had_errors {
        // eprintln!("警告：解码时部分字节无法识别");
    }

    // println!("[TEST] {} {}", serde_json::from_str::<serde_json::Value>(&cow).is_err(), path.display());

    if max_bracket_depth(&cow) <= MAX_NBT_DEPTH
        && serde_json::from_str::<serde_json::Value>(&cow).is_err()
        && let Ok(mut snbt) = quartz_nbt::snbt::parse(&cow)
    {
        println!("[SNBT] {}", path.display());
        // 是 SNBT 格式，处理后再写回去
        process_nbt(&mut snbt, uuid_map);

        // 别忘了替换里面的字符串
        let new_snbt = swap_uuids_in_string(&snbt.to_string(), &create_reverse_map(uuid_map));

        let (encoded, _, _) = encoding.encode(&new_snbt);
        // 如果没区别就不写了
        if encoded == bytes {
            // println!("文件 {:?} 内容未发生变化，已跳过", path);
            return Err(anyhow::anyhow!("文件内容未发生变化，已跳过"));
        }
        fs::write(path, encoded.as_ref())?;
    } else {
        // 可能是二进制
        let flavor = detect_compress_flavor(&bytes);
        let mut cursor = Cursor::new(bytes);
        let (mut nbt, root_name) = quartz_nbt::io::read_nbt(&mut cursor, flavor)?;
        process_nbt(&mut nbt, uuid_map);

        let mut output = Vec::new();
        quartz_nbt::io::write_nbt(&mut output, Some(&root_name), &nbt, flavor)?;

        // 如果没区别就不写了
        if output == cursor.into_inner() {
            // println!("文件 {:?} 内容未发生变化，已跳过", path);
            return Err(anyhow::anyhow!("文件内容未发生变化，已跳过"));
        }
        fs::write(path, output)?;
    }

    Ok(())
}

/// 解析 mca 文件，并提取其中的区块与 NBT 数据
pub fn process_mca_file(mca_path: &Path, uuid_map: &HashMap<Uuid, Uuid>) -> Result<()> {
    let mca_file = fs::read(mca_path)?;

    let mut region = RegionReader::new(&mca_file)?;
    let mut new_region = RegionWriter::new();

    for (x, z) in region.generated_chunks()? {
        // 上面已经只遍历了已生成的区块，这里正常来说不会为 None
        if let Some(chunk) = region.chunk_data(x, z)? {
            // println!("Processing chunk at ({}, {})", x, z);
            let compression_type = chunk.compression.clone();
            let buffer = region.decompress_to_internal_buffer(chunk)?;

            let mut cursor = Cursor::new(buffer);
            let (mut nbt, root_name) = quartz_nbt::io::read_nbt(&mut cursor, Flavor::Uncompressed)?;
            process_nbt(&mut nbt, uuid_map);

            let mut output = Vec::new();
            quartz_nbt::io::write_nbt(&mut output, Some(&root_name), &nbt, Flavor::Uncompressed)?;

            if buffer.len() != output.len() {
                return Err(anyhow::anyhow!("NBT 数据长度发生了变化，无法写回原区块"));
            }

            new_region.set_chunk(x, z, output, compression_type)?;
        } else {
            return Err(anyhow::anyhow!("区块 ({}, {}) 数据缺失，无法处理", x, z));
        }
    }

    let file = File::create(format!("{}", mca_path.display()))?;
    let mut writer = BufWriter::new(file);

    new_region.write(&mut writer)?;
    writer.flush()?;

    Ok(())
}

#[test]
fn bracket_depth() {
    assert_eq!(max_bracket_depth(""), 0);
    assert_eq!(max_bracket_depth("{a:[1,2],b:[3]}"), 2);
    assert_eq!(max_bracket_depth("]]]{["), 2);
    assert_eq!(max_bracket_depth(&"[".repeat(1000)), 1000);
}

#[test]
fn deeply_nested_text_is_skipped() -> Result<()> {
    // 没有深度预检时，SNBT 解析这种文件会直接栈溢出
    let path = std::env::temp_dir().join("uuid_remap_deep_nest_test.snbt");
    fs::write(&path, "[".repeat(1_000_000))?;

    let result = process_nbt_file(&path, &HashMap::new());
    fs::remove_file(&path)?;

    assert!(result.is_err());
    Ok(())
}

#[test]
fn mca() -> Result<()> {
    // 以文件为单位

    use std::str::FromStr;

    let mut uuid_map = HashMap::new();
    uuid_map.insert(
        Uuid::from_str("59c66d96-d356-364a-a84e-0511b286a31b")?,
        Uuid::from_str("9db4226c-1015-40da-8fa5-4335aab896b6")?,
    );

    let start_time = std::time::Instant::now();
    process_mca_file(
        Path::new(r"C:\Users\27978\Downloads\新建文件夹\serverfab\world\entities\r.-1.0.mca"),
        &uuid_map,
    )?;
    let duration = start_time.elapsed();
    println!("Time taken: {:?}", duration);

    Ok(())
}
