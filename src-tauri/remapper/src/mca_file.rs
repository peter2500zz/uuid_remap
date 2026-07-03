use std::{
    collections::HashMap,
    fs::{self, File},
    io::{BufWriter, Cursor, Write},
    path::Path,
};

use aho_corasick::AhoCorasick;
use anyhow::Result;
use chardetng::{EncodingDetector, Iso2022JpDetection, Utf8Detection};
use mca::{RegionReader, RegionWriter};
use quartz_nbt::{NbtCompound, NbtTag, io::Flavor};
use uuid::Uuid;

use crate::utils::{i32s_to_uuid4, uuid_map_variants, uuid4_to_i32s};

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
            // test_uuid_64_to_128
            let old_uuid =
                Uuid::from_u128(((uuid_most as u64 as u128) << 64) | (uuid_least as u64 as u128));

            if let Some(&other_uuid) = uuid_map.get(&old_uuid) {
                new_uuids.insert(most_key, NbtTag::Long((other_uuid.as_u128() >> 64) as i64));
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

/// 遍历整棵 NBT 树并交换其中的 UUID
///
/// 字符串统一交给文本算法（`ac`/`replacements`）做单遍双向替换，
/// 同时覆盖整串与嵌入在长字符串中的 UUID；
/// int 数组和 Most/Least 长整数对则用 `uuid_map` 查表交换
fn process_nbt(
    nbt: &mut NbtCompound,
    uuid_map: &HashMap<Uuid, Uuid>,
    ac: &AhoCorasick,
    replacements: &[String],
) {
    let mut stack = Vec::with_capacity(nbt.len());

    process_compound(nbt, uuid_map);
    for (_, tag) in nbt {
        stack.push(tag);
    }

    while let Some(tag) = stack.pop() {
        match tag {
            NbtTag::String(string) => {
                if ac.is_match(string.as_str()) {
                    *string = ac.replace_all(string, replacements);
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

    let (patterns, replacements) = uuid_map_variants(uuid_map);
    let ac = AhoCorasick::new(&patterns)?;

    // SNBT 解析器过于宽容（jsonc 等非严格 JSON 的文本也能被「成功」解析然后改写损坏），
    // 因此只对 .snbt 扩展名的文件按 SNBT 处理，其余文本交给纯文本替换路径
    if path.extension().unwrap_or_default() == "snbt" {
        let mut detector = EncodingDetector::new(Iso2022JpDetection::Allow);
        detector.feed(&bytes, true);
        let encoding = detector.guess(None, Utf8Detection::Allow);
        let (cow, _, _) = encoding.decode(&bytes);

        if max_bracket_depth(&cow) > MAX_NBT_DEPTH {
            return Err(anyhow::anyhow!("括号嵌套过深，不是有效的 SNBT，已跳过"));
        }

        let mut snbt = quartz_nbt::snbt::parse(&cow)?;
        println!("[SNBT] {}", path.display());
        process_nbt(&mut snbt, uuid_map, &ac, &replacements);

        let new_snbt = snbt.to_string();
        let (encoded, _, _) = encoding.encode(&new_snbt);
        // 如果没区别就不写了
        if encoded == bytes {
            return Err(anyhow::anyhow!("文件内容未发生变化，已跳过"));
        }
        fs::write(path, encoded.as_ref())?;
    } else {
        // 可能是二进制
        let flavor = detect_compress_flavor(&bytes);
        let mut cursor = Cursor::new(bytes);
        let (mut nbt, root_name) = quartz_nbt::io::read_nbt(&mut cursor, flavor)?;
        process_nbt(&mut nbt, uuid_map, &ac, &replacements);

        let mut output = Vec::new();
        quartz_nbt::io::write_nbt(&mut output, Some(&root_name), &nbt, flavor)?;

        // 如果没区别就不写了
        if output == cursor.into_inner() {
            return Err(anyhow::anyhow!("文件内容未发生变化，已跳过"));
        }
        fs::write(path, output)?;
    }

    Ok(())
}

/// 解析 mca 文件，并提取其中的区块与 NBT 数据
pub fn process_mca_file(mca_path: &Path, uuid_map: &HashMap<Uuid, Uuid>) -> Result<()> {
    let mca_file = fs::read(mca_path)?;

    let (patterns, replacements) = uuid_map_variants(uuid_map);
    let ac = AhoCorasick::new(&patterns)?;

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
            process_nbt(&mut nbt, uuid_map, &ac, &replacements);

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
fn test_uuid_64_to_128() {
    let uuid = Uuid::parse_str("f89043ac-4df9-401d-813a-24459916827e").unwrap();
    let most: i64 = -535853948335472611;
    let least: i64 = -9134949012827897218;

    let new_uuid = Uuid::from_u128(((most as u64 as u128) << 64) | (least as u64 as u128));

    assert_eq!(uuid, new_uuid);
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
fn jsonc_not_rewritten_as_snbt() -> Result<()> {
    use crate::utils::create_reverse_map;
    use std::str::FromStr;

    let content = "{\n    // Peter_2500[Online] <-> Peter_2500[Offline]\n    \"9db4226c-1015-40da-8fa5-4335aab896b6\": \"59c66d96-d356-364a-a84e-0511b286a31b\"\n}";
    let path = std::env::temp_dir().join("uuid_remap_test_map.jsonc");
    fs::write(&path, content)?;

    let uuid_map = create_reverse_map(&HashMap::from([(
        Uuid::from_str("9db4226c-1015-40da-8fa5-4335aab896b6")?,
        Uuid::from_str("59c66d96-d356-364a-a84e-0511b286a31b")?,
    )]));

    let result = process_nbt_file(&path, &uuid_map);
    let after = fs::read_to_string(&path)?;
    fs::remove_file(&path)?;

    // 非 .snbt 的文本不应被当作 SNBT 改写，应原样落到纯文本替换路径
    assert!(result.is_err());
    assert_eq!(after, content);
    Ok(())
}

#[test]
fn snbt_exact_uuid_string_swapped_once() -> Result<()> {
    use crate::utils::create_reverse_map;
    use std::str::FromStr;

    let path = std::env::temp_dir().join("uuid_remap_test_swap.snbt");
    fs::write(&path, r#"{owner:"9db4226c-1015-40da-8fa5-4335aab896b6"}"#)?;

    // 双向表：曾经的双重交换 bug 会让整串 UUID 被换回原值
    let uuid_map = create_reverse_map(&HashMap::from([(
        Uuid::from_str("9db4226c-1015-40da-8fa5-4335aab896b6")?,
        Uuid::from_str("59c66d96-d356-364a-a84e-0511b286a31b")?,
    )]));

    let result = process_nbt_file(&path, &uuid_map);
    let after = fs::read_to_string(&path)?;
    fs::remove_file(&path)?;

    result?;
    assert!(after.contains("59c66d96-d356-364a-a84e-0511b286a31b"));
    assert!(!after.contains("9db4226c"));
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
