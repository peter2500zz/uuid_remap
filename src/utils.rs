use std::{collections::HashMap, fs};

use crab_nbt::{NbtCompound, NbtTag};
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


/// 对组合型Nbt数据的每一对键值应用handler</br>
/// 返回处理后的数据
pub fn process_compound<F>(compound: &NbtCompound, handler: F) -> Option<NbtCompound>
where
    F: Fn(&String, &NbtTag) -> Option<NbtTag>,
{
    let mut new_compound = NbtCompound::new();

    for (tag_name, nbt_tag) in &compound.child_tags {
        new_compound.put(tag_name.clone(), handler(tag_name, nbt_tag)?);
    }

    Some(new_compound)
}

/// 对列表型Nbt数据的每个值应用handler</br>
/// 返回处理后的数据
pub fn process_list<F>(list: &Vec<NbtTag>, handler: F) -> Option<Vec<NbtTag>>
where
    F: Fn(&NbtTag) -> Option<NbtTag>,
{
    let mut new_list = Vec::new();

    for nbt_tag in list {
        new_list.push(handler(nbt_tag)?);
    }

    Some(new_list)
}


fn to_u128(a: i32, b: i32, c: i32, d: i32) -> u128 {
    let a = a as u32 as u128;
    let b = b as u32 as u128;
    let c = c as u32 as u128;
    let d = d as u32 as u128;

    (a << 96) | (b << 64) | (c << 32) | d
}

fn from_u128(mut value: u128) -> [i32; 4] {
    let mut parts = [0i32; 4];
    for i in (0..4).rev() {
        let part = (value & 0xFFFF_FFFF) as u32;
        parts[i] = part as i32;
        value >>= 32;
    }
    parts
}

pub fn i32s_to_uuid4(values: &[i32]) -> Uuid {
    Uuid::from_u128(to_u128(values[0], values[1], values[2], values[3]))
}

pub fn uuid4_to_i32s(uuid: Uuid) -> [i32; 4] {
    from_u128(uuid.as_u128())
}
