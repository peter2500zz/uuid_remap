import { invoke } from "@tauri-apps/api/core";
import { UuidPair } from "./uuidUtils";

// 与 src-tauri 侧 serde 序列化形状一一对应的镜像类型。
// 带字段的枚举使用 #[serde(tag = "kind", content = "data")]（world.rs / process.rs），
// InsertError 是 serde 默认的外部标签（remapper/src/map.rs），
// 无字段枚举（UpdateSide）序列化为纯字符串。

// ---- update_uuid_map（src-tauri/src/map.rs）----

type UpdateSide = "Left" | "Right" | "Both";

type InsertError =
    | { Duplicate: string }
    | { SelfPair: string };

interface UpdateError {
    index: number;
    side: UpdateSide;
    error: InsertError;
}

/// 全量提交交换表到后端 state；任何一行无效则整体不生效，reject UpdateError[]。
/// 格式合法性由前端保证（无效 UUID 会死在参数反序列化，拿不到结构化错误）
function updateUuidMap(uuidPairs: UuidPair[]): Promise<void> {
    return invoke("update_uuid_map", {
        uuidPairs: uuidPairs.map(p => [p.left, p.right]),
    });
}

// 提交失败的兜底文案：本地校验先于提交拦下所有已知问题，
// 这里只在两边判定不一致时兜底，所以不做 i18n，拼一个可读的诊断串
function updateErrorText(e: unknown): string {
    if (!Array.isArray(e)) return String(e);
    return (e as UpdateError[])
        .map(({ index, side, error }) => {
            const [kind, uuid] = "Duplicate" in error
                ? ["duplicate", error.Duplicate]
                : ["self-pair", error.SelfPair];
            return `#${index + 1}[${side}] ${kind}: ${uuid}`;
        })
        .join("; ");
}

// ---- process_world（src-tauri/src/process.rs）----

type ProcessError =
    | { kind: "WorldNotFound"; data: string }
    | { kind: "PoisonedLock" };

/// 用后端 state 中已提交的交换表处理存档，进度经由窗口事件上报
function processWorld(worldPath: string): Promise<void> {
    return invoke("process_world", { worldPath });
}

function processErrorText(e: unknown): string {
    const err = e as ProcessError;
    switch (err?.kind) {
        case "WorldNotFound": return err.data;
        case "PoisonedLock": return "PoisonedLock";
        default: return String(e);
    }
}

// ---- 进度事件（remapper/src/world.rs，经 process.rs 逐 variant 转发）----
// 事件通道：set-total(number) / start-phase(0|1) / start-task(string) / finish-task(FinishTaskData)

type FileProcessError =
    | { kind: "McaError"; data: string }
    | { kind: "NbtError"; data: string }
    | { kind: "ContentError"; data: string }
    | { kind: "RenameError"; data: string };

type TaskResult =
    | { kind: "Success" }
    | { kind: "NoChange" }
    | { kind: "Unsupported"; data: FileProcessError[] }
    | { kind: "Error"; data: FileProcessError[] };

interface FinishTaskData {
    path: string;
    result: TaskResult;
}

export { updateUuidMap, updateErrorText, processWorld, processErrorText };
export type { UpdateSide, InsertError, UpdateError, ProcessError, FileProcessError, TaskResult, FinishTaskData };
