use std::{collections::HashMap, path::Path};

use remapper::world::process_world;
use uuid::Uuid;

fn main() {
    // quartz_nbt 的解析/序列化是递归实现，rayon 工作线程默认栈只有 2MiB，
    // 深层 NBT 会在工作线程上栈溢出，这里加大栈并命名线程方便定位崩溃
    rayon::ThreadPoolBuilder::new()
        .stack_size(64 * 1024 * 1024)
        .thread_name(|i| format!("rayon-worker-{i}"))
        .build_global()
        .expect("初始化 rayon 线程池失败");

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

    // CLI 模式下进度直接由库内的日志输出体现，忽略进度事件
    if let Err(e) = process_world(
        Path::new(r"C:\Users\27978\Downloads\新建文件夹\serverold\"),
        &uuid_map,
        |_event| {},
    ) {
        eprintln!("处理失败: {e}");
    }
}
