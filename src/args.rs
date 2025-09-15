use std::sync::LazyLock;
use clap::Parser;

#[derive(Parser)]
#[command(name = "uuid_remap")]
#[command(about = "快速改变存档中的uuid")]
pub struct Args {
    /// world目录位置
    #[arg(long, value_name = "path")]
    pub world: String,

    /// json映射文件位置
    #[arg(long, value_name = "path")]
    pub map: String,

    /// 仅探测不写入
    #[arg(long, default_value_t = false)]
    pub dry: bool, 

    /// 禁用无 --dry 参数时的警告
    #[arg(long, default_value_t = false)]
    pub no_backup_warning: bool, 

    /// 忽略方块
    #[arg(long, default_value_t = false)]
    pub ignore_region: bool, 

    /// 忽略实体
    #[arg(long, default_value_t = false)]
    pub ignore_entities: bool,

    /// 忽略兴趣点
    #[arg(long, default_value_t = false)]
    pub ignore_poi: bool, 

    /// 忽略玩家数据
    #[arg(long, default_value_t = false)]
    pub ignore_player: bool, 
}

pub static ARGS: LazyLock<Args> = LazyLock::new(|| {
    Args::parse()
});
