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
