import style from "../styles/remapProgress.module.css";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { useAppContext } from "../utils/context";
import { listen } from "@tauri-apps/api/event";

function RemapProgress() {
    const [progress, setProgress] = useState([0, 0]);
    const [tasks, setTasks] = useState<string[]>([]);
    const {
        worldPathState,
        nameMapping,
        uuidMapping,
    } = useAppContext();

    useEffect(() => {

        const unlistenSetTotal = listen<number>('set-total', (event) => {
            console.log("Total tasks to process:", event.payload);
            setProgress([0, event.payload]);
        });

        const unlistenStartTask = listen<string>('start-task', (event) => {
            console.log("Starting task:", event.payload);
            setTasks(prev => [...prev, event.payload]);
        });

        const unlistenFinishTask = listen<string>('finish-task', (event) => {
            console.log("Finished task:", event.payload);
            setTasks(prev => prev.filter(task => task !== event.payload));
            setProgress(prev => [prev[0] + 1, prev[1]]);
        });

        return () => {
            unlistenSetTotal.then(f => f());
            unlistenStartTask.then(f => f());
            unlistenFinishTask.then(f => f());
        };
    }, []);

    return (
        <div className={style.container}>
            <button className="btn" onClick={() => {
                console.log(nameMapping);
                console.log(uuidMapping);
            }}>debug</button>
            <button className="btn" onClick={async () => {
                await invoke("process_world", {
                    worldPath: worldPathState.path,
                    uuidMap: Object.fromEntries(uuidMapping),
                });

            }}>开始转换</button>
            <div className={style.prog}>
                <label>{progress[0]} / {progress[1]}</label>
                <progress className="progress" value={progress[0]} max={progress[1]} />
                {tasks.length > 0 && 
                <div>
                    <h4>正在处理:</h4>
                    <ul>
                        {tasks.map((task, index) => (
                            <li key={index}>{task}</li>
                        ))}
                    </ul>
                </div>}
            </div>
        </div>
    )
}

export default RemapProgress;
