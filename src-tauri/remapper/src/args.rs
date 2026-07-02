use std::path::PathBuf;

use clap::Parser;

/// 快速改变存档中的uuid
#[derive(Debug, Parser)]
#[command(name = "uuid_remap", version)]
pub struct Args {
    /// world目录位置
    #[arg(long, value_name = "PATH")]
    pub world: PathBuf,

    /// json/jsonc映射文件位置
    #[arg(long, value_name = "PATH")]
    pub map: PathBuf,

    /// 禁用警告
    #[arg(long)]
    pub no_backup_warning: bool,
}
