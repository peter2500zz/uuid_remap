mod map;
mod process;

use std::{
    collections::HashMap,
    fs::{self, File},
    io::BufReader,
    path::{Path, PathBuf},
    sync::Mutex,
};

use remapper::{map::SymBiMap, world::ProgressEvent};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

use crate::process::process_world;

#[allow(unused)]
#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserCache {
    name: String,
    uuid: Uuid,
    #[serde(rename = "expiresOn")]
    expires_on: String,
}

#[allow(unused)]
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PlayerData {
    avatar: Option<String>,
    name: String,
    mode: String,
}

#[tauri::command]
fn check_world_dir(dir_path: String) -> Result<Option<PathBuf>, String> {
    let path = PathBuf::from(dir_path);
    if path.join("level.dat").exists() {
        Ok(Some(path))
    } else {
        Ok(None)
    }
}

#[tauri::command]
fn check_server_dir(dir_path: String) -> Result<Option<(PathBuf, PathBuf)>, String> {
    let server_path = PathBuf::from(dir_path);
    if let Ok(file) = File::open(server_path.join("server.properties"))
        && let Ok(props) = java_properties::read(file)
        && let Some(level_name) = props.get("level-name")
    {
        let world_path = server_path.join(level_name);

        if world_path.join("level.dat").exists() {
            return Ok(Some((server_path, world_path)));
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

#[tauri::command]
fn check_dir_exist(dir_path: String) -> Result<bool, String> {
    Ok(PathBuf::from(dir_path).exists())
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
async fn export_uuid_map(
    uuid_map: HashMap<Uuid, Uuid>,
    name_map: HashMap<Uuid, PlayerData>,
    path: PathBuf,
) -> Result<(), String> {
    let mut jsonc = "{".to_string();

    for (index, (left_uuid, right_uuid)) in uuid_map.iter().enumerate() {
        if index != 0 {
            jsonc.push_str("\n");
        }

        let left_data = name_map.get(&left_uuid);

        let right_data = name_map.get(&right_uuid);

        if left_data.is_some() || right_data.is_some() {
            jsonc.push_str(&format!(
                "\n    // {} <-> {}",
                left_data
                    .map(|pd| format!("{}[{}]", pd.name, pd.mode))
                    .unwrap_or("<anonymous>".into()),
                right_data
                    .map(|pd| format!("{}[{}]", pd.name, pd.mode))
                    .unwrap_or("<anonymous>".into())
            ));
        }

        jsonc.push_str(&format!(
            "\n    \"{}\": \"{}\"{}",
            left_uuid.to_string(),
            right_uuid.to_string(),
            if index < uuid_map.len() - 1 { "," } else { "" }
        ));
    }

    jsonc.push_str("\n}");

    fs::write(path, jsonc).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn import_uuid_map(path: PathBuf) -> Result<HashMap<Uuid, Uuid>, String> {
    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let uuid_map: HashMap<Uuid, Uuid> =
        serde_jsonc::from_str(&content).map_err(|e| e.to_string())?;
    Ok(uuid_map)
}

#[derive(Debug)]
struct AppState {
    pub uuid_map: Mutex<SymBiMap<Uuid>>,
}

impl AppState {
    fn new() -> Self {
        Self {
            uuid_map: Mutex::new(SymBiMap::new()),
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // quartz_nbt 的解析/序列化是递归实现，rayon 工作线程默认栈只有 2MiB，
    // 深层 NBT 会在工作线程上栈溢出，这里加大栈并命名线程方便定位崩溃
    rayon::ThreadPoolBuilder::new()
        .stack_size(64 * 1024 * 1024)
        .thread_name(|i| format!("rayon-worker-{i}"))
        .build_global()
        .expect("初始化 rayon 线程池失败");

    tauri::Builder::default()
        .manage(AppState::new())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            check_world_dir,
            check_server_dir,
            read_cache,
            read_player_data,
            process_world,
            check_dir_exist,
            export_uuid_map,
            import_uuid_map
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
