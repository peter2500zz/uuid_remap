import { invoke } from "@tauri-apps/api/core";
import style from "./folderSelect.module.css";
import { open } from '@tauri-apps/plugin-dialog';
import { useEffect, useState } from "react";

function FolderSelect(
    {
        worldPathState,
        setWorldPathState
    }: {
        worldPathState: {
            path: string;
            isValid: boolean
        },
        setWorldPathState: (path: string, isValid: boolean) => void
    }
) {
    const [display, setDisplay] = useState(true);

    useEffect(() => {
        const fetchCache = async () => {
            try {
                const caches = await invoke(
                    "read_cache",
                    { filePath: `${worldPathState.path}/../usercache.json` }
                );

                console.log("读取到的caches:", caches);
            } catch (error) {
                console.error("读取usercache.json失败:", error);
            }
        };

        if (worldPathState.isValid) {
            fetchCache();
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

                        setWorldPathState(newPath, await invoke("check_dir", { dirPath: newPath }));
                    }} />
                    <button onClick={async () => {
                        // 点按钮的时候也检查
                        const selected = await open({ multiple: false, directory: true });
                        if (selected) {

                            setWorldPathState(selected as string, await invoke("check_dir", { dirPath: selected }));
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
