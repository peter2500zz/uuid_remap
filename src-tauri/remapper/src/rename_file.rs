use std::{
    fs,
    path::{Path, PathBuf},
};

use aho_corasick::AhoCorasick;
use anyhow::Result;

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
