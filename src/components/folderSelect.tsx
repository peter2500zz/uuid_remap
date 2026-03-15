import { invoke } from "@tauri-apps/api/core";
import style from "./folderSelect.module.css";
import { open } from '@tauri-apps/plugin-dialog';
import { useEffect, useState } from "react";
import { useAppContext } from "../context";


type UserCache = {
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
        uuidMapping,
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
                const newNameMappingEntries = caches.reduce((acc, cache) =>
                    ({ ...acc, [cache.uuid]: cache.name }), {}
                );
                const newUuidMappingEntries = caches.reduce((acc, cache) =>
                    ({ ...acc, [cache.uuid]: uuidMapping[cache.uuid] ?? "" }), {}
                );

                setNameMapping(prev => ({ ...prev, ...newNameMappingEntries }));
                setUuidMapping(prev => ({ ...prev, ...newUuidMappingEntries }));
                console.log("[usercache] 更新后的nameMapping:", { ...nameMapping, ...newNameMappingEntries });
                console.log("[usercache] 更新后的uuidMapping:", { ...uuidMapping, ...newUuidMappingEntries });
            } catch (error) {
                console.error("读取usercache.json失败:", error);
            }
        };

        const fetchPlayerData = async () => {
            try {
                const playerData: string[] = await invoke(
                    "read_player_data",
                    { dirPath: `${worldPathState.path}/playerdata` }
                );

                console.log("读取到的playerData:", playerData);
                const newUuidMappingEntries = playerData.reduce((acc, uuid) =>
                    ({ ...acc, [uuid]: uuidMapping[uuid] ?? "" }), {}
                );

                setUuidMapping(prev => ({ ...prev, ...newUuidMappingEntries }));
                console.log("[playerdata] 更新后的uuidMapping:", { ...uuidMapping, ...newUuidMappingEntries });
            } catch (error) {
                console.error("读取playerdata失败:", error);
            }
        }

        if (worldPathState.isValid) {
            fetchCache();
            fetchPlayerData();
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
