use std::{
    collections::HashMap,
    fs::{self, File},
    io::BufReader,
    path::PathBuf,
};

use rayon::iter::{ParallelBridge, ParallelIterator};
use remapper::{
    content_replace::swap_uuids_in_file,
    mca_file::{process_mca_file, process_nbt_file},
    rename_file::iter_folder_and_replace,
    utils::{assert_no_chain_or_cycle, create_reverse_map},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use walkdir::WalkDir;

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
async fn process_world(world_path: String, uuid_map: HashMap<Uuid, Uuid>) -> Result<(), String> {
    let world_pathbuf = PathBuf::from(world_path);

    // 如果上级目录存在 server.properties，则提升目标路径到上级目录
    // 这样可以确保处理服务器的资源内容
    let target_path = if let Some(server_dir_or_not) = world_pathbuf.parent()
        && server_dir_or_not.join("server.properties").exists()
    {
        server_dir_or_not.to_path_buf()
    } else {
        world_pathbuf
    };

    assert_no_chain_or_cycle(&uuid_map);
    let reverse_map = create_reverse_map(&uuid_map);

    let start_time = std::time::Instant::now();

    // 遍历文件
    let _: Vec<_> = WalkDir::new(&target_path)
        .max_depth(255)
        .into_iter()
        .flatten()
        .par_bridge()
        .map(|entry| {
            let path = entry.path();
            if path.exists() && path.is_file() {
                if let Ok(_) = process_mca_file(path, &reverse_map) {
                    println!(
                        "[MCA] 成功: {}",
                        path.file_name().unwrap().to_string_lossy()
                    );
                } else if let Ok(_) = process_nbt_file(path, &reverse_map) {
                    println!(
                        "[NBT] 成功: {}",
                        path.file_name().unwrap().to_string_lossy()
                    );
                } else {
                    match swap_uuids_in_file(path, &uuid_map) {
                        Ok(_) => println!(
                            "[NOR] 成功: {}",
                            path.file_name().unwrap().to_string_lossy()
                        ),
                        Err(_) => {}
                    }
                }
            }
        })
        .collect();

    let duration_nbt = start_time.elapsed();
    println!("NBT 替换耗时: {:.2?}", duration_nbt);

    if let Err(e) = iter_folder_and_replace(&uuid_map, &target_path.to_string_lossy()) {
        eprintln!("处理文件夹时出错: {}", e);
    }

    let duration = start_time.elapsed();
    println!("总耗时: {:.2?}", duration);

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
