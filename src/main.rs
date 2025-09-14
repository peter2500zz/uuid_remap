use std::fs;
use crab_nbt::{Nbt, NbtCompound, NbtTag};
use mca::{RegionReader, RegionWriter};
use uuid::Uuid;

const PATH: &str = "/server/world/entities/r.0.-1.mca";

fn main() {
    let f = fs::read(format!(".{PATH}")).unwrap();

    let region = RegionReader::new(&f).unwrap();
    let mut new_region = RegionWriter::new();

    for z in 0..=31u8 {
        for x in 0..=31u8 {
            if let Ok(chunk) = region.get_chunk(x as usize, z as usize) && let Some(chunk) = chunk {
                let c = chunk.decompress().unwrap();

                let nbt = Nbt::read(&mut c.as_slice()).unwrap();

                println!("{:?}", nbt);

                let mut root = NbtCompound::new();

                for (key, value) in &nbt.child_tags {
                    if key == "Entities" {
                        let mut new_entites = Vec::new();
                        for i in value.extract_list().unwrap() {
                            let mut tag = NbtCompound::new();
                            for (key, value) in &i.extract_compound().unwrap().child_tags {
                                if key == "UUID" {
                                    let tv = value.extract_int_array().unwrap();
                                    let uuid = i32s_to_uuid4(&tv[0..4]);

                                    let v = if uuid.to_string() == "58d99299-2da3-4b95-ab02-9f1056c4fc10" {
                                        uuid_string_to_i32s("68d99299-2da3-4b95-ab02-9f1056c4fc11").unwrap().to_vec()
                                    } else {
                                        tv.clone()
                                    };

                                    tag.put(key.clone(), v);
                                } else {
                                    tag.put(key.clone(), value.clone());
                                }
                                
                            }
                            new_entites.push(NbtTag::from(tag));
                        }
                        root.put(key.clone(), new_entites);
                    } else {
                        root.put(key.clone(), value.clone());
                    }
                }

                let new = Nbt::new("".to_string(), root).write();
                println!("{:?}", new);
                new_region.push_chunk_with_compression(&new, (x, z), chunk.get_compression_type()).unwrap();
            }
        }
    }

    let mut newf = fs::File::create(format!(".{PATH}")).unwrap();
    new_region.write(&mut newf).unwrap();

    // println!("{:#?}", );

}

fn i32s_to_uuid4(values: &[i32]) -> Uuid {
    // 将4个i32转换为16字节数组（大端序）
    let mut bytes = [0u8; 16];
    
    bytes[0..4].copy_from_slice(&values[0].to_be_bytes());
    bytes[4..8].copy_from_slice(&values[1].to_be_bytes());
    bytes[8..12].copy_from_slice(&values[2].to_be_bytes());
    bytes[12..16].copy_from_slice(&values[3].to_be_bytes());
    
    // 设置版本位（版本4）和变体位
    bytes[6] = (bytes[6] & 0x0F) | 0x40; // 版本4
    bytes[8] = (bytes[8] & 0x3F) | 0x80; // 变体位
    
    Uuid::from_bytes(bytes)
}

fn uuid4_to_i32s(uuid: Uuid) -> [i32; 4] {
    let bytes = uuid.as_bytes();
    
    let high = i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let mid_high = i32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    let mid_low = i32::from_be_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
    let low = i32::from_be_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);
    
    [high, mid_high, mid_low, low]
}

fn uuid_string_to_i32s(uuid_str: &str) -> Result<[i32; 4], uuid::Error> {
    let uuid = Uuid::parse_str(uuid_str)?;
    Ok(uuid4_to_i32s(uuid))
}
