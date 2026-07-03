use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
};

use remapper::map::{InsertError, SymBiMap};
use serde::Serialize;
use uuid::Uuid;

use crate::{AppState, PlayerData};

#[tauri::command]
pub async fn import_uuid_map(path: PathBuf) -> Result<SymBiMap<Uuid>, String> {
    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let uuid_map: SymBiMap<Uuid> = serde_jsonc::from_str(&content).map_err(|e| e.to_string())?;
    Ok(uuid_map)
}

#[tauri::command]
pub async fn export_uuid_map(
    uuid_map: SymBiMap<Uuid>,
    name_map: HashMap<Uuid, PlayerData>,
    path: PathBuf,
) -> Result<(), String> {
    let mut jsonc = "{".to_string();

    for (index, (left_uuid, right_uuid)) in uuid_map.iter_pairs().enumerate() {
        if index != 0 {
            jsonc.push_str("\n");
        }

        let left_data = name_map.get(&left_uuid);

        let right_data = name_map.get(&right_uuid);

        if left_data.is_some() || right_data.is_some() {
            jsonc.push_str(&format!(
                "\n    // {} <-> {}",
                left_data
                    .map(|pd| format!("{}[{}]", pd.name, pd.mode))
                    .unwrap_or("<anonymous>".into()),
                right_data
                    .map(|pd| format!("{}[{}]", pd.name, pd.mode))
                    .unwrap_or("<anonymous>".into())
            ));
        }

        jsonc.push_str(&format!(
            "\n    \"{}\": \"{}\"{}",
            left_uuid.to_string(),
            right_uuid.to_string(),
            if index < uuid_map.len() - 1 { "," } else { "" }
        ));
    }

    jsonc.push_str("\n}");

    fs::write(path, jsonc).map_err(|e| e.to_string())?;

    Ok(())
}

#[derive(Debug, Serialize)]
pub enum UpdateSide {
    Left,
    Right,
    Both,
}

#[derive(Debug, Serialize)]
pub struct UpdateError {
    index: usize,
    side: UpdateSide,
    error: InsertError<Uuid>,
}

/// 校验并构建映射表；标记与插入解耦，保证：
/// 1. 一行两侧同时冲突时报 Both 而不是只报先被检查的 Left；
/// 2. 失败行的 UUID 也视为已占用，与其冲突的后续行同样会被标记。
fn build_uuid_map(uuid_pairs: &[(Uuid, Uuid)]) -> Result<SymBiMap<Uuid>, Vec<UpdateError>> {
    let mut new_uuid_map = SymBiMap::new();
    let mut used_by_failed: HashSet<Uuid> = HashSet::new();
    let mut errors = Vec::new();

    for (index, (left_uuid, right_uuid)) in uuid_pairs.iter().enumerate() {
        if left_uuid == right_uuid {
            errors.push(UpdateError {
                index,
                side: UpdateSide::Both,
                error: InsertError::SelfPair(*left_uuid),
            });
            used_by_failed.insert(*left_uuid);
            continue;
        }

        let taken = |uuid: &Uuid| new_uuid_map.contains(uuid) || used_by_failed.contains(uuid);
        let (side, dup_uuid) = match (taken(left_uuid), taken(right_uuid)) {
            (true, true) => (UpdateSide::Both, *left_uuid),
            (true, false) => (UpdateSide::Left, *left_uuid),
            (false, true) => (UpdateSide::Right, *right_uuid),
            (false, false) => {
                new_uuid_map
                    .insert(*left_uuid, *right_uuid)
                    .expect("自环与重复已预检");
                continue;
            }
        };
        errors.push(UpdateError {
            index,
            side,
            error: InsertError::Duplicate(dup_uuid),
        });
        used_by_failed.insert(*left_uuid);
        used_by_failed.insert(*right_uuid);
    }

    if errors.is_empty() {
        Ok(new_uuid_map)
    } else {
        Err(errors)
    }
}

#[tauri::command]
pub async fn update_uuid_map(
    state: tauri::State<'_, AppState>,
    uuid_pairs: Vec<(Uuid, Uuid)>,
) -> Result<(), Vec<UpdateError>> {
    let new_uuid_map = build_uuid_map(&uuid_pairs)?;
    *state.uuid_map.lock().unwrap() = new_uuid_map;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn uuid(n: u32) -> Uuid {
        Uuid::from_str(&format!("00000000-0000-4000-8000-{:012x}", n)).unwrap()
    }

    #[test]
    fn both_sides_conflicting_reports_both() {
        let (a, b) = (uuid(1), uuid(2));
        let errors = build_uuid_map(&[(a, b), (a, b)]).unwrap_err();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].index, 1);
        assert!(matches!(errors[0].side, UpdateSide::Both));
        assert!(matches!(errors[0].error, InsertError::Duplicate(_)));
    }

    #[test]
    fn conflict_with_failed_row_is_reported() {
        let (c, d) = (uuid(3), uuid(4));
        // 第 0 行自环失败，第 1 行与失败行共用 c，也应被标记
        let errors = build_uuid_map(&[(c, c), (c, d)]).unwrap_err();
        assert_eq!(errors.len(), 2);
        assert!(matches!(errors[0].error, InsertError::SelfPair(_)));
        assert_eq!(errors[1].index, 1);
        assert!(matches!(errors[1].side, UpdateSide::Left));
        assert_eq!(errors[1].error, InsertError::Duplicate(c));
    }

    #[test]
    fn valid_rows_still_build_the_map() {
        let map = build_uuid_map(&[(uuid(1), uuid(2)), (uuid(3), uuid(4))]).unwrap();
        assert_eq!(map.len(), 2);
        assert_eq!(map.get(&uuid(2)), Some(&uuid(1)));
    }
}
