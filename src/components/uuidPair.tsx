import { useEffect, useState } from "react";
import style from "../styles/uuidPair.module.css";
import { useAppContext } from "../utils/context";
import { isValidUUID, normalizeUUID } from "../utils/uuidUtils";
import { cachePlayerName } from "../utils/getAvatar";
import { fetch } from '@tauri-apps/plugin-http';
import UuidTool from "./uuidTool";


function AvaterAndInput({ uuid, onChange }: { uuid: string, onChange: (newUuid: string) => void }) {
    const {
        nameMapping,
    } = useAppContext();


    return (
        <div className={style.inputWithAvatar}>
            {nameMapping[uuid]?.avatar && <div>
                <img className={style.avatar} src={nameMapping[uuid].avatar} alt="Avatar" />
                <span>{nameMapping[uuid].name}{nameMapping[uuid].mode === "NotMatch" && "?"}</span>
            </div>}
            <input
                className={`input input-bordered w-full ${!isValidUUID(uuid) ? style.invalidInput : ""}`}
                placeholder="原UUID"
                value={uuid}
                onChange={e => onChange(e.target.value)}
            />
        </div>
    )
}

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
                    cachePlayerName(data.name, null, setNameMapping)
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
        <div className={style.pairRow}>
            <AvaterAndInput uuid={oldUuid} onChange={uuid => changeUuid(index, uuid, "Left")} />
            <button className="btn btn-outline" onClick={() => swapUuid(index)}>↔</button>
            <AvaterAndInput uuid={newUuid} onChange={uuid => changeUuid(index, uuid, "Right")} />
            <button className="btn btn-outline btn-error" onClick={() => setUuidMapping(prev => prev.filter((_, i) => i !== index))}>
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
        if (!worldPathState.isValid) {
            setDisplay(false);
        }
    }, [worldPathState.isValid]);

    return (
        <div className={style.container}>
            <button
                className="btn btn-secondary"
                onClick={() => setDisplay(!display)}
                disabled={!worldPathState.isValid}
            >
                设定UUID转换规则
            </button>

            <div className={`${style.panel} ${!display ? style.hiddenPanel : ""}`}>
                <div className={style.toolWrap}>
                    <UuidTool />
                </div>
                <div className={style.rows}>
                    {uuidMapping.map(([oldUuid, newUuid], index) => (
                        <UuidPair key={index} index={index} oldUuid={oldUuid} newUuid={newUuid} />
                    ))}
                </div>
                <button className="btn btn-outline" onClick={() => setUuidMapping(prev => [...prev, ["", ""]])}>
                    +
                </button>
            </div>
        </div>
    )
}

export default UuidPairs;
