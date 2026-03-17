use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use anyhow::Result;
use uuid::Uuid;
use walkdir::WalkDir;

fn exchange_file(path: &Path, left: &str, right: &str) -> Result<()> {
    let parent = match path.parent() {
        Some(p) => p,
        None => return Ok(()),
    };

    let file_name = path.file_name().unwrap_or_default().to_string_lossy();

    let (src, dst_name) = if file_name.contains(left) {
        (path.to_path_buf(), file_name.replace(left, right))
    } else if file_name.contains(right) {
        (path.to_path_buf(), file_name.replace(right, left))
    } else {
        return Ok(());
    };

    let dst = parent.join(&dst_name);

    if src.exists() && dst.exists() {
        let tmp = parent.join(format!("{}.tmp", file_name));
        println!("交换文件：{} <-> {}", src.display(), dst.display());
        fs::rename(&src, &tmp)?;
        fs::rename(&dst, &src)?;
        fs::rename(&tmp, &dst)?;
    } else if src.exists() {
        println!("重命名文件：{} -> {}", src.display(), dst.display());
        fs::rename(&src, &dst)?;
    }

    Ok(())
}

pub fn iter_folder_and_replace(uuid_pair: (Uuid, Uuid), folder_path: &str) -> Result<()> {
    // 预存一些不同的 UUID 变体
    let variants: Vec<(String, String)> = [
        (uuid_pair.0.to_string(), uuid_pair.1.to_string()),
        (
            uuid_pair.0.to_string().to_uppercase(),
            uuid_pair.1.to_string().to_uppercase(),
        ),
        (
            uuid_pair.0.to_string().replace('-', ""),
            uuid_pair.1.to_string().replace('-', ""),
        ),
        (
            uuid_pair.0.to_string().to_uppercase().replace('-', ""),
            uuid_pair.1.to_string().to_uppercase().replace('-', ""),
        ),
    ]
    .into();

    // 收集一下文件，否则运行时 walk 会重复
    let files: Vec<PathBuf> = WalkDir::new(folder_path)
        .max_depth(255)
        .into_iter()
        .flatten()
        .filter(|e| e.path().is_file())
        .map(|e| e.path().to_path_buf())
        .collect();

    // 缓存以跳过已经处理过的文件
    let mut visited: HashSet<PathBuf> = HashSet::new();

    for file_path in files {
        if !file_path.exists() || visited.contains(&file_path) {
            continue;
        }

        let file_name = file_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        for (a, b) in &variants {
            if file_name.contains(a.as_str()) || file_name.contains(b.as_str()) {
                // 把交换的两个路径都标记为已访问
                let dst_name = if file_name.contains(a.as_str()) {
                    file_name.replace(a.as_str(), b.as_str())
                } else {
                    file_name.replace(b.as_str(), a.as_str())
                };
                let dst_path = file_path.parent().unwrap().join(&dst_name);

                visited.insert(file_path.clone());
                visited.insert(dst_path);

                exchange_file(&file_path, a, b)?;
                break;
            }
        }
    }

    Ok(())
}

#[test]
fn exchange() -> Result<()> {
    use std::str::FromStr;

    let uuid1 = Uuid::from_str("9db4226c-1015-40da-8fa5-4335aab896b6")?;
    let uuid2 = Uuid::from_str("59c66d96-d356-364a-a84e-0511b286a31b")?;

    iter_folder_and_replace(
        (uuid1, uuid2),
        r"C:\Users\27978\Downloads\新建文件夹\server\",
    )?;

    Ok(())
}
