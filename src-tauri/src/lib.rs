use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    io::BufReader,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

use rayon::iter::{IntoParallelIterator, ParallelIterator};
use remapper::{
    content_replace::swap_uuids_in_file,
    mca_file::{process_mca_file, process_nbt_file},
    rename_file::exchange_file,
    utils::{assert_no_chain_or_cycle, create_reverse_map, uuid_swap_variants},
};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use uuid::Uuid;
use walkdir::{DirEntry, WalkDir};

#[allow(unused)]
#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserCache {
    name: String,
    uuid: Uuid,
    #[serde(rename = "expiresOn")]
    expires_on: String,
}

#[tauri::command]
fn check_dir(dir_path: String) -> Result<bool, String> {
    Ok(PathBuf::from(dir_path).join("level.dat").exists())
}

#[tauri::command]
fn read_cache(file_path: String) -> Result<Vec<UserCache>, String> {
    let file = File::open(file_path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);

    let user_cache: Vec<UserCache> = serde_json::from_reader(reader).map_err(|e| e.to_string())?;

    Ok(user_cache)
}

#[tauri::command]
fn read_player_data(dir_path: String) -> Result<Vec<Uuid>, String> {
    let mut uuids: Vec<Uuid> = Vec::new();

    let entries = fs::read_dir(dir_path).map_err(|e| e.to_string())?;

    for entry in entries {
        if let Ok(entry) = entry
            && let Some(file_prefix) = entry.path().file_prefix()
            && let Some(uuid) = Uuid::parse_str(&file_prefix.to_string_lossy()).ok()
            && !uuids.contains(&uuid)
        {
            uuids.push(uuid);
        }
    }

    Ok(uuids)
}

#[tauri::command]
async fn process_world(
    app: AppHandle,
    world_path: String,
    uuid_map: HashMap<Uuid, Uuid>,
) -> Result<(), String> {
    let world_pathbuf = PathBuf::from(world_path);

    // 如果有 server.properties 则提升
    let target_path = if let Some(parent) = world_pathbuf.parent()
        && parent.join("server.properties").exists()
    {
        parent.to_path_buf()
    } else {
        world_pathbuf
    };

    // TODO 改为返回错误而不是断言
    assert_no_chain_or_cycle(&uuid_map);
    // 反转表格是给 nbt remap 用的
    let reverse_map = create_reverse_map(&uuid_map);

    let start_time = std::time::Instant::now();

    // 收集所有文件
    let mut all_entries: Vec<DirEntry> = WalkDir::new(&target_path)
        .max_depth(255)
        .into_iter()
        .flatten()
        .collect();

    // 计算总量
    let total = (&all_entries)
        .iter()
        .filter_map(|entry| entry.file_type().is_file().then_some(()))
        .collect::<Vec<_>>()
        .len() + all_entries.len();
    println!("共发现 {} 个条目，开始处理...", total);

    let _ = app.emit("set-total", total);

    // NBT/文件内容 做并行处理
    all_entries
        .clone()
        .into_par_iter()
        .filter(|e| e.file_type().is_file())
        .for_each(|entry| {
            let path = entry.path();

            if let Some(extension) = path.extension()
                && vec!["jar"].contains(&extension.to_string_lossy().as_ref())
            {
                let _ = app.emit("finish-task", path);

                return;
            }

            let _ = app.emit("start-task", path);

            if process_mca_file(path, &reverse_map).is_ok() {
                println!("[MCA] 成功: {}", entry.file_name().to_string_lossy());
            } else if process_nbt_file(path, &reverse_map).is_ok() {
                println!("[NBT] 成功: {}", entry.file_name().to_string_lossy());
            } else if swap_uuids_in_file(path, &uuid_map).is_ok() {
                println!("[NOR] 成功: {}", entry.file_name().to_string_lossy());
            } else {
                println!("[SKP] 跳过: {}", entry.file_name().to_string_lossy());
            }

            let _ = app.emit("finish-task", path);
        });

    println!("NBT 替换耗时: {:.2?}", start_time.elapsed());

    let (patterns, replacements) = uuid_swap_variants(&uuid_map);

    // 对于重命名需要排序文件，先深后浅，文件优先于目录，避免重命名导致的路径失效
    all_entries.sort_unstable_by(|l, r| {
        r.depth()
            .cmp(&l.depth())
            .then_with(|| r.file_type().is_file().cmp(&l.file_type().is_file()))
    });

    // 为了避免交换过的文件被重复交换，维护一个访问过的路径集合
    let mut visited: HashSet<PathBuf> = HashSet::new();

    for entry in &all_entries {
        let path = entry.path();

        // 检查路径是否已经被访问过
        if visited.contains(path) {
            let _ = app.emit("finish-task", path);
            continue;
        }
        // 检查路径是否存在
        if !path.exists() {
            let _ = app.emit("finish-task", path);
            continue;
        }

        let _ = app.emit("start-task", path);

        let (src, dst) =
            exchange_file(path, &patterns, &replacements).map_err(|e| e.to_string())?;

        if let Some(s) = src {
            visited.insert(s);
        }
        if let Some(d) = dst {
            visited.insert(d);
        }

        let _ = app.emit("finish-task", path);

    }

    println!("总耗时: {:.2?}", start_time.elapsed());

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            check_dir,
            read_cache,
            read_player_data,
            process_world
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
