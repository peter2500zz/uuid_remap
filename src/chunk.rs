use std::{collections::HashMap, sync::{Arc, Mutex}};

use bytes::Bytes;
use crab_nbt::{Nbt, NbtCompound};
use mca::RawChunk;
use anyhow::Result;
use uuid::Uuid;

use crate::{i32s_to_uuid4, process_compound, process_list, remap, uuid4_to_i32s};



pub struct ChunkProcesser {
    find: Arc<Mutex<HashMap<Uuid, u32>>>,
}

impl ChunkProcesser {
    pub fn new(find: Arc<Mutex<HashMap<Uuid, u32>>>) -> Self {
        Self { 
            find
        }
    }

    pub fn process(&self, chunk: RawChunk) -> Result<Bytes> {
        let c = chunk.decompress()?;

        let nbt = Nbt::read(&mut c.as_slice())?;

        let find = Arc::clone(&self.find);

        let new_nbt = process_compound(
            &NbtCompound::from(nbt), 
            move |tag_name, nbt_tag| {
                let find = Arc::clone(&find);

                // 获取实体NBT
                let new_tag = if tag_name == "Entities" {
                    process_list(
                        nbt_tag.extract_list()?, 
                        move |nbt_tag| {
                            let find = Arc::clone(&find);

                            // 处理实体NBT
                            let new_tag = process_compound(
                                nbt_tag.extract_compound()?, 
                                move |tag_name, nbt_tag| {
                                    let find = Arc::clone(&find);

                                    // 生物自身UUID
                                    // 单独所需值
                                    match tag_name.as_str() {
                                        "UUID" => {
                                            let old_uuid = i32s_to_uuid4(nbt_tag.extract_int_array()?);
                                            let new_uuid = if let Some(new_uuid) = remap(old_uuid) {
                                                // 有对应
                                                let mut guard = find.lock().expect("无法使用被毒化的Mutex");
                                                *guard.entry(old_uuid).or_insert(0) += 1;
                                                drop(guard);

                                                uuid4_to_i32s(new_uuid).to_vec().into()
                                            } else {
                                                nbt_tag.clone()
                                            };
                                            Some(new_uuid)
                                        },
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
