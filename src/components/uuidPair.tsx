import { useEffect, useState } from "react";
import style from "./uuidPair.module.css";
import { useAppContext } from "../context";
import { isValidUUID, normalizeUUID } from "../uuidUtils";
import { getPlayerAvatar } from "../getAvatar";
import { fetch } from '@tauri-apps/plugin-http';
import UuidTool from "./uuidTool";

function UuidPair({ index, oldUuid, newUuid }: {
    index: number;
    oldUuid: string;
    newUuid: string;
}) {
    const {
        setUuidMapping,
        nameMapping,
        setNameMapping,
    } = useAppContext();

    // 给 input 提供用于修改的函数，因为左值也可以修改所以用索引定位了
    const changeUuid = (index: number, uuid: string, side: "Left" | "Right") => {
        const normalized = normalizeUUID(uuid);

        if (isValidUUID(uuid) && normalized && !nameMapping[normalized]) {
            console.log(`Find new valid UUID: ${normalized}, fetching player name and avatar...`);

            fetch(`https://sessionserver.mojang.com/session/minecraft/profile/${normalized}`)
                .then(res => res.ok ? res.json() : null)
                .then(async data => {
                    if (!data) return;
                    const avatar = await getPlayerAvatar(normalized);
                    if (!avatar) return;
                    setNameMapping(prev => ({
                        ...prev,
                        [normalized]: { name: data.name, avatar, mode: "Online" }
                    }));
                });
        }

        setUuidMapping(prev => prev.map(([k, v], i) =>
            i === index
                ? side === "Left" ? [normalized || uuid, v] : [k, normalized || uuid]
                : [k, v]
        ));
    };

    const swapUuid = (index: number) => {
        setUuidMapping(prev => prev.map(([k, v], i) =>
            i === index ? [v, k] : [k, v]
        ));
    };

    return (
        <div>
            {nameMapping[oldUuid]?.avatar && <img src={nameMapping[oldUuid].avatar} alt="Avatar" />}
            <input className={!isValidUUID(oldUuid) ? style.invalidInput : ""} value={oldUuid} onChange={e => changeUuid(index, e.target.value, "Left")} />
            <button onClick={() => swapUuid(index)}>↔</button>
            {nameMapping[newUuid]?.avatar && <img src={nameMapping[newUuid].avatar} alt="Avatar" />}
            <input className={!isValidUUID(newUuid) ? style.invalidInput : ""} value={newUuid} onChange={e => changeUuid(index, e.target.value, "Right")} />
            <button onClick={() => setUuidMapping(prev => prev.filter((_, i) => i !== index))}>
                删除
            </button>
        </div>
    )
}

function UuidPairs() {
    const [display, setDisplay] = useState(false);
    const {
        worldPathState,
        uuidMapping,
        setUuidMapping,
    } = useAppContext();

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
                    <div>
                        <UuidTool />
                    </div>
                    {uuidMapping.map(([oldUuid, newUuid], index) => (
                        <UuidPair key={index} index={index} oldUuid={oldUuid} newUuid={newUuid} />
                    ))}
                    <button onClick={() => setUuidMapping(prev => [...prev, ["", ""]])}>
                        +
                    </button>
                </div>
            )}
        </div>
    )
}

export default UuidPairs;
