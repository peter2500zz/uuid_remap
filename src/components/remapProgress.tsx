import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";
import { useAppContext } from "../utils/context";

function RemapProgress() {
    const [status, setStatus] = useState("idle"); // idle, processing, completed
    const {
        worldPathState,
        uuidMapping,
    } = useAppContext();

    return (
        <div>
            <button onClick={() => {
                console.log(uuidMapping);
            }}>debug</button>
            <button className="btn" onClick={async () => {
                setStatus("processing");

                await invoke("process_world", {
                    worldPath: worldPathState.path,
                    uuidMap: Object.fromEntries(uuidMapping),
                });

                setStatus("completed");
            }}>开始转换</button>
            <span>{ status }</span>
        </div>
    )
}

export default RemapProgress;
