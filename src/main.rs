use std::{fs, path::PathBuf};
use bytes::Bytes;
use clap::Parser;
use crab_nbt::{Nbt, NbtCompound, NbtTag};
use mca::{RawChunk, RegionReader, RegionWriter};
use tokio::task::JoinSet;
use uuid::Uuid;
use anyhow::Result;

const PATH: &str = "/server/world";

#[derive(Parser)]
#[command(name = "uuid_remap")]
#[command(about = "快速改变存档中出现的uuid")]
struct Args {
    /// world目录
    #[arg(long, value_name = "Path of world")]
    world: String,
    
    /// 是否启用详细输出
    #[arg(short, long)]
    verbose: bool,
}


#[tokio::main]
async fn main() {
    let args = Args::parse();

    let a = get_all_mca(&format!(".{PATH}"));

    // println!("{:?}", a);
    let mut tasks = JoinSet::new();

    for path in a {
        println!("processing: {:?}", path);
        tasks.spawn(process_mca(path));
    }

    while let Some(_) = tasks.join_next().await {}
    
    println!("所有文件处理完成");
}

fn get_all_mca(dir: &str) -> Vec<PathBuf> {
    let mut files = Vec::new();
    
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(extension) = path.extension() && extension == "mca" {
                files.push(path);
            } else if path.is_dir() {
                files.extend(get_all_mca(path.to_str().unwrap()));
            }
        }
    }
    
    files
}

async fn process_mca(path: PathBuf) -> Result<()> {
    let f = fs::read(path).unwrap();

    let region = match RegionReader::new(&f) {
        Ok(region) => region,
        Err(e) => return Ok(())
    };
    let mut new_region = RegionWriter::new();

    for z in 0..=31u8 {
        for x in 0..=31u8 {
            if let Ok(chunk) = region.get_chunk(x as usize, z as usize) && let Some(chunk) = chunk {
                let compression_type = chunk.get_compression_type();

                let data = process_chunk(chunk).unwrap();

                // new_region.push_chunk_with_compression(&data, (x, z), compression_type).unwrap();
            }
        }
    };

    Ok(())

    // let mut newf = fs::File::create(format!(".{PATH}")).unwrap();
    // new_region.write(&mut newf).unwrap();
}

fn process_chunk(chunk: RawChunk) -> Result<Bytes> {
    let c = chunk.decompress()?;

    let nbt = Nbt::read(&mut c.as_slice())?;

    // println!("{:?}", nbt);

    let new_nbt = process_compound(
        &NbtCompound::from(nbt), 
        |tag_name, nbt_tag| {
            // 获取实体NBT
            let new_tag = if tag_name == "Entities" {
                process_list(
                    nbt_tag.extract_list()?, 
                    |nbt_tag| {
                        // 处理实体NBT
                        let new_tag = process_compound(
                            nbt_tag.extract_compound()?, 
                            |tag_name, nbt_tag| {
                                if tag_name == "UUID" {
                                    let uuid = i32s_to_uuid4(nbt_tag.extract_int_array()?);
                                    // println!("find uuid: {}", uuid);
                                }

                                Some(nbt_tag.clone())
                            }
                        )?;

                        Some(new_tag.into())
                    }
                )?.into()
            } else {
                nbt_tag.clone()
            };

            Some(new_tag)
        }
    ).unwrap();

    // println!("{:?}", r);

    Ok(Nbt::new("".to_string(), new_nbt).write())
}

fn process_compound(compound: &NbtCompound, handler: fn(&String, &NbtTag) -> Option<NbtTag>) -> Option<NbtCompound> {
    let mut new_compound = NbtCompound::new();

    for (tag_name, nbt_tag) in &compound.child_tags {
        new_compound.put(tag_name.clone(), handler(tag_name, nbt_tag)?);
    }

    Some(new_compound)
}

fn process_list(list: &Vec<NbtTag>, handler: fn(nbt_tag: &NbtTag) -> Option<NbtTag>) -> Option<Vec<NbtTag>> {
    let mut new_list = Vec::new();

    for nbt_tag in list {
        new_list.push(handler(nbt_tag)?);
    }

    Some(new_list)
}

fn i32s_to_uuid4(values: &[i32]) -> Uuid {
    // 将4个i32转换为16字节数组（大端序）
    let mut bytes = [0u8; 16];
    
    bytes[0..4].copy_from_slice(&values[0].to_be_bytes());
    bytes[4..8].copy_from_slice(&values[1].to_be_bytes());
    bytes[8..12].copy_from_slice(&values[2].to_be_bytes());
    bytes[12..16].copy_from_slice(&values[3].to_be_bytes());
    
    // 设置版本位（版本4）和变体位
    bytes[6] = (bytes[6] & 0x0F) | 0x40; // 版本4
    bytes[8] = (bytes[8] & 0x3F) | 0x80; // 变体位
    
    Uuid::from_bytes(bytes)
}

fn uuid4_to_i32s(uuid: Uuid) -> [i32; 4] {
    let bytes = uuid.as_bytes();
    
    let high = i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let mid_high = i32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    let mid_low = i32::from_be_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
    let low = i32::from_be_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);
    
    [high, mid_high, mid_low, low]
}

fn uuid_string_to_i32s(uuid_str: &str) -> Result<[i32; 4]> {
    let uuid = Uuid::parse_str(uuid_str)?;
    Ok(uuid4_to_i32s(uuid))
}
