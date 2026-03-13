import { useEffect, useState } from "react";
import style from "./uuidPair.module.css";
import { invoke } from "@tauri-apps/api/core";

function UuidPair() {

    return (
        <div>
            <input placeholder="原UUID" />
            <button>→</button>
            <input placeholder="新UUID" />
        </div>
    )
}

function UuidPairs({ worldPath }: { worldPath: string }) {
    const [display, setDisplay] = useState(false);

    useEffect(() => {
        setDisplay(!!worldPath);

    }, [worldPath]);

    return (
        <div className={style.container}>
            <button onClick={() => setDisplay(!display)} disabled={!worldPath}>
                设定UUID转换规则
            </button>

            {display && (
                <div>
                    <UuidPair />
                    <UuidPair />
                    <UuidPair />
                </div>
            )}
        </div>
    )
}

export default UuidPairs;
