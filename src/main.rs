use std::{collections::{HashMap, HashSet}, fs, path::PathBuf, sync::{Arc, LazyLock, OnceLock}};
use bytes::Bytes;
use clap::Parser;
use crab_nbt::{Nbt, NbtCompound, NbtTag};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use mca::{RawChunk, RegionReader, RegionWriter};
use tokio::{sync::Mutex, task::JoinSet};
use uuid::Uuid;
use anyhow::Result;

#[derive(Parser)]
#[command(name = "uuid_remap")]
#[command(about = "快速改变存档中的uuid")]
struct Args {
    /// world目录位置
    #[arg(long, value_name = "path")]
    world: String,

    /// 映射文件位置
    #[arg(long, value_name = "path")]
    map: String,

    /// 是否仅探测不写入
    #[arg(long, default_value_t = false)]
    dry: bool, 
}

static ARGS: LazyLock<Args> = LazyLock::new(|| {
    Args::parse()
});

static MAP: OnceLock<HashMap<String, String>> = OnceLock::new();

#[tokio::main]
async fn main() {
    let _ = MAP.get_or_init(init_map);

    let a = get_all_mca(&ARGS.world);

    // println!("{:?}", a);
    let mut tasks = JoinSet::new();
    let mpb = Arc::new(MultiProgress::new());

    let pro = Arc::new(Processer::new(&mpb));

    let pb = Arc::new(mpb.add(ProgressBar::new(a.len() as u64)));
    pb.set_style(ProgressStyle::with_template("[{spinner}] {prefix} {pos}/{len}").unwrap_or(ProgressStyle::default_bar()).tick_chars("-\\|/"));
    pb.set_prefix("映射中");

    for path in a {
        let p = Arc::clone(&pro);
        let pb = Arc::clone(&pb);
        tasks.spawn(async move {
            match p.process_mca(&path).await {
                Ok(_) => {
                    p.success.lock().await.insert(path);
                },
                Err(_) => {
                    p.failed.lock().await.insert(path);
                },
            }

            pb.inc(1);
        });
    }

    while let Some(_) = tasks.join_next().await {}

    pb.finish_and_clear();

    println!("所有文件处理完成");
    println!("成功: {}", pro.success.lock().await.len());
    let failed = pro.failed.lock().await;
    let failed_len = failed.len();

    if failed_len > 0 {
        println!("失败: {}", failed_len);
        for mca_file in failed.iter() {
            println!("- {}", mca_file.to_str().unwrap_or("UNKNOWN"))
        }
        println!("以上是失败的文件")
    }
    if ARGS.dry {
        println!("由于启用了 --dry 参数，并未写入任何文件");
    }
}

fn get_all_mca(dir: &str) -> Vec<PathBuf> {
    let mut files = Vec::new();
    
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(extension) = path.extension() && extension == "mca" {
                files.push(path);
            } else if path.is_dir() && let Some(path) = path.to_str() {
                files.extend(get_all_mca(path));
            }
        }
    }

    files
}

fn init_map() -> HashMap<String, String> {
    match fs::read_to_string(&ARGS.map) {
        Ok(map_file) => match serde_json::from_str(&map_file) {
            Ok(map) => map,
            Err(e) => {
                println!("无法读取 {}: {}", ARGS.map, e);
                std::process::exit(1);
            }
        },
        Err(e) => {
            println!("无法读取 {}: {}", ARGS.map, e);
            std::process::exit(1);
        }
    }
}

fn remap(uuid: String) -> String {
    if let Some(new) = MAP.get_or_init(init_map).get(&uuid) {
        // println!("find uuid: {}", uuid);
        new.clone()
    } else {
        uuid
    }
}

struct Processer {
    success: Mutex<HashSet<PathBuf>>,
    failed: Mutex<HashSet<PathBuf>>,

    pb: Arc<MultiProgress>,
    pb_style: ProgressStyle
}

impl Processer {
    fn new(pb: &Arc<MultiProgress>) -> Self {
        Self {
            success: Mutex::new(HashSet::new()),
            failed: Mutex::new(HashSet::new()),

            pb: Arc::clone(pb),
            pb_style: ProgressStyle::with_template("{prefix} [{wide_bar:.green/red}] {pos}/{len}").unwrap_or(ProgressStyle::default_bar()).progress_chars("--")
        }
    }

    async fn process_mca(&self, path: &PathBuf) -> Result<()> {
        // println!("processing: {:?}", path);
        let mca_file = fs::read(&path)?;

        let region = match RegionReader::new(&mca_file) {
            Ok(region) => region,
            Err(_) => return Ok(())
        };

        let mut new_region = RegionWriter::new();
        let pb = self.pb.add(ProgressBar::new(32 * 32));

        pb.set_style(self.pb_style.clone());
        pb.set_prefix(format!("{}", path.to_str().unwrap_or("UNKNOWN")));

        for z in 0..=31u8 {
            for x in 0..=31u8 {
                if let Ok(chunk) = region.get_chunk(x as usize, z as usize) && let Some(chunk) = chunk {
                    let compression_type = chunk.get_compression_type();

                    let data = process_chunk(chunk)?;

                    new_region.push_chunk_with_compression(&data, (x, z), compression_type)?;
                    pb.inc(1);
                }
            }
        };

        pb.finish_and_clear();

        if !ARGS.dry {
            let mut newf = fs::File::create(&path)?;
            new_region.write(&mut newf)?;
        } else {
            // println!("dry mode on, wont write")
        }

        // println!("done: {:?}", path);
        Ok(())
    }
}

fn process_chunk(chunk: RawChunk) -> Result<Bytes> {
    let c = chunk.decompress()?;

    let nbt = Nbt::read(&mut c.as_slice())?;

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
                                // 生物自身UUID
                                if tag_name == "UUID" && let Ok(uuid_new) = uuid_string_to_i32s(&remap(i32s_to_uuid4(nbt_tag.extract_int_array()?).to_string())) {
                                    Some(uuid_new.to_vec().into())
                                } else {
                                    Some(nbt_tag.clone())
                                }
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
    );

    if let Some(new_nbt) = new_nbt {
        Ok(Nbt::new("".to_string(), new_nbt).write())
    } else {
        Err(anyhow::Error::msg("无法构建NBT"))
    }
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
