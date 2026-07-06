use std::{fs, path::Path};

use anyhow::Result;
use infer::MatcherType;
use uuid::Uuid;

pub fn to_u128(a: i32, b: i32, c: i32, d: i32) -> u128 {
    let a = a as u32 as u128;
    let b = b as u32 as u128;
    let c = c as u32 as u128;
    let d = d as u32 as u128;

    (a << 96) | (b << 64) | (c << 32) | d
}

pub fn from_u128(mut value: u128) -> [i32; 4] {
    let mut parts = [0i32; 4];
    for i in (0..4).rev() {
        let part = (value & 0xFFFF_FFFF) as u32;
        parts[i] = part as i32;
        value >>= 32;
    }
    parts
}

pub fn i32s_to_uuid4(values: &[i32]) -> Uuid {
    Uuid::from_u128(to_u128(values[0], values[1], values[2], values[3]))
}

pub fn uuid4_to_i32s(uuid: Uuid) -> [i32; 4] {
    from_u128(uuid.as_u128())
}

pub fn i64pair_to_uuid4(most: i64, least: i64) -> Uuid {
    Uuid::from_u128(((most as u64 as u128) << 64) | (least as u64 as u128))
}

/// 按迭代到的条目原样生成 Aho-Corasick 模式与替换串，不自动补反向
///
/// 如需双向交换，传入 [`SymBiMap::iter`](crate::map::SymBiMap::iter)
/// （每对自带两个方向）或 [`create_reverse_map`] 的结果
pub fn uuid_map_variants<'a>(
    map: impl IntoIterator<Item = (&'a Uuid, &'a Uuid)>,
) -> (Vec<String>, Vec<String>) {
    let mut patterns = Vec::new();
    let mut replacements = Vec::new();

    for (from, to) in map {
        let (p, r) = uuid_variants(*from, *to);
        patterns.extend(p);
        replacements.extend(r);
    }

    (patterns, replacements)
}

// UUID 变体生成
fn uuid_variants(from: Uuid, to: Uuid) -> (Vec<String>, Vec<String>) {
    let from_hyphen = from.to_string();
    let from_upper = from_hyphen.to_uppercase();
    let from_bare = from_hyphen.replace('-', "");
    let from_bare_u = from_upper.replace('-', "");

    let to_hyphen = to.to_string();
    let to_upper = to_hyphen.to_uppercase();
    let to_bare = to_hyphen.replace('-', "");
    let to_bare_u = to_upper.replace('-', "");

    let patterns = vec![from_hyphen, from_upper, from_bare, from_bare_u];
    let replacements = vec![to_hyphen, to_upper, to_bare, to_bare_u];

    (patterns, replacements)
}

pub fn atomic_overwrite(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut tmp_name = path.file_name().unwrap_or_default().to_os_string();
    tmp_name.push(".uuid_remap_tmp");
    let tmp = path.with_file_name(tmp_name);

    fs::write(&tmp, bytes)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

// 非 None 则可以跳过该文件，返回值为文件扩展名
pub fn can_skip_this_file(path: &Path) -> Option<String> {
    // // 判断文件是否过大
    // if let Ok(meta) = fs::metadata(path) {
    //     if (meta.len() > 64 * 1024 * 1024)
    // }

    // 根据 magic number 判断文件类型
    let file_type = match infer::get_from_path(path) {
        Ok(Some(t)) => t,
        _ => return None,
    };

    // 可以跳过的归档文件类型
    let archives_ext = vec!["gz", "jar", "zip", "7z", "xz"];

    // 获取文件扩展名
    let extension = path
        .extension()
        .map(|ostr| ostr.to_string_lossy().to_string())
        .unwrap_or_default();

    // println!(
    //     "[DEBUG] {} IS {:?} {}",
    //     path.display(),
    //     file_type.matcher_type(),
    //     extension
    // );

    match (file_type.matcher_type(), extension.as_str()) {
        // 临时文件
        (_, "uuid_remap_tmp") => Some(extension),
        // 根据扩展名判断，因为 dat 文件的 magic number 是归档文件，但必须处理
        (MatcherType::Archive, ext) if archives_ext.contains(&ext) => Some(extension),
        // 这些文件一半不会存储字符串形式的 UUID
        (
            MatcherType::App
            | MatcherType::Audio
            | MatcherType::Book
            | MatcherType::Doc
            | MatcherType::Font
            | MatcherType::Image
            | MatcherType::Video,
            _,
        ) => Some(extension),
        _ => None,
    }
}
