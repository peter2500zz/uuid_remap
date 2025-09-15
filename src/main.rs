mod chunk;

use std::{collections::{HashMap, HashSet}, fs, path::PathBuf, sync::{Arc, LazyLock, OnceLock}};
use clap::Parser;
use crab_nbt::{NbtCompound, NbtTag};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use mca::{RegionReader, RegionWriter};
use tokio::{sync::Mutex, task::JoinSet};
use uuid::Uuid;
use anyhow::Result;

use chunk::ChunkProcesser;

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

static MAP: OnceLock<HashMap<Uuid, Uuid>> = OnceLock::new();

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
    pb.set_prefix(if ARGS.dry { "检索中" } else { "映射中" });

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
    let a = pb.elapsed();
    pb.finish_and_clear();

    println!("{} took {:.2}s", if ARGS.dry { "已检索所有文件" } else { "所有文件处理完成" }, a.as_secs_f32());
    println!("成功: {}", pro.success.lock().await.len());

    let guard = pro.find.lock().expect("无法使用被毒化的Mutex");
    println!("{}{}个目标UUID", if ARGS.dry { "发现了" } else { "共处理" }, guard.values().sum::<u32>());
    for (uuid, count) in guard.iter() {
        println!("- {}: {}", uuid, count);
    }

    let failed = pro.failed.lock().await;
    let failed_len = failed.len();

    if failed_len > 0 {
        println!("出现异常的文件: {}", failed_len);
        for mca_file in failed.iter() {
            println!("- {}", mca_file.to_str().unwrap_or("UNKNOWN"));
        }
        println!("以上是异常的文件")
    }
    println!();

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

fn init_map() -> HashMap<Uuid, Uuid> {
    match fs::read_to_string(&ARGS.map) {
        Ok(map_file) => match serde_json::from_str::<HashMap<String, String>>(&map_file) {
            Ok(map) => {
                let mut new_map = HashMap::new();
                for (key, value) in &map {
                    let new_key =  match Uuid::parse_str(key) {
                        Ok(key) => key,
                        Err(e) => {
                            println!("错误的UUID {}: {}", key, e);
                            std::process::exit(1);
                        }
                    };

                    let new_value =  match Uuid::parse_str(value) {
                        Ok(value) => value,
                        Err(e) => {
                            println!("错误的UUID {}: {}", value, e);
                            std::process::exit(1);
                        }
                    };

                    new_map.insert(new_key, new_value);
                }

                new_map
            },
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

/// 尝试从UUID映射表中获取对应UUID
fn remap(uuid: Uuid) -> Option<Uuid> {
    MAP.get_or_init(init_map).get(&uuid).cloned()
}

struct Processer {
    success: Mutex<HashSet<PathBuf>>,
    failed: Mutex<HashSet<PathBuf>>,
    find: Arc<std::sync::Mutex<HashMap<Uuid, u32>>>,

    pb: Arc<MultiProgress>,
    pb_style: ProgressStyle
}

impl Processer {
    fn new(pb: &Arc<MultiProgress>) -> Self {
        Self {
            success: Mutex::new(HashSet::new()),
            failed: Mutex::new(HashSet::new()),
            find: Arc::new(std::sync::Mutex::new(HashMap::new())),

            pb: Arc::clone(pb),
            pb_style: ProgressStyle::with_template("{prefix} [{wide_bar:.green/red}] {pos}/{len}").unwrap_or(ProgressStyle::default_bar()).progress_chars("--")
        }
    }

    async fn process_mca(&self, path: &PathBuf) -> Result<()> {
        // 从路径读取mca文件
        let mca_file = fs::read(&path)?;

        // 尝试读取为region
        let region = match RegionReader::new(&mca_file) {
            Ok(region) => region,
            Err(_) => return Ok(())
        };

        let mut new_region = RegionWriter::new();
        let pb = self.pb.add(ProgressBar::new(32 * 32));

        let cp = ChunkProcesser::new(Arc::clone(&self.find));

        // 设置进度条样式
        pb.set_style(self.pb_style.clone());
        pb.set_prefix(format!("{}", path.to_str().unwrap_or("UNKNOWN")));

        for z in 0..=31u8 {
            for x in 0..=31u8 {
                if let Ok(chunk) = region.get_chunk(x as usize, z as usize) && let Some(chunk) = chunk {
                    let compression_type = chunk.get_compression_type();

                    let data = cp.process(chunk)?;

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



fn process_compound<F>(compound: &NbtCompound, handler: F) -> Option<NbtCompound>
where
    F: Fn(&String, &NbtTag) -> Option<NbtTag>,
{
    let mut new_compound = NbtCompound::new();

    for (tag_name, nbt_tag) in &compound.child_tags {
        new_compound.put(tag_name.clone(), handler(tag_name, nbt_tag)?);
    }

    Some(new_compound)
}

fn process_list<F>(list: &Vec<NbtTag>, handler: F) -> Option<Vec<NbtTag>>
where
    F: Fn(&NbtTag) -> Option<NbtTag>,
{
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
