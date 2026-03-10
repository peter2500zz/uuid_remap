mod chunk;
mod utils;
mod args;
mod processer;
mod detect;

use std::{process::exit, sync::Arc};
use console::style;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use inquire::Confirm;
use tokio::task::JoinSet;
use utils::*;
use crate::{args::ARGS, processer::FileProcesser};


#[tokio::main]
async fn main() {
    let map = Arc::new(init_map());

    if !ARGS.dry && !ARGS.no_backup_warning {
        println!("{} 请务必在使用本程序之前备份你的游戏存档", style("!").red().bold());
        let ans = Confirm::new("是否继续?")
            .with_default(false)  // 默认值，例如空输入时为 false
            .prompt();

        match ans {
            Ok(true) => (),
            Ok(false) => {
                println!("已取消运行");
                exit(0)
            },
            Err(_) => {
                println!("程序终止");
                exit(0)
            },
        }
    }

    // println!("{:#?}", map);

    let mca_files = get_all_mca(&ARGS.world);

    let mut dat_files = Vec::new();
    let mut uuid_files = Vec::new();
    if !ARGS.ignore_player {
        dat_files.extend(get_player_dat(&ARGS.world));
        uuid_files.extend(get_uuid_file(&ARGS.world, Arc::clone(&map)));
    }

    // println!("{:?}", a);
    let mut tasks = JoinSet::new();
    let bar_mgr = Arc::new(MultiProgress::new());

    let processer = Arc::new(FileProcesser::new(&bar_mgr, map));

    let bar = Arc::new(bar_mgr.add(ProgressBar::new(
        (mca_files.len() + dat_files.len())
        as u64)
    ));

    bar.set_style(ProgressStyle::with_template("[{spinner}] {prefix}: {msg} {pos}/{len}").unwrap_or(ProgressStyle::default_bar()).tick_chars("-\\|/"));
    bar.set_prefix(if ARGS.dry { "检索中" } else { "映射中" });

    bar.set_message("世界文件");
    for path in mca_files {
        let processer = Arc::clone(&processer);
        let bar = Arc::clone(&bar);
        tasks.spawn(async move {
            match processer.process_mca(&path).await {
                Ok(_) => {
                    processer.success.lock().await.insert(path);
                },
                Err(e) => {
                    processer.failed.lock().await.insert(path, e.to_string());
                },
            }

            bar.inc(1);
        });
    }

    while let Some(_) = tasks.join_next().await {}

    if !ARGS.ignore_player {
        bar.set_message("玩家存档");
        for path in dat_files {
            let processer = Arc::clone(&processer);
            let bar = Arc::clone(&bar);
            tasks.spawn(async move {
                match processer.process_dat(&path).await {
                    Ok(_) => {
                        processer.success.lock().await.insert(path);
                    },
                    Err(e) => {
                        processer.failed.lock().await.insert(path, e.to_string());
                    },
                }

                bar.inc(1);
            });
        }
        
        while let Some(_) = tasks.join_next().await {}

        bar.set_message("UUID文件");
        for mut path in uuid_files {
            let processer = Arc::clone(&processer);
            let bar = Arc::clone(&bar);
            tasks.spawn(async move {
                match processer.uuid_file_transfer(&mut path).await {
                    Ok(_) => {
                        processer.success.lock().await.insert(path);
                    },
                    Err(e) => {
                        processer.failed.lock().await.insert(path, e.to_string());
                    },
                }

                bar.inc(1);
            });
        }
        
        while let Some(_) = tasks.join_next().await {}
    }


    let a = bar.elapsed();
    bar.finish_and_clear();


    println!("{}, took {:.2}s", if ARGS.dry { "已检索所有文件" } else { "所有文件处理完成" }, a.as_secs_f32());
    println!("成功: {}", processer.success.lock().await.len());

    let guard = processer.find.lock().expect("无法使用被毒化的Mutex");
    println!("{}UUID", if guard.is_empty() { format!("{} {}", style("?").dim(),  if ARGS.dry { "未发现目标" } else { "未找到任何可以处理的" }) } else { format!("{} {}", style("↓").green(), if ARGS.dry { "发现的" } else { "处理的" }) });
    for (uuid, count) in guard.iter() {
        println!("{} {}: {}", style("-").dim(), uuid, count);
    }

    let failed = processer.failed.lock().await;
    let failed_len = failed.len();

    if failed_len > 0 {
        println!("{} 出现{}个异常的文件", style("!").red().bold(), failed_len);
        for (file_path, err) in failed.iter() {
            println!("{} {} {}{}{}", style("-").dim(), file_path.to_str().unwrap_or("UNKNOWN"), style("[").dim(), style(err).on_red(), style("]").dim());
        }
        println!("以上是异常的文件")
    }

    if ARGS.dry {
        println!();
        println!("由于启用了 --dry 参数，并未写入任何文件");
    }
}
