use std::{
    // collections::HashMap,
    collections::HashMap,
    fs,
    io::{BufWriter, Write},
    rc::Rc,
};

use anyhow::Result;
use mca::{RegionReader, RegionWriter};
use uuid::Uuid;

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

/// 对于每个 NBT，会递归调用此函数以处理可能的所有值
///
/// 触发递归的只有列表和复合标签
fn process_nbt(nbt: &mut fastnbt::Value, uuid_map: Rc<HashMap<Uuid, Uuid>>) {
    match nbt {
        // fastnbt::Value::Byte(_) => todo!(),
        // fastnbt::Value::Short(_) => todo!(),
        // fastnbt::Value::Int(_) => todo!(),
        // fastnbt::Value::Long(_) => todo!(),
        // fastnbt::Value::Float(_) => todo!(),
        // fastnbt::Value::Double(_) => todo!(),
        // fastnbt::Value::String(v) => {
        //     if v == "minecraft:diamond_hoe" {
        //         *v = "minecraft:diamond_axe".to_string();
        //         println!("{}", v);
        //     } else if v == "minecraft:diamond_axe" {
        //         *v = "minecraft:diamond_hoe".to_string();
        //         println!("{}", v);
        //     }
        // }
        // fastnbt::Value::ByteArray(byte_array) => todo!(),
        fastnbt::Value::IntArray(int_array) => {
            if int_array.len() == 4 {
                let old_uuid = i32s_to_uuid4(int_array);

                if let Some(&other_uuid) = uuid_map.get(&old_uuid) {
                    println!("Mapping UUID {} to {}", old_uuid, other_uuid);
                    let new_ints = uuid4_to_i32s(other_uuid);
                    int_array.copy_from_slice(&new_ints);
                }
            }
        }
        // fastnbt::Value::LongArray(long_array) => todo!(),
        fastnbt::Value::List(values) => {
            for item in values {
                process_nbt(item, Rc::clone(&uuid_map));
            }
        }
        fastnbt::Value::Compound(hash_map) => {
            for v in hash_map.values_mut() {
                process_nbt(v, Rc::clone(&uuid_map));
            }
            // drain 保留了未来修改 key 的灵活性
            // let new_map = hash_map
            //     .drain()
            //     .map(|(k, mut v)| {
            //         process_nbt(&mut v, Rc::clone(&uuid_map));
            //         (k, v)
            //     })
            //     .collect();
            // *hash_map = new_map;
        }
        _ => (),
    }
}

/// 解析 mca 文件，并提取其中的区块与 NBT 数据
pub fn process_mca(mca_path: &str, uuid_map: Rc<HashMap<Uuid, Uuid>>) -> Result<()> {
    let mca_file = fs::read(mca_path)?;

    let mut region = RegionReader::new(&mca_file)?;
    let mut new_region = RegionWriter::new();

    for (x, z) in region.generated_chunks()? {
        // 上面已经只遍历了已生成的区块，这里正常来说不会为 None
        if let Some(chunk) = region.chunk_data(x, z)? {
            // println!("Processing chunk at ({}, {})", x, z);
            let compression_type = chunk.compression.clone();
            let buffer = region.decompress_to_internal_buffer(chunk)?;

            let mut nbt: fastnbt::Value = fastnbt::from_bytes(buffer)?;
            // println!("Original NBT: {:#?}", nbt);
            // let mut input = String::new();
            // std::io::stdin().read_line(&mut input).unwrap();
            process_nbt(&mut nbt, Rc::clone(&uuid_map));

            let new_nbt = fastnbt::to_bytes(&nbt)?;

            assert!(
                buffer.len() == new_nbt.len(),
                "NBT数据长度发生了变化，无法写回原区块"
            );

            new_region.set_chunk(x, z, new_nbt, compression_type)?;
        } else {
            panic!("bad chunk? is the mca file damaged?")
        }
    }

    let file = fs::File::create(format!("{}", mca_path))?;
    let mut writer = BufWriter::new(file);

    new_region.write(&mut writer)?;
    writer.flush()?;

    Ok(())
}

#[test]
fn mca() -> Result<()> {
    use std::str::FromStr;

    let mut uuid_map = HashMap::new();
    uuid_map.insert(
        Uuid::from_str("9db4226c-1015-40da-8fa5-4335aab896b6")?,
        Uuid::from_str("59c66d96-d356-364a-a84e-0511b286a31b")?,
    );
    uuid_map.insert(
        Uuid::from_str("59c66d96-d356-364a-a84e-0511b286a31b")?,
        Uuid::from_str("9db4226c-1015-40da-8fa5-4335aab896b6")?,
    );

    let start_time = std::time::Instant::now();
    process_mca(
        r"C:\Users\27978\Downloads\新建文件夹\server\world\entities\r.0.0.mca",
        Rc::new(uuid_map),
    )?;
    let duration = start_time.elapsed();
    println!("Time taken: {:?}", duration);

    Ok(())
}
