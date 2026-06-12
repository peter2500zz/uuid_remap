import { invoke } from "@tauri-apps/api/core";
import { open } from '@tauri-apps/plugin-dialog';
import { useRef } from "react";
import { useAppContext, WorldPathState } from "../utils/context";
import { cachePlayerByUuid, cachePlayerName } from "../utils/getAvatar";
import { normalizeUUID } from "../utils/uuidUtils";
import { dirname } from "@tauri-apps/api/path";
import toast from "react-hot-toast";


interface UserCache {
    name: string;
    uuid: string;
    expiresOn: string;
}

// 根据路径检测结果展示提示，以及后续操作的入口
function PathStatusAlert({ state, onUseServerDir, onKeepWorldDir, onForceUse }: {
    state: WorldPathState;
    onUseServerDir: () => void;
    onKeepWorldDir: () => void;
    onForceUse: () => void;
}) {
    if (!state.path || state.type === "NotExist") {
        return null;
    }

    switch (state.type) {
        case "World":
            return (
                <div role="alert" className="alert">
                    <span>✅ 检测到世界目录</span>
                </div>
            );
        case "Server":
            return (
                <div role="alert" className="alert">
                    <span>✅ 检测到服务器目录</span>
                </div>
            );
        case "WorldButHasServer":
            return (
                <div className="flex flex-col gap-4">
                    <div role="alert" className="alert">
                        <span>
                            ✅ 检测到了游戏存档，但是它似乎在一个服务器文件夹下。<br />
                            如果这是服务器的存档，推荐选择服务器文件夹以获得更完全的处理。
                        </span>
                    </div>
                    <div role="alert" className="alert flex justify-between">
                        <span>你想要使用服务器文件夹吗？</span>
                        <div className="flex gap-2">
                            <button className="btn btn-sm btn-primary" onClick={onUseServerDir}>是的</button>
                            <button className="btn btn-sm" onClick={onKeepWorldDir}>不了</button>
                        </div>
                    </div>
                </div>
            );
        default:
            return (
                <div className="flex flex-col gap-4">
                    <div role="alert" className="alert">
                        <span>❌ 这个目录看起来既不是存档也不是服务器文件夹。</span>
                    </div>
                    {state.type === "Invalid" &&
                        <div role="alert" className="alert flex justify-between">
                            <span>仍然使用它？</span>
                            <button className="btn btn-sm" onClick={onForceUse}>是的</button>
                        </div>
                    }
                </div>
            );
    }
}

function FolderSelect() {
    const {
        worldPathState,
        setWorldPathState,
        setPlayerInfoMap,
        setUuidPairs,
    } = useAppContext();
    // 输入时每个按键都会触发一次检测，用序号丢弃过期的检测结果
    const updateSeqRef = useRef(0);

    const fetchCache = async (dir: string) => {
        try {
            const caches: UserCache[] = await invoke(
                "read_cache",
                { filePath: `${dir}/../usercache.json` }
            );
            console.log("读取到的caches:", caches);
            return caches;
        } catch (error) {
            console.error("读取usercache.json失败:", error);
            return [];
        }
    };

    const fetchPlayerData = async (dir: string) => {
        // 新版存档的玩家数据在 players/data，旧版在 playerdata，按顺序尝试
        for (const subdir of ["players/data", "playerdata"]) {
            try {
                const playerData: string[] = await invoke(
                    "read_player_data",
                    { dirPath: `${dir}/${subdir}` }
                );
                if (playerData.length > 0) {
                    console.log(`从 ${subdir} 读取到的playerData:`, playerData);
                    return playerData;
                }
            } catch (error) {
                console.warn(`读取 ${subdir} 失败:`, error);
            }
        }

        toast.error("未能从 players/data 或 playerdata 读取到玩家数据");
        return [];
    }

    const fetchAll = async (dir: string) => {
        const [caches, playerData] = await Promise.all([
            fetchCache(dir),
            fetchPlayerData(dir),
        ]);

        // 玩家信息做增量更新
        caches.forEach(cache => {
            cachePlayerName(cache.name, cache.uuid, setPlayerInfoMap);
        });
        // usercache 没覆盖到的扫描 UUID，尝试反查在线玩家信息
        const cachedUuids = new Set(caches.map(c => normalizeUUID(c.uuid) ?? c.uuid));
        playerData
            .filter(uuid => !cachedUuids.has(normalizeUUID(uuid) ?? uuid))
            .forEach(uuid => cachePlayerByUuid(uuid, setPlayerInfoMap));
        // 交换列表全量覆盖
        const allUuids = new Set([
            ...caches.map(c => c.uuid),
            ...playerData,
        ]);
        setUuidPairs([...allUuids].map(uuid => [uuid, ""]));
    };

    const updatePath = async (dir: string) => {
        const seq = ++updateSeqRef.current;
        const isStale = () => seq !== updateSeqRef.current;

        const serverResult = await invoke<[string, string] | null>("check_server_dir", { dirPath: dir });
        if (isStale()) return;

        if (serverResult) {
            fetchAll(serverResult[1]);
            setWorldPathState({ path: dir, type: "Server" });
            return;
        }

        const worldResult = await invoke<string | null>("check_world_dir", { dirPath: dir });
        if (isStale()) return;

        if (worldResult) {
            const serverDirExists = await invoke<[string, string] | null>("check_server_dir", { dirPath: `${dir}/../` });
            if (isStale()) return;

            fetchAll(worldResult);
            setWorldPathState({
                path: dir,
                type: serverDirExists ? "WorldButHasServer" : "World",
            });
            return;
        }

        const dirExists = await invoke<boolean>("check_dir_exist", { dirPath: dir });
        if (isStale()) return;

        setWorldPathState({
            path: dir,
            type: dirExists ? "Invalid" : "NotExist",
        });
    }

    return (
        <div className="min-h-screen flex flex-col items-center justify-start pt-32 gap-6">
            <h1 className="text-5xl font-bold tracking-tight">
                UUID 交换器
            </h1>

            <div className="relative w-full max-w-xl">
                <input
                    className="input w-full px-5 py-6 border shadow-sm"
                    placeholder="/path/to/world"
                    value={worldPathState.path}
                    onChange={e => {
                        const newPath = e.target.value;
                        setWorldPathState(prev => ({ ...prev, path: newPath }));
                        updatePath(newPath);
                    }}
                />

                <button
                    className="btn absolute right-2 top-1/2 -translate-y-1/2 w-20 h-10"
                    onClick={async () => {
                        const selected = await open({ multiple: false, directory: true });
                        if (selected) {
                            await updatePath(selected);
                        }
                    }}
                >
                    浏览
                </button>
            </div>

            <div className="flex gap-4">
                <PathStatusAlert
                    state={worldPathState}
                    onUseServerDir={async () => {
                        setWorldPathState({
                            path: await dirname(worldPathState.path),
                            type: "Server",
                        });
                    }}
                    onKeepWorldDir={() => {
                        setWorldPathState({
                            path: worldPathState.path,
                            type: "World",
                        });
                    }}
                    onForceUse={() => {
                        setWorldPathState({
                            path: worldPathState.path,
                            type: "InvalidButForce",
                        });
                        fetchAll(worldPathState.path);
                    }}
                />
            </div>
        </div>
    );
}

export default FolderSelect;
export type { UserCache };
