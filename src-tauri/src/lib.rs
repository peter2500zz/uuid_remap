use std::{collections::HashMap, fs::File, io::BufReader, path::PathBuf};

use serde::{Deserialize, Serialize};
use uuid::Uuid;


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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            check_dir,
            read_cache
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
