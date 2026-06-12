use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use anyhow::Result;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use uuid::Uuid;
use walkdir::{DirEntry, WalkDir};

use crate::{
    content_replace::swap_uuids_in_file,
    mca_file::{process_mca_file, process_nbt_file},
    rename_file::exchange_file,
    utils::{create_reverse_map, ensure_no_chain_or_cycle, uuid_swap_variants},
};

/// 处理过程中上报给调用方的进度事件
///
/// 库本身不关心进度如何展示，GUI 可以转发为窗口事件，CLI 可以打印或忽略
#[derive(Debug, Clone)]
pub enum ProgressEvent {
    /// 扫描完成，总任务数已确定
    SetTotal(usize),
    /// 进入新阶段：0 = 并行替换文件内容，1 = 串行重命名文件
    StartPhase(usize),
    /// 开始处理一个文件（相对于处理根目录的路径）
    StartTask(PathBuf),
    /// 一个文件处理完毕
    FinishTask(PathBuf),
}

/// 如果世界目录在服务器目录下，则提升为服务器目录以获得更完全的处理
pub fn resolve_target_path(world_path: &Path) -> PathBuf {
    match world_path.parent() {
        Some(parent) if parent.join("server.properties").exists() => parent.to_path_buf(),
        _ => world_path.to_path_buf(),
    }
}

/// 对世界/服务器目录执行完整的 UUID 互换：
/// 先并行替换所有文件内容，再串行重命名文件
pub fn process_world(
    world_path: &Path,
    uuid_map: &HashMap<Uuid, Uuid>,
    on_progress: impl Fn(ProgressEvent) + Sync,
) -> Result<()> {
    let target_path = resolve_target_path(world_path);

    ensure_no_chain_or_cycle(uuid_map)?;
    // 反转表格是给 nbt remap 用的
    let reverse_map = create_reverse_map(uuid_map);

    let start_time = std::time::Instant::now();

    // 收集所有文件
    let mut all_entries: Vec<DirEntry> = WalkDir::new(&target_path)
        .max_depth(255)
        .into_iter()
        .flatten()
        .collect();

    // 第一阶段只处理文件，第二阶段处理所有条目
    let total = all_entries.iter().filter(|e| e.file_type().is_file()).count() + all_entries.len();
    println!("共发现 {} 个条目，开始处理...", total);

    let relative = |path: &Path| -> PathBuf {
        path.strip_prefix(&target_path).unwrap_or(path).to_path_buf()
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
                on_progress(ProgressEvent::FinishTask(relative_path));
                return;
            }

            on_progress(ProgressEvent::StartTask(relative_path.clone()));

            let file_name = entry.file_name().to_string_lossy();

            match process_mca_file(path, &reverse_map) {
                Ok(()) => println!("[MCA] 成功: {}", file_name),
                Err(mca_err) => match process_nbt_file(path, &reverse_map) {
                    Ok(()) => println!("[NBT] 成功: {}", file_name),
                    Err(nbt_err) => match swap_uuids_in_file(path, uuid_map) {
                        Ok(()) => println!("[NOR] 成功: {}", file_name),
                        // 跳过时把各级尝试的失败原因一并打到终端，便于排查
                        Err(swap_err) => println!(
                            "[SKP] 跳过: {}（mca: {mca_err} | nbt: {nbt_err} | 文本: {swap_err}）",
                            file_name
                        ),
                    },
                },
            }

            on_progress(ProgressEvent::FinishTask(relative_path));
        });

    println!("NBT 替换耗时: {:.2?}", start_time.elapsed());

    on_progress(ProgressEvent::StartPhase(1));

    let (patterns, replacements) = uuid_swap_variants(uuid_map);

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
            on_progress(ProgressEvent::FinishTask(relative_path));
            continue;
        }

        on_progress(ProgressEvent::StartTask(relative_path.clone()));

        let (src, dst) = exchange_file(path, &patterns, &replacements)?;

        if let Some(s) = src {
            visited.insert(s);
        }
        if let Some(d) = dst {
            visited.insert(d);
        }

        on_progress(ProgressEvent::FinishTask(relative_path));
    }

    println!("总耗时: {:.2?}", start_time.elapsed());

    Ok(())
}
