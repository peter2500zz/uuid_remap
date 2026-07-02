use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use aho_corasick::AhoCorasick;
use anyhow::Result;
use uuid::Uuid;
use walkdir::WalkDir;

use crate::utils::uuid_swap_variants;

pub fn exchange_file(
    path: &Path,
    patterns: &[String],
    replacements: &[String],
) -> Result<(Option<PathBuf>, Option<PathBuf>)> {
    let parent = match path.parent() {
        Some(p) => p,
        None => return Ok((None, None)),
    };

    let file_name = path.file_name().unwrap_or_default().to_string_lossy();

    let ac = AhoCorasick::new(patterns)?;
    let new_name = ac.replace_all(&file_name, replacements);

    if new_name == file_name {
        return Ok((None, None));
    }

    let src = path.to_path_buf();
    let dst = parent.join(new_name);

    if src.exists() && dst.exists() {
        let tmp = parent.join(format!("{}.tmp", file_name));
        println!(
            "[SWP] 交换文件：{} <-> {}",
            src.file_name().unwrap_or_default().to_string_lossy(),
            dst.file_name().unwrap_or_default().to_string_lossy()
        );
        fs::rename(&src, &tmp)?;
        fs::rename(&dst, &src)?;
        fs::rename(&tmp, &dst)?;
    } else if src.exists() {
        println!(
            "[REN] 重命名文件：{} -> {}",
            src.file_name().unwrap_or_default().to_string_lossy(),
            dst.file_name().unwrap_or_default().to_string_lossy()
        );
        fs::rename(&src, &dst)?;
    }

    Ok((Some(src), Some(dst)))
}

pub fn iter_folder_and_replace(uuid_pairs: &HashMap<Uuid, Uuid>, folder_path: &str) -> Result<()> {
    // 预存一些不同的 UUID 变体
    let (patterns, replacements) = uuid_swap_variants(uuid_pairs);

    // 收集一下文件，否则运行时 walk 会重复
    let mut entries: Vec<PathBuf> = WalkDir::new(folder_path)
        .max_depth(255)
        .into_iter()
        .flatten()
        .map(|e| e.path().to_path_buf())
        .collect();

    entries.sort_by(|left, right| {
        let depth_left = left.components().count();
        let depth_right = right.components().count();

        // 深度深的靠前
        depth_right.cmp(&depth_left).then_with(|| {
            // 同深度：文件排在文件夹前面
            let is_file_left = left.is_file() as u8;
            let is_file_right = right.is_file() as u8;
            is_file_right.cmp(&is_file_left).reverse()
            // is_file 为 true(1) 的靠前，所以用 right.cmp(left)
        })
    });

    // 缓存以跳过已经处理过的文件
    let mut visited: HashSet<PathBuf> = HashSet::new();

    for entry in entries {
        if !entry.exists() || visited.contains(&entry) {
            continue;
        }

        let (src, dst) = exchange_file(&entry, &patterns, &replacements)?;
        if let Some(s) = src {
            visited.insert(s);
        }
        if let Some(d) = dst {
            visited.insert(d);
        }
    }

    Ok(())
}

#[test]
fn exchange() -> Result<()> {
    // 以文件夹为单位

    use std::str::FromStr;

    let uuid1 = Uuid::from_str("9db4226c-1015-40da-8fa5-4335aab896b6")?;
    let uuid2 = Uuid::from_str("59c66d96-d356-364a-a84e-0511b286a31b")?;
    let uuid3 = Uuid::from_str("00000000-0000-0000-0000-000000000000")?;
    let uuid4 = Uuid::from_str("ffffffff-ffff-ffff-ffff-ffffffffffff")?;

    let uuid_pairs = HashMap::from([(uuid1, uuid2), (uuid3, uuid4)]);

    iter_folder_and_replace(&uuid_pairs, r"C:\Users\27978\Downloads\新建文件夹\server\")?;

    Ok(())
}
