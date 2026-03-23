import style from "../styles/remapProgress.module.css";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { useAppContext } from "../utils/context";
import { listen } from "@tauri-apps/api/event";

function RemapProgress() {
    const [progress, setProgress] = useState([0, 0, "idle"]);
    const {
        worldPathState,
        uuidMapping,
    } = useAppContext();

    useEffect(() => {

        const unlisten = listen<[number, number, string]>('update-progress', (event) => {
            const [current, total, stage] = event.payload;
            setProgress([current, total, stage]);
        });

        return () => {
            unlisten.then(f => f());
        };
    }, []);

    return (
        <div className={style.container}>
            <button className="btn" onClick={() => {
                console.log(uuidMapping);
            }}>debug</button>
            <button className="btn" onClick={async () => {
                setProgress([0, 0, "prepare"]);

                await invoke("process_world", {
                    worldPath: worldPathState.path,
                    uuidMap: Object.fromEntries(uuidMapping),
                });

            }}>开始转换</button>
            <div className={style.prog}>
                <label>{progress[2]}</label>
                <progress className="progress" value={progress[0]} max={progress[1]} />
            </div>
        </div>
    )
}

export default RemapProgress;
