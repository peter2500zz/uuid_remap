use aho_corasick::AhoCorasick;
use anyhow::Result;
use chardetng::EncodingDetector;
use std::{collections::HashMap, fs, path::Path};
use content_inspector::inspect;
use uuid::Uuid;

use crate::utils::uuid_swap_variants;

pub fn swap_uuids_in_string(content: &str, swaps: &HashMap<Uuid, Uuid>) -> String {
    let (patterns, replacements) = uuid_swap_variants(swaps);
    let ac = AhoCorasick::new(&patterns).unwrap();
    ac.replace_all(content, &replacements)
}

pub fn swap_uuids_in_file(path: &Path, swaps: &HashMap<Uuid, Uuid>) -> Result<()> {
    let bytes = fs::read(path)?;

    if !inspect(&bytes).is_text() {
        return Err(anyhow::anyhow!("文件是二进制文件，已跳过"));
    }

    let mut detector = EncodingDetector::new();
    detector.feed(&bytes, true);
    let encoding = detector.guess(None, true);
    let (cow, _, had_errors) = encoding.decode(&bytes);
    if had_errors {
        // eprintln!("警告：解码时部分字节无法识别");
    }

    let new_content = swap_uuids_in_string(&cow, swaps);

    let (encoded, _, _) = encoding.encode(&new_content);
    // 如果没区别就不写了
    if encoded == bytes {
        // println!("文件 {:?} 内容未发生变化，已跳过", path);
        return Err(anyhow::anyhow!("文件内容未发生变化，已跳过"));
    }
    fs::write(path, encoded.as_ref())?;

    // println!(
    //     "编码: {}，{} 对 UUID 互换完成",
    //     encoding.name(),
    //     swaps.len()
    // );
    Ok(())
}

#[test]
fn replace() -> Result<()> {
    // 以文件为单位

    use std::str::FromStr;

    let uuid1 = Uuid::from_str("9db4226c-1015-40da-8fa5-4335aab896b6")?;
    let uuid2 = Uuid::from_str("59c66d96-d356-364a-a84e-0511b286a31b")?;
    let uuid3 = Uuid::from_str("00000000-0000-0000-0000-000000000000")?;
    let uuid4 = Uuid::from_str("ffffffff-ffff-ffff-ffff-ffffffffffff")?;

    let map = HashMap::from([(uuid1, uuid2), (uuid3, uuid4)]);

    swap_uuids_in_file(
        Path::new(r"C:\Users\27978\Downloads\新建文件夹\server\world\entities\r.0.0.mca"),
        &map,
    )?;

    Ok(())
}
