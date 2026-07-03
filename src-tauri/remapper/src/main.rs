mod args;

use std::{
    fs,
    io::{self, Write},
};

use anyhow::{Context, Result};
use clap::Parser;
use remapper::{map::SymBiMap, world::process_world};
use uuid::Uuid;

use crate::args::Args;

fn main() -> Result<()> {
    let args = Args::parse();

    // quartz_nbt 的解析/序列化是递归实现，rayon 工作线程默认栈只有 2MiB，
    // 深层 NBT 会在工作线程上栈溢出，这里加大栈并命名线程方便定位崩溃
    rayon::ThreadPoolBuilder::new()
        .stack_size(64 * 1024 * 1024)
        .thread_name(|i| format!("rayon-worker-{i}"))
        .build_global()
        .expect("初始化 rayon 线程池失败");

    let map_content = fs::read_to_string(&args.map)
        .with_context(|| format!("读取映射文件 {} 失败", args.map.display()))?;
    let uuid_map: SymBiMap<Uuid> = serde_jsonc::from_str(&map_content)
        .with_context(|| format!("解析映射文件 {} 失败", args.map.display()))?;

    if !args.no_backup_warning {
        eprintln!(
            "警告：即将直接修改 {} 中的文件，请确认已备份存档，且没有游戏/服务器正在使用它。",
            args.world.display()
        );
        eprint!("继续？[y/N] ");
        io::stderr().flush()?;

        let mut answer = String::new();
        io::stdin().read_line(&mut answer)?;
        if !matches!(answer.trim(), "y" | "Y") {
            println!("已取消");
            return Ok(());
        }
    }

    // CLI 模式下进度直接由库内的日志输出体现，忽略进度事件
    process_world(&args.world, &uuid_map, |_event| {})?;

    Ok(())
}
