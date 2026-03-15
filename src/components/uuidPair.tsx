import { useEffect, useState } from "react";
import style from "./uuidPair.module.css";
import { invoke } from "@tauri-apps/api/core";
import { useAppContext } from "../context";

function UuidPair() {

    return (
        <div>
            <input placeholder="原UUID" />
            <button>→</button>
            <input placeholder="新UUID" />
        </div>
    )
}

function UuidPairs() {
    const [display, setDisplay] = useState(false);
    const { worldPathState } = useAppContext();


    useEffect(() => {
        setDisplay(!!worldPathState.path);

    }, [worldPathState]);

    return (
        <div className={style.container}>
            <button onClick={() => setDisplay(!display)} disabled={!worldPathState.path}>
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
