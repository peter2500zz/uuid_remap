use std::{collections::{HashMap, HashSet}, fs, io::{Read, Write}, path::PathBuf, str::FromStr, sync::Arc};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use mca::{RegionReader, RegionWriter};
use tokio::sync::Mutex;
use uuid::Uuid;
use anyhow::Result;

use crate::{chunk::NbtProcesser, ARGS};



pub struct FileProcesser {
    pub success: Mutex<HashSet<PathBuf>>,       
    pub failed: Mutex<HashMap<PathBuf, String>>,
    pub find: Arc<std::sync::Mutex<HashMap<Uuid, u32>>>,
    map: Arc<HashMap<Uuid, Uuid>>,

    bar_mgr: Arc<MultiProgress>,
    pb_mca_style: ProgressStyle,
    pb_dat_style: ProgressStyle,
}

impl FileProcesser {
    pub fn new(pb: &Arc<MultiProgress>, map: Arc<HashMap<Uuid, Uuid>>) -> Self {
        Self {
            success: Mutex::new(HashSet::new()),
            failed: Mutex::new(HashMap::new()),
            find: Arc::new(std::sync::Mutex::new(HashMap::new())),
            map,

            bar_mgr: Arc::clone(pb),
            pb_mca_style: ProgressStyle::with_template("{prefix} [{wide_bar:.green/red}] {pos}/{len}").unwrap_or(ProgressStyle::default_bar()).progress_chars("--"),
            pb_dat_style: ProgressStyle::with_template("{prefix}").unwrap_or(ProgressStyle::default_bar())
        }
    }

    pub async fn process_mca(&self, path: &PathBuf) -> Result<()> {
        // 从路径读取mca文件
        let mca_file = fs::read(&path)?;

        // 尝试读取为region
        let region = match RegionReader::new(&mca_file) {
            Ok(region) => region,
            Err(_) => return Ok(())
        };

        let mut new_region = RegionWriter::new();
        let bar = self.bar_mgr.add(ProgressBar::new(32 * 32));

        let nbt_processer = NbtProcesser::new(Arc::clone(&self.find), Arc::clone(&self.map));

        // 设置进度条样式
        bar.set_style(self.pb_mca_style.clone());
        bar.set_prefix(format!("{}", path.to_str().unwrap_or("UNKNOWN")));

        for z in 0..=31u8 {
            for x in 0..=31u8 {
                if let Ok(chunk) = region.get_chunk(x as usize, z as usize) && let Some(chunk) = chunk {
                    let compression_type = chunk.get_compression_type();

                    let data = nbt_processer.chunk(chunk)?;

                    new_region.push_chunk_with_compression(&data, (x, z), compression_type)?;
                    bar.inc(1);
                }
            }
        };

        bar.finish_and_clear();

        if !ARGS.dry {
            let mut newf = fs::File::create(&path)?;
            new_region.write(&mut newf)?;
        } else {
            // println!("dry mode on, wont write")
        }

        // println!("done: {:?}", path);
        Ok(())
    }

    pub async fn process_dat(&self, path: &PathBuf) -> Result<()> {
        // 从路径读取dat文件
        let mca_file = fs::File::open(&path)?;

        let bar = self.bar_mgr.add(ProgressBar::new(1));

        let nbt_processer = NbtProcesser::new(Arc::clone(&self.find), Arc::clone(&self.map));

        // 设置进度条样式
        bar.set_style(self.pb_dat_style.clone());
        bar.set_prefix(format!("{}", path.to_str().unwrap_or("UNKNOWN")));

        let mut buffer = Vec::new();
        let mut decoder = flate2::read::GzDecoder::new(mca_file);
        decoder.read_to_end(&mut buffer)?;

        let data = nbt_processer.player_dat(buffer)?;

        bar.finish_and_clear();

        if !ARGS.dry {
            let newf = fs::File::create(&path)?;
            let mut encoder = flate2::write::GzEncoder::new(newf, flate2::Compression::default());
            encoder.write_all(&data)?;
            encoder.finish()?;
        } else {
            // println!("dry mode on, wont write")
        }

        Ok(())
    }

    pub async fn uuid_file_transfer(&self, path: &mut PathBuf) -> Result<()> {
        if let Some(old_uuid) = path.file_stem().and_then(|s| s.to_str()).and_then(|s| Uuid::from_str(s).ok()) {
            let bar = self.bar_mgr.add(ProgressBar::new(1));

            bar.set_style(self.pb_dat_style.clone());
            bar.set_prefix(format!("{}", path.to_str().unwrap_or("UNKNOWN")));

            let new_uuid = self.map.get(&old_uuid);

            // 看看有没有映射
            if let Some(new_uuid) = new_uuid {
                let mut guard = self.find.lock().expect("无法使用被毒化的Mutex");
                *guard.entry(old_uuid).or_insert(0) += 1;
                drop(guard);

                let file_name = if let Some(ext) = path.extension() {
                    format!("{}.{}", new_uuid, ext.to_string_lossy())
                } else {
                    new_uuid.to_string()
                };

                let mut new_path = path.clone();
                new_path.set_file_name(file_name);
                fs::rename(&path, &new_path)?;
            }

            bar.finish_and_clear();
        }

        Ok(())
    }
}
