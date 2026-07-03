use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::hash::Hash;
use std::marker::PhantomData;

use serde::de::{self, Deserialize, Deserializer, MapAccess, Visitor};
use serde::ser::{Serialize, SerializeMap, Serializer};

/// SymBiMap 是一种扁平的双向映射结构。
/// 它保证每个元素只能出现在一对中，且 a->b 与 b->a 是对称的。
#[derive(Debug, Default, Clone)]
pub struct SymBiMap<T> {
    map: HashMap<T, T>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum InsertError<T> {
    /// a 或 b 已经存在于某一对中。
    Duplicate(T),
    /// a == b
    SelfPair(T),
}

impl<T: Eq + Hash> SymBiMap<T> {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// 查询某个元素的搭档，此操作是对称的。
    pub fn get(&self, key: &T) -> Option<&T> {
        self.map.get(key)
    }

    /// 查询某个元素是否存在于某一对中。
    pub fn contains(&self, key: &T) -> bool {
        self.map.contains_key(key)
    }

    /// 用任意一端删除整对，返回 (被查的那个, 它的搭档)。
    pub fn remove(&mut self, key: &T) -> Option<(T, T)> {
        let partner = self.map.remove(key)?;
        let this = self.map.remove(&partner)?;
        Some((this, partner))
    }

    /// 返回总对数。
    pub fn len(&self) -> usize {
        self.map.len() / 2
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

impl<T: Eq + Hash + Clone> SymBiMap<T> {
    /// 插入一对元素。
    pub fn insert(&mut self, a: T, b: T) -> Result<(), InsertError<T>> {
        if a == b {
            return Err(InsertError::SelfPair(a));
        }
        if self.map.contains_key(&a) {
            return Err(InsertError::Duplicate(a));
        }
        if self.map.contains_key(&b) {
            return Err(InsertError::Duplicate(b));
        }
        self.map.insert(a.clone(), b.clone());
        self.map.insert(b, a);
        Ok(())
    }
}

impl<T> Serialize for SymBiMap<T>
where
    T: Serialize + Eq + Hash,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // 内部每对存了 a->b 和 b->a 两条，这里每对只输出一次。
        let mut seen: HashSet<&T> = HashSet::with_capacity(self.len());
        let mut map = serializer.serialize_map(Some(self.len()))?;
        for (k, v) in &self.map {
            // 若这一对已从另一方向输出过，跳过。
            if seen.contains(k) || seen.contains(v) {
                continue;
            }
            seen.insert(k);
            seen.insert(v);
            map.serialize_entry(k, v)?;
        }
        map.end()
    }
}

impl<'de, T> Deserialize<'de> for SymBiMap<T>
where
    T: Deserialize<'de> + Eq + Hash + Clone,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SymBiMapVisitor<T>(PhantomData<T>);

        impl<'de, T> Visitor<'de> for SymBiMapVisitor<T>
        where
            T: Deserialize<'de> + Eq + Hash + Clone,
        {
            type Value = SymBiMap<T>;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a flat map of unique, symmetric pairs")
            }

            fn visit_map<A>(self, mut access: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut out = SymBiMap::new();
                if let Some(hint) = access.size_hint() {
                    out.map.reserve(hint * 2); // 同模块内可访问私有字段
                }
                // 关键：每对都走 insert，让不变量在反序列化时就被强制执行。
                while let Some((a, b)) = access.next_entry::<T, T>()? {
                    out.insert(a, b).map_err(|e| match e {
                        InsertError::SelfPair(_) => {
                            de::Error::custom("invalid pair: a key maps to itself")
                        }
                        InsertError::Duplicate(_) => {
                            de::Error::custom("invalid pair: an element appears more than once")
                        }
                    })?;
                }
                Ok(out)
            }
        }

        deserializer.deserialize_map(SymBiMapVisitor(PhantomData))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_get_are_symmetric() {
        let mut map = SymBiMap::new();
        map.insert(1, 2).unwrap();
        assert_eq!(map.get(&1), Some(&2));
        assert_eq!(map.get(&2), Some(&1));
        assert_eq!(map.get(&3), None);
    }

    #[test]
    fn insert_rejects_self_pair() {
        let mut map = SymBiMap::new();
        assert_eq!(map.insert(1, 1), Err(InsertError::SelfPair(1)));
        assert!(map.is_empty());
    }

    #[test]
    fn insert_rejects_duplicates_on_either_side() {
        let mut map = SymBiMap::new();
        map.insert(1, 2).unwrap();
        assert_eq!(map.insert(1, 3), Err(InsertError::Duplicate(1)));
        assert_eq!(map.insert(3, 2), Err(InsertError::Duplicate(2)));
        // 已有对的两端调换方向也算重复
        assert_eq!(map.insert(2, 3), Err(InsertError::Duplicate(2)));
    }

    #[test]
    fn failed_insert_leaves_map_unchanged() {
        let mut map = SymBiMap::new();
        map.insert(1, 2).unwrap();
        assert!(map.insert(3, 1).is_err());
        // 冲突发生在 b 端时，a 端不应被插入一半
        assert!(!map.contains(&3));
        assert_eq!(map.get(&1), Some(&2));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn contains_checks_both_ends() {
        let mut map = SymBiMap::new();
        map.insert(1, 2).unwrap();
        assert!(map.contains(&1));
        assert!(map.contains(&2));
        assert!(!map.contains(&3));
    }

    #[test]
    fn remove_works_from_either_end() {
        let mut map = SymBiMap::new();
        map.insert(1, 2).unwrap();
        assert_eq!(map.remove(&2), Some((2, 1)));
        assert!(map.is_empty());
        assert_eq!(map.get(&1), None);
        assert_eq!(map.get(&2), None);
        assert_eq!(map.remove(&1), None);
        // 删除后同样的元素可重新配对
        map.insert(1, 2).unwrap();
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn len_counts_pairs() {
        let mut map: SymBiMap<i32> = SymBiMap::new();
        assert_eq!(map.len(), 0);
        assert!(map.is_empty());
        map.insert(1, 2).unwrap();
        map.insert(3, 4).unwrap();
        assert_eq!(map.len(), 2);
        assert!(!map.is_empty());
    }

    #[test]
    fn serialize_outputs_each_pair_once() {
        let mut map = SymBiMap::new();
        map.insert("a".to_string(), "b".to_string()).unwrap();
        map.insert("c".to_string(), "d".to_string()).unwrap();
        let json = serde_jsonc::to_string(&map).unwrap();
        // 内部存了双向两条，输出条目数应等于对数
        let flat: HashMap<String, String> = serde_jsonc::from_str(&json).unwrap();
        assert_eq!(flat.len(), 2);
        // 每对输出方向不确定，但必须与原映射一致
        for (k, v) in &flat {
            assert_eq!(map.get(k), Some(v));
        }
        let mut elems: Vec<&str> = flat
            .iter()
            .flat_map(|(k, v)| [k.as_str(), v.as_str()])
            .collect();
        elems.sort();
        assert_eq!(elems, ["a", "b", "c", "d"]);
    }

    #[test]
    fn deserialize_builds_symmetric_map() {
        let map: SymBiMap<String> = serde_jsonc::from_str(r#"{"a": "b", "c": "d"}"#).unwrap();
        assert_eq!(map.len(), 2);
        assert_eq!(map.get(&"b".to_string()), Some(&"a".to_string()));
        assert_eq!(map.get(&"d".to_string()), Some(&"c".to_string()));
    }

    #[test]
    fn deserialize_rejects_self_pair() {
        let result: Result<SymBiMap<String>, _> = serde_jsonc::from_str(r#"{"a": "a"}"#);
        let err = result.unwrap_err().to_string();
        assert!(err.contains("maps to itself"), "unexpected error: {err}");
    }

    #[test]
    fn deserialize_rejects_duplicate_elements() {
        // "b" 同时出现在两对中（一次作值、一次作键）
        let result: Result<SymBiMap<String>, _> = serde_jsonc::from_str(r#"{"a": "b", "b": "c"}"#);
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("appears more than once"),
            "unexpected error: {err}"
        );
        // 两对的值端重复同样被拒绝
        let result: Result<SymBiMap<String>, _> = serde_jsonc::from_str(r#"{"a": "b", "c": "b"}"#);
        assert!(result.is_err());
    }

    #[test]
    fn serde_roundtrip_preserves_pairs() {
        let mut map = SymBiMap::new();
        for (a, b) in [("a", "b"), ("c", "d"), ("e", "f")] {
            map.insert(a.to_string(), b.to_string()).unwrap();
        }
        let json = serde_jsonc::to_string(&map).unwrap();
        let restored: SymBiMap<String> = serde_jsonc::from_str(&json).unwrap();
        assert_eq!(restored.len(), map.len());
        for (k, v) in &map.map {
            assert_eq!(restored.get(k), Some(v));
        }
    }
}
