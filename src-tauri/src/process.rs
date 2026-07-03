use std::path::Path;

use remapper::world::ProgressEvent;
use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::AppState;

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "kind", content = "data")]
pub enum ProcessError {
    Other(String),
    PoisonedLock,
}

#[tauri::command]
pub async fn process_world(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    world_path: String,
) -> Result<(), ProcessError> {
    let uuid_map = state
        .uuid_map
        .lock()
        .map_err(|_| ProcessError::PoisonedLock)?
        .clone();

    // 核心逻辑在 remapper 中，这里只负责把进度事件转发给前端
    let emit_progress = |event: ProgressEvent| {
        let _ = match event {
            ProgressEvent::SetTotal(total) => app.emit("set-total", total),
            ProgressEvent::StartPhase(phase) => app.emit("start-phase", phase),
            ProgressEvent::StartTask(path) => app.emit("start-task", path),
            ProgressEvent::FinishTask(result) => app.emit("finish-task", result),
        };
    };

    remapper::world::process_world(Path::new(&world_path), &uuid_map, emit_progress)
        .map_err(|e| ProcessError::Other(e.to_string()))
}
