import { invoke } from "@tauri-apps/api/core";
import style from "../styles/folderSelect.module.css";
import { open } from '@tauri-apps/plugin-dialog';
import { useEffect, useRef, useState } from "react";
import { useAppContext } from "../utils/context";
import { cachePlayerName } from "../utils/getAvatar";


interface UserCache {
    name: string;
    uuid: string;
    expiresOn: string;
}

type ToastKind = "success" | "error" | "warning";

function FolderSelect() {
    const [toast, setToast] = useState<{ kind: ToastKind; message: string } | null>(null);
    const toastTimerRef = useRef<number | null>(null);
    const loadedPathRef = useRef("");

    const {
        worldPathState,
        setWorldPathState,
        setNameMapping,
        setUuidMapping,
    } = useAppContext();

    const hasPath = worldPathState.path.trim() !== "";

    const showToast = (kind: ToastKind, message: string) => {
        setToast({ kind, message });

        if (toastTimerRef.current !== null) {
            window.clearTimeout(toastTimerRef.current);
        }

        toastTimerRef.current = window.setTimeout(() => {
            setToast(null);
            toastTimerRef.current = null;
        }, 1800);
    };

    useEffect(() => {
        const fetchCache = async () => {
            try {
                const caches: UserCache[] = await invoke(
                    "read_cache",
                    { filePath: `${worldPathState.path}/../usercache.json` }
                );
                console.log("读取到的caches:", caches);
                return caches;
            } catch (error) {
                console.error("读取usercache.json失败:", error);
                return [];
            }
        };

        const fetchPlayerData = async () => {
            try {
                const playerData: string[] = await invoke(
                    "read_player_data",
                    { dirPath: `${worldPathState.path}/playerdata` }
                );
                console.log("读取到的playerData:", playerData);
                return playerData;

            } catch (error) {
                console.error("读取playerdata失败:", error);
                return [];
            }
        }

        const fetchAll = async () => {
            const [caches, playerData] = await Promise.all([
                fetchCache(),
                fetchPlayerData(),
            ]);

            // namemap 增量更新
            caches.forEach(cache => {
                cachePlayerName(cache.name, cache.uuid, setNameMapping);
            });
            // UUID map 使用全量覆盖
            const allUuids = new Set([
                ...caches.map(c => c.uuid),
                ...playerData,
            ]);
            setUuidMapping([...allUuids].map(uuid => [uuid, ""]));
        };

        const trimmedPath = worldPathState.path.trim();

        if (worldPathState.isValid && trimmedPath && trimmedPath !== loadedPathRef.current) {
            // 能执行到这里说明选择了新的世界目录
            // 清空 UUID 映射表
            setUuidMapping([]);
            loadedPathRef.current = trimmedPath;

            fetchAll();
        }
    }, [worldPathState]);

    useEffect(() => {
        return () => {
            if (toastTimerRef.current !== null) {
                window.clearTimeout(toastTimerRef.current);
            }
        };
    }, []);

    return (
        <div className={style.container}>
            <div className={style.inputRow}>
                <input
                    className="input input-bordered w-full"
                    placeholder="选择世界的路径"
                    value={worldPathState.path}
                    onChange={async (e) => {
                        // 直接输入时候检查合法性
                        const newPath = e.target.value;

                        setWorldPathState({
                            path: newPath,
                            isValid: await invoke("check_dir", { dirPath: newPath })
                        });
                    }}
                />
                <button
                    className="btn btn-outline"
                    onClick={async () => {
                        // 点按钮的时候也检查
                        const selected = await open({ multiple: false, directory: true });
                        if (selected) {
                            setWorldPathState({
                                path: selected as string,
                                isValid: await invoke("check_dir", { dirPath: selected })
                            });
                        }
                    }}
                >···
                </button>
            </div>
            <button
            className="btn btn-primary"
            onClick={async () => {
                if (!hasPath) {
                    showToast("warning", "目录不能为空");
                    return;
                }

                showToast(
                    worldPathState.isValid ? "success" : "error",
                    worldPathState.isValid ? "有效的世界目录" : "无效的世界目录"
                );
            }}
            >
                检测目录是否有效
            </button> 

            {toast && (
                <div className="toast toast-top toast-center z-50">
                    <div className={`alert ${
                        toast.kind === "success"
                            ? "alert-success"
                            : toast.kind === "error"
                                ? "alert-error"
                                : "alert-warning"
                    }`}>
                        <span>{toast.message}</span>
                    </div>
                </div>
            )}
        </div>
    );
}

export default FolderSelect;
export type { UserCache };
