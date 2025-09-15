pub mod nbt_process;
pub mod uuid_tools;

use std::{collections::HashMap, fs};

use uuid::Uuid;

use crate::ARGS;

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

