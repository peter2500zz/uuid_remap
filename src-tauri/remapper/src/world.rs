use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use anyhow::Result;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::Serialize;
use uuid::Uuid;
use walkdir::{DirEntry, WalkDir};

use crate::{
    content_replace::{FileContentSwapResult, swap_uuids_in_file},
    map::SymBiMap,
    mca_file::{process_mca_file, process_nbt_file},
    rename_file::exchange_file,
    utils::uuid_map_variants,
};

/// 处理过程中上报给调用方的进度事件
///
/// 库本身不关心进度如何展示，GUI 可以转发为窗口事件，CLI 可以打印或忽略
#[derive(Debug, Serialize, Clone)]
#[serde(tag = "kind", content = "data")]
pub enum ProgressEvent {
    /// 扫描完成，总任务数已确定
    SetTotal(usize),
    /// 进入新阶段：0 = 并行替换文件内容，1 = 串行重命名文件
    StartPhase(usize),
    /// 开始处理一个文件（相对于处理根目录的路径）
    StartTask(PathBuf),
    /// 一个文件处理完毕
    FinishTask(FinishTaskData),
}

#[derive(Debug, Serialize, Clone)]
pub struct FinishTaskData {
    path: PathBuf,
    result: TaskResult,
}

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "kind", content = "data")]
pub enum TaskResult {
    Success,
    NoChange,
    Unsupported(Vec<FileProcessError>),
    Error(Vec<FileProcessError>),
}

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "kind", content = "data")]
pub enum FileProcessError {
    McaError(String),
    NbtError(String),
    ContentError(String),
    RenameError(String),
}

/// 对世界/服务器目录执行完整的 UUID 互换：
/// 先并行替换所有文件内容，再串行重命名文件
pub fn process_world(
    process_path: &Path,
    uuid_map: &SymBiMap<Uuid>,
    on_progress: impl Fn(ProgressEvent) + Sync,
) -> Result<()> {
    anyhow::ensure!(
        process_path.try_exists()?,
        "path does not exist: {}",
        process_path.display()
    );

    let start_time = std::time::Instant::now();

    // 收集所有文件
    let mut all_entries: Vec<DirEntry> = WalkDir::new(&process_path)
        .max_depth(255)
        .into_iter()
        .flatten()
        .collect();

    // 第一阶段只处理文件，第二阶段处理所有条目
    let total = all_entries
        .iter()
        .filter(|e| e.file_type().is_file())
        .count()
        + all_entries.len();
    println!("共发现 {} 个条目，开始处理...", total);

    let relative = |path: &Path| -> PathBuf {
        path.strip_prefix(&process_path)
            .unwrap_or(path)
            .to_path_buf()
    };

    on_progress(ProgressEvent::SetTotal(total));
    on_progress(ProgressEvent::StartPhase(0));

    // NBT/文件内容 做并行处理
    all_entries
        .par_iter()
        .filter(|e| e.file_type().is_file())
        .for_each(|entry| {
            let path = entry.path();
            let relative_path = relative(path);

            if path.extension().is_some_and(|ext| ext == "jar") {
                println!("[SKP] 跳过: {}", relative_path.display());
                // todo!("not only for jar, but also for zip, mcpack, mctemplate, etc. (all are zip format)");
                return;
            }

            on_progress(ProgressEvent::StartTask(relative_path.clone()));

            let file_name = entry.file_name().to_string_lossy();

            let result = 'foo: {
                let mca_err = match process_mca_file(path, uuid_map) {
                    Ok(()) => {
                        println!("[MCA] 成功: {}", file_name);
                        break 'foo TaskResult::Success;
                    }
                    Err(e) => FileProcessError::McaError(e.to_string()),
                };

                let nbt_err = match process_nbt_file(path, uuid_map) {
                    Ok(()) => {
                        println!("[NBT] 成功: {}", file_name);
                        break 'foo TaskResult::Success;
                    }
                    Err(e) => FileProcessError::NbtError(e.to_string()),
                };

                match swap_uuids_in_file(path, uuid_map) {
                    Ok(result) => match result {
                        FileContentSwapResult::IsBinary => {
                            println!(
                                "[SKP] 跳过: {}（mca: {:?} | nbt: {:?} | content: 不是文本文件）",
                                file_name, mca_err, nbt_err
                            );
                            TaskResult::Unsupported(vec![mca_err, nbt_err])
                        }
                        FileContentSwapResult::NoChange => {
                            println!("[SKP] 跳过: {}", file_name);
                            TaskResult::NoChange
                        }
                        FileContentSwapResult::Changed => {
                            println!("[NOR] 成功: {}", file_name);
                            TaskResult::Success
                        }
                    },
                    Err(e) => {
                        println!("[ERR] 错误: {}（{:?}）", file_name, e);
                        TaskResult::Error(vec![
                            mca_err,
                            nbt_err,
                            FileProcessError::ContentError(e.to_string()),
                        ])
                    }
                }
            };

            on_progress(ProgressEvent::FinishTask(FinishTaskData {
                path: relative_path,
                result,
            }));
        });

    println!("NBT 替换耗时: {:.2?}", start_time.elapsed());

    on_progress(ProgressEvent::StartPhase(1));

    let (patterns, replacements) = uuid_map_variants(uuid_map.iter());

    // 对于重命名需要排序文件，先深后浅，文件优先于目录，避免重命名导致的路径失效
    all_entries.sort_unstable_by(|l, r| {
        r.depth()
            .cmp(&l.depth())
            .then_with(|| r.file_type().is_file().cmp(&l.file_type().is_file()))
    });

    // 为了避免交换过的文件被重复交换，维护一个访问过的路径集合
    let mut visited: HashSet<PathBuf> = HashSet::new();

    for entry in &all_entries {
        let path = entry.path();
        let relative_path = relative(path);

        // 已交换过或已不存在（被重命名走）的路径直接计入进度
        if visited.contains(path) || !path.exists() {
            on_progress(ProgressEvent::FinishTask(FinishTaskData {
                path: relative_path,
                result: TaskResult::NoChange,
            }));
            continue;
        }

        on_progress(ProgressEvent::StartTask(relative_path.clone()));

        let (src, dst) = match exchange_file(path, &patterns, &replacements) {
            Ok((src, dst)) => (src, dst),
            Err(e) => {
                println!(
                    "[ERR] 重命名失败: {}（{e}）",
                    path.file_name().unwrap_or_default().to_string_lossy()
                );
                on_progress(ProgressEvent::FinishTask(FinishTaskData {
                    path: relative_path,
                    result: TaskResult::Error(vec![FileProcessError::RenameError(e.to_string())]),
                }));
                continue;
            }
        };

        if let Some(s) = src {
            visited.insert(s);
        }
        if let Some(d) = dst {
            visited.insert(d);
        }

        on_progress(ProgressEvent::FinishTask(FinishTaskData {
            path: relative_path,
            result: TaskResult::Success,
        }));
    }

    println!("总耗时: {:.2?}", start_time.elapsed());

    Ok(())
}
