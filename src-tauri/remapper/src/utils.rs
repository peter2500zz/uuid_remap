use std::{collections::HashMap, fs, path::Path};

use anyhow::Result;
use uuid::Uuid;

use crate::map::SymBiMap;

pub fn ensure_no_chain_or_cycle(map: &HashMap<Uuid, Uuid>) -> Result<()> {
    for (k, v) in map {
        // 没有环：key 不能映射到自身
        if k == v {
            anyhow::bail!("发现自环: {k} -> {v}");
        }

        // 没有链：value 不能同时是某个 key
        if map.contains_key(v) {
            anyhow::bail!("发现链: {k} -> {v} -> {}", map[v]);
        }
    }

    Ok(())
}

pub fn ensure_no_duplicate_uuid(map: &HashMap<Uuid, Uuid>) -> Result<()> {
    let mut seen = Vec::new();
    for (k, v) in map {
        if seen.contains(&k) {
            anyhow::bail!("发现重复的 UUID {}", k);
        }
        seen.push(&k);

        if seen.contains(&v) {
            anyhow::bail!("发现重复的 UUID {}", v);
        }
        seen.push(&v);
    }

    Ok(())
}

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

/// 由对称映射生成双向交换的 Aho-Corasick 模式与替换串
///
/// [`SymBiMap::iter`] 每对自带两个方向，无需再手动补反向
pub fn uuid_swap_variants(swaps: &SymBiMap<Uuid>) -> (Vec<String>, Vec<String>) {
    uuid_map_variants(swaps.iter())
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

pub fn create_reverse_map(map: &HashMap<Uuid, Uuid>) -> HashMap<Uuid, Uuid> {
    let mut reverse = HashMap::new();
    for (k, v) in map {
        reverse.insert(*k, *v);
        reverse.insert(*v, *k);
    }
    reverse
}

pub fn atomic_overwrite(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut tmp_name = path.file_name().unwrap_or_default().to_os_string();
    tmp_name.push(".uuid_remap_tmp");
    let tmp = path.with_file_name(tmp_name);

    fs::write(&tmp, bytes)?;
    fs::rename(&tmp, path)?;
    Ok(())
}
