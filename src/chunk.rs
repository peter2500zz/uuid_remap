use std::{collections::HashMap, sync::{Arc, Mutex}};
use bytes::Bytes;
use crab_nbt::{Nbt, NbtCompound, NbtTag};
use mca::RawChunk;
use anyhow::Result;
use uuid::Uuid;

use crate::utils::*;

pub struct ChunkProcesser {
    find: Arc<Mutex<HashMap<Uuid, u32>>>,
    map : Arc<HashMap<Uuid, Uuid>>
}

impl ChunkProcesser {
    pub fn new(find: Arc<Mutex<HashMap<Uuid, u32>>>, map: Arc<HashMap<Uuid, Uuid>>) -> Self {
        Self { 
            find,
            map
        }
    }

    pub fn process(&self, chunk: RawChunk) -> Result<Bytes> {
        let c = chunk.decompress()?;

        let nbt = Nbt::read(&mut c.as_slice())?;

        let cp = Arc::new(Mutex::new(self));

        
        fn _remap(nbt_tag: &NbtTag, cp: Arc<Mutex<&ChunkProcesser>>) -> Option<NbtTag> {
            let old_uuid = i32s_to_uuid4(nbt_tag.extract_int_array()?);
            let cp = cp.lock().expect("无法使用被毒化的Mutex");
            // println!("{}", old_uuid.to_string());
            let new_uuid = if let Some(new_uuid) = cp.map.get(&old_uuid) {
                // 有对应
                let mut guard = cp.find.lock().expect("无法使用被毒化的Mutex");
                *guard.entry(old_uuid).or_insert(0) += 1;
                drop(guard);

                uuid4_to_i32s(new_uuid.clone()).to_vec().into()
            } else {
                nbt_tag.clone()
            };
            Some(new_uuid)
        }

        let new_nbt = process_compound(
            &NbtCompound::from(nbt), 
            move |tag_name, nbt_tag| {
                let cp = Arc::clone(&cp);

                // 获取实体NBT
                let new_tag = if tag_name == "Entities" {
                    process_list(
                        nbt_tag.extract_list()?, 
                        move |nbt_tag| {
                            let cp = Arc::clone(&cp);

                            // 处理实体NBT
                            let new_tag = process_compound(
                                nbt_tag.extract_compound()?, 
                                move |tag_name, nbt_tag| {
                                    let cp = Arc::clone(&cp);

                                    // 生物自身UUID
                                    // 单独所需值
                                    match tag_name.as_str() {
                                        // 对于可驯服生物 | 最后一次被攻击时的玩家 | 弹射物 | 中立生物仇恨对象
                                        "Owner" | "last_hurt_by_player" | "AngryAt" => {
                                            _remap(nbt_tag, cp)
                                        },
                                        // 对于狐狸
                                        "Trusted" => {
                                            let new_tag = process_list(
                                                nbt_tag.extract_list()?, 
                                                move |nbt_tag| {
                                                    let cp = Arc::clone(&cp);

                                                    _remap(nbt_tag, cp)
                                                }
                                            )?;

                                            Some(new_tag.into())
                                        },
                                        // 对于村民的言论
                                        "Gossips" => {
                                            let new_tag = process_list(
                                                nbt_tag.extract_list()?, 
                                                move |nbt_tag| {
                                                    let cp = Arc::clone(&cp);

                                                    let new_tag = process_compound(
                                                        nbt_tag.extract_compound()?, 
                                                        move |tag_name, nbt_tag| {
                                                            let cp = Arc::clone(&cp);

                                                            match tag_name.as_str() {
                                                                // 言论对象
                                                                "Target" => {
                                                                    _remap(nbt_tag, cp)
                                                                },
                                                                _ => Some(nbt_tag.clone()),
                                                            }
                                                        }
                                                    )?;

                                                    Some(new_tag.into())
                                                }
                                            )?;

                                            Some(new_tag.into())
                                        }
                                        _ => Some(nbt_tag.clone()),
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
}
