pub mod nbt_process;
pub mod uuid_tools;

use std::{collections::HashMap, fs, path::PathBuf, sync::Arc};
use uuid::Uuid;
use crate::ARGS;


pub fn get_all_mca(dir: &str) -> Vec<PathBuf> {
    let mut files = Vec::new();
    
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(extension) = path.extension() && extension == "mca" {
                if let Some(parent) = path.parent() && let Some(file_name) = parent.file_name() {
                    if (file_name == "region" && ARGS.ignore_region) ||
                    (file_name == "entities" && ARGS.ignore_entities) ||
                    (file_name == "poi" && ARGS.ignore_poi)
                    {
                        continue;
                    }
                }
                files.push(path);
            } else if path.is_dir() && let Some(path) = path.to_str() {
                files.extend(get_all_mca(path));
            }
        }
    }

    files
}

pub fn get_player_dat(dir: &str) -> Vec<PathBuf> {
    let mut files = Vec::new();
    
    if let Ok(entries) = fs::read_dir(format!("{}/playerdata", dir)) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(extension) = path.extension() && (extension == "dat" || extension == "dat_old") {
                files.push(path);
            }
        }
    }

    files
}

pub fn get_uuid_file(dir: &str, map: Arc<HashMap<Uuid, Uuid>>) -> Vec<PathBuf> {
    let mut files = Vec::new();
    
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(file_name) = path.file_stem() && map.keys().any(|&key| *key.to_string() == *file_name) {
                files.push(path);
            } else if path.is_dir() && let Some(path) = path.to_str() {
                files.extend(get_uuid_file(path, Arc::clone(&map)));
            }
        }
    }

    files
}

pub fn init_map() -> HashMap<Uuid, Uuid> {
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

