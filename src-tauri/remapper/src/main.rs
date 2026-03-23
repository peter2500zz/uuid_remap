mod utils;

use rayon::iter::{ParallelBridge, ParallelIterator};
use remapper::{
    content_replace::swap_uuids_in_file,
    mca_file::{process_mca_file, process_nbt_file},
    rename_file::iter_folder_and_replace,
};
use std::collections::HashMap;
use uuid::Uuid;
use walkdir::WalkDir;

use crate::utils::{assert_no_chain_or_cycle, create_reverse_map};

fn main() {
    let uuid_map = HashMap::from([
        (
            Uuid::parse_str("8e289159-2034-3a16-96b9-9fa637848b3b").unwrap(),
            Uuid::parse_str("fbcd7556-7c16-3a58-911f-bfebf971c7da").unwrap(),
        ),
        // (
        //     Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap(),
        //     Uuid::parse_str("ffffffff-ffff-ffff-ffff-ffffffffffff").unwrap(),
        // ),
    ]);
    assert_no_chain_or_cycle(&uuid_map);
    let reverse_map = create_reverse_map(&uuid_map);

    let start_time = std::time::Instant::now();

    // 遍历文件
    let _: Vec<_> = WalkDir::new(r"C:\Users\27978\Downloads\新建文件夹\serverold\")
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

    if let Err(e) =
        iter_folder_and_replace(&uuid_map, r"C:\Users\27978\Downloads\新建文件夹\serverold\")
    {
        eprintln!("处理文件夹时出错: {}", e);
    }

    let duration = start_time.elapsed();
    println!("总耗时: {:.2?}", duration);
}
