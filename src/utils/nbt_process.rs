use crab_nbt::{NbtCompound, NbtTag};

/// 对组合型Nbt数据的每一对键值应用handler</br>
/// 返回处理后的数据
pub fn process_compound<F>(compound: &NbtCompound, handler: F) -> Option<NbtCompound>
where
    F: Fn(&String, &NbtTag) -> Option<NbtTag>,
{
    let mut new_compound = NbtCompound::new();

    for (tag_name, nbt_tag) in &compound.child_tags {
        new_compound.put(tag_name.clone(), handler(tag_name, nbt_tag)?);
    }

    Some(new_compound)
}

/// 对列表型Nbt数据的每个值应用handler</br>
/// 返回处理后的数据
pub fn process_list<F>(list: &Vec<NbtTag>, handler: F) -> Option<Vec<NbtTag>>
where
    F: Fn(&NbtTag) -> Option<NbtTag>,
{
    let mut new_list = Vec::new();

    for nbt_tag in list {
        new_list.push(handler(nbt_tag)?);
    }

    Some(new_list)
}
