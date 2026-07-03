use aho_corasick::AhoCorasick;
use anyhow::Result;
use chardetng::{EncodingDetector, Iso2022JpDetection, Utf8Detection};
use content_inspector::inspect;
use serde::Serialize;
use std::{fs, path::Path};
use uuid::Uuid;

use crate::{
    map::SymBiMap,
    utils::{atomic_overwrite, uuid_map_variants},
};

#[derive(Debug, Serialize)]
#[serde(tag = "kind", content = "data")]
pub enum FileContentSwapResult {
    IsBinary,
    NoChange,
    Changed,
}

pub fn swap_uuids_in_string(content: &str, uuid_map: &SymBiMap<Uuid>) -> String {
    let (patterns, replacements) = uuid_map_variants(uuid_map.iter());
    let ac = AhoCorasick::new(&patterns).unwrap();
    ac.replace_all(content, &replacements)
}

pub fn swap_uuids_in_file(path: &Path, swaps: &SymBiMap<Uuid>) -> Result<FileContentSwapResult> {
    let bytes = fs::read(path)?;

    if !inspect(&bytes).is_text() {
        return Ok(FileContentSwapResult::IsBinary);
    }

    let mut detector = EncodingDetector::new(Iso2022JpDetection::Allow);
    detector.feed(&bytes, true);
    let encoding = detector.guess(None, Utf8Detection::Allow);
    let (cow, _, had_errors) = encoding.decode(&bytes);
    if had_errors {
        // eprintln!("警告：解码时部分字节无法识别");
    }

    let new_content = swap_uuids_in_string(&cow, swaps);

    let (encoded, _, _) = encoding.encode(&new_content);
    // 如果没区别就不写了
    if encoded == bytes {
        // println!("文件 {:?} 内容未发生变化，已跳过", path);
        return Ok(FileContentSwapResult::NoChange);
    }

    atomic_overwrite(path, &encoded)?;

    Ok(FileContentSwapResult::Changed)
}

// #[test]
// fn replace() -> Result<()> {
//     // 以文件为单位

//     use std::str::FromStr;

//     let uuid1 = Uuid::from_str("9db4226c-1015-40da-8fa5-4335aab896b6")?;
//     let uuid2 = Uuid::from_str("59c66d96-d356-364a-a84e-0511b286a31b")?;
//     let uuid3 = Uuid::from_str("00000000-0000-0000-0000-000000000000")?;
//     let uuid4 = Uuid::from_str("ffffffff-ffff-ffff-ffff-ffffffffffff")?;

//     let map = HashMap::from([(uuid1, uuid2), (uuid3, uuid4)]);

//     swap_uuids_in_file(
//         Path::new(r"C:\Users\27978\Downloads\新建文件夹\server\world\entities\r.0.0.mca"),
//         &map,
//     )?;

//     Ok(())
// }
