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

fn detect_compress_flavor(data: &[u8]) -> Flavor {
    match data {
        [0x1f, 0x8b, ..] => Flavor::GzCompressed,   // gzip
        [0x78, 0x9c, ..] => Flavor::ZlibCompressed, // zlib default
        [0x78, 0x01, ..] => Flavor::ZlibCompressed, // zlib no compression
        [0x78, 0xda, ..] => Flavor::ZlibCompressed, // zlib best compression
        _ => Flavor::Uncompressed,
    }
}

fn process_nbt_tag(nbt: &mut NbtTag, uuid_map: &HashMap<Uuid, Uuid>) {
    match nbt {
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
            for item in list.inner_mut() {
                process_nbt_tag(item, uuid_map);
            }
        }
        NbtTag::Compound(compound) => {
            process_nbt(compound, uuid_map);
            // drain 保留了未来修改 key 的灵活性
            // let new_map = hash_map
            //     .drain()
            //     .map(|(k, mut v)| {
            //         process_nbt(&mut v, Arc::clone(&uuid_map));
            //         (k, v)
            //     })
            //     .collect();
            // *hash_map = new_map;
        }
        _ => {}
    }
}

/// 对于每个 NBT，会递归调用此函数以处理可能的所有值
///
/// 触发递归的只有列表和复合标签
fn process_nbt(nbt: &mut NbtCompound, uuid_map: &HashMap<Uuid, Uuid>) {
    for v in nbt.inner_mut().values_mut() {
        process_nbt_tag(v, uuid_map);
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

    if serde_json::from_str::<serde_json::Value>(&cow).is_err() && let Ok(mut snbt) = quartz_nbt::snbt::parse(&cow)
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
fn mca() -> Result<()> {
    // 以文件为单位

    use std::str::FromStr;

    let mut uuid_map = HashMap::new();
    uuid_map.insert(
        Uuid::from_str("9db4226c-1015-40da-8fa5-4335aab896b6")?,
        Uuid::from_str("59c66d96-d356-364a-a84e-0511b286a31b")?,
    );

    let start_time = std::time::Instant::now();
    process_mca_file(
        Path::new(r"C:\Users\27978\Downloads\新建文件夹\server\usercache.json"),
        &uuid_map,
    )?;
    let duration = start_time.elapsed();
    println!("Time taken: {:?}", duration);

    Ok(())
}
