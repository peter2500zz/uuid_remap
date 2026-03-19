use std::{collections::HashMap};

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

pub fn uuid_swap_variants(swaps: &HashMap<Uuid, Uuid>) -> (Vec<String>, Vec<String>) {
    let mut patterns = Vec::new();
    let mut replacements = Vec::new();

    for (a, b) in swaps.iter() {
        let (p_ab, r_ab) = uuid_variants(*a, *b); // A → B
        let (p_ba, r_ba) = uuid_variants(*b, *a); // B → A
        patterns.extend(p_ab);
        replacements.extend(r_ab);
        patterns.extend(p_ba);
        replacements.extend(r_ba);
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

pub fn create_reverse_map(map: &HashMap<Uuid, Uuid>) -> HashMap<Uuid, Uuid> {
    let mut reverse = HashMap::new();
    for (k, v) in map {
        reverse.insert(*k, *v);
        reverse.insert(*v, *k);
    }
    reverse
}
