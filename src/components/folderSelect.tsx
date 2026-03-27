import { invoke } from "@tauri-apps/api/core";
import { open } from '@tauri-apps/plugin-dialog';
import { useAppContext } from "../utils/context";
import { cachePlayerName } from "../utils/getAvatar";
import { dirname } from "@tauri-apps/api/path";


interface UserCache {
    name: string;
    uuid: string;
    expiresOn: string;
}

function FolderSelect() {

    const {
        worldPathState,
        setWorldPathState,
        setNameMapping,
        setUuidMapping,
    } = useAppContext();

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
        try {
            const playerData: string[] = await invoke(
                "read_player_data",
                { dirPath: `${dir}/playerdata` }
            );
            console.log("读取到的playerData:", playerData);
            return playerData;

        } catch (error) {
            console.error("读取playerdata失败:", error);
            return [];
        }
    }

    const fetchAll = async (dir: string) => {
        const [caches, playerData] = await Promise.all([
            fetchCache(dir),
            fetchPlayerData(dir),
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

    function GenerateAlert() {
        if (!worldPathState.path || worldPathState.type === "NotExist") {
            return (
                <></>
            )
        } else {
            if (worldPathState.type === "World") {
                return (
                    <div role="alert" className="alert">
                        <span>
                            ✅ 检测到世界目录
                        </span>
                    </div>
                )
            } else if (worldPathState.type === "WorldButHasServer") {
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
                                <button className="btn btn-sm btn-primary" onClick={async () => {
                                    setWorldPathState({
                                        path: await dirname(worldPathState.path),
                                        type: "Server",
                                    })
                                }}>是的</button><button className="btn btn-sm" onClick={() => {
                                    setWorldPathState({
                                        path: worldPathState.path,
                                        type: "World",
                                    })
                                }}>不了</button>
                            </div>
                        </div>
                    </div>
                )
            } else if (worldPathState.type === "Server") {
                return (
                    <div role="alert" className="alert">
                        <span>
                            ✅ 检测到服务器目录
                        </span>
                    </div>
                )
            } else {
                return (
                    <div className="flex flex-col gap-4">
                        <div role="alert" className="alert">
                            <span>
                                ❌ 这个目录看起来既不是存档也不是服务器文件夹。
                            </span>
                        </div>

                        {(worldPathState.type === "Invalid") &&
                            <div role="alert" className="alert flex justify-between">
                                <span>仍然使用它？</span>
                                <button className="btn btn-sm" onClick={() => {
                                    setWorldPathState({
                                        path: worldPathState.path,
                                        type: "InvalidButForce",
                                    })
                                    fetchAll(worldPathState.path);
                                }}>是的</button>
                            </div>
                        }

                    </div>
                )
            }
        }
    }

    const updatePath = async (dir: string) => {
        const serverResult = await invoke<[string, string] | null>("check_server_dir", { dirPath: dir });

        if (serverResult) {
            fetchAll(serverResult[1]);
            setWorldPathState({
                path: dir,
                type: "Server",
            });


            return;
        }

        const worldResult = await invoke<string | null>("check_world_dir", { dirPath: dir });
        if (worldResult) {
            const doesServerExits = await invoke<[string, string] | null>("check_server_dir", { dirPath: `${dir}/../` });

            fetchAll(worldResult);
            setWorldPathState({
                path: dir,
                type: doesServerExits ? "WorldButHasServer" : "World",
            });

            return;
        }

        // console.warn("既不是服务器目录也不是世界目录，标记为无效，但用户选择了使用它", dir);

        setWorldPathState({
            path: dir,
            type: await invoke<boolean>("check_dir_exist", { dirPath: dir }) ? "Invalid" : "NotExist",
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
                    onChange={async (e) => {
                        const newPath = e.target.value;

                        setWorldPathState(prev => ({ ...prev, path: newPath }));

                        await updatePath(newPath);
                    }}
                />

                <button
                    className="btn absolute right-2 top-1/2 -translate-y-1/2 w-20 h-10"
                    onClick={async () => {
                        // 点按钮的时候也检查

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
                <GenerateAlert />
            </div>
        </div>
    );
}

export default FolderSelect;
export type { UserCache };
