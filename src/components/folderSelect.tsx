import { invoke } from "@tauri-apps/api/core";
import style from "./folderSelect.module.css";
import { open } from '@tauri-apps/plugin-dialog';
import { useEffect, useState } from "react";
import { useAppContext } from "../context";
import { cachePlayerName } from "../getAvatar";


interface UserCache {
    name: string;
    uuid: string;
    expiresOn: string;
}

function FolderSelect() {
    const [display, setDisplay] = useState(true);
    const {
        worldPathState,
        setWorldPathState,
        nameMapping,
        setNameMapping,
        setUuidMapping,
    } = useAppContext();

    useEffect(() => {
        const fetchCache = async () => {
            try {
                const caches: UserCache[] = await invoke(
                    "read_cache",
                    { filePath: `${worldPathState.path}/../usercache.json` }
                );

                console.log("读取到的caches:", caches);

                // // 增量修改名称映射表
                // const newNameEntries: Record<string, string> = {};
                // caches.forEach(cache => {
                //     newNameEntries[cache.uuid] = cache.name;
                // });

                // setNameMapping(prev => ({
                //     ...prev,
                //     ...newNameEntries
                // }));

                // // 增量添加 UUID 映射表，使用左值作为主键
                // const existingKeys = new Set(uuidMapping.map(([k, _]) => k));
                // const newUuidEntries: [string, string][] = caches
                //     .filter(cache => !existingKeys.has(cache.uuid))
                //     .map(cache => [cache.uuid, ""]);

                // setUuidMapping(prev => [...prev, ...newUuidEntries]);

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

                // // 同样增量添加 UUID 映射表，使用左值作为主键
                // const existingKeys = new Set(uuidMapping.map(([k, _]) => k));
                // const newUuidEntries: [string, string][] = playerData
                //     .filter(uuid => !existingKeys.has(uuid))
                //     .map(uuid => [uuid, ""]);
                // setUuidMapping(prev => [...prev, ...newUuidEntries]);
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

        if (worldPathState.isValid) {
            // 能执行到这里说明选择了新的世界目录

            // 清空 UUID 映射表
            setUuidMapping([]);

            fetchAll();
        }
    }, [worldPathState]);

    return (
        <div className={style.container}>
            <button onClick={() => setDisplay(!display)}>定位你的世界文件夹</button>

            {display && (
                <div>
                    <input value={worldPathState.path} onChange={async (e) => {
                        // 直接输入时候检查合法性
                        const newPath = e.target.value;

                        setWorldPathState({
                            path: newPath,
                            isValid: await invoke("check_dir", { dirPath: newPath })
                        });
                    }} />
                    <button onClick={async () => {
                        // 点按钮的时候也检查
                        const selected = await open({ multiple: false, directory: true });
                        if (selected) {

                            setWorldPathState({
                                path: selected as string,
                                isValid: await invoke("check_dir", { dirPath: selected })
                            });
                        }
                    }}>...</button>
                    <div>
                        {worldPathState.isValid ? "有效的世界目录" : "无效的世界目录"}
                    </div>
                </div>
            )}
        </div>
    );
}

export default FolderSelect;
export type { UserCache };
