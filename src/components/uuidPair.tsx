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
    const info = nameMapping[uuid];


    return (
        <div className="flex flex-col gap-1 w-full" >
            {
                info?.avatar ? (
                    <div className="flex flex-row items-center gap-2 h-10">
                        <img className={style.avatar} src={info.avatar} alt="Avatar" />
                        <span>{info.name}{info.mode === "NotMatch" && "?"}</span>
                    </div>
                ) : (
                    <div className="h-10 w-full" />
                )
            }
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
        <div className="flex flex-row items-end border-base-300 border gap-2 p-2 rounded-xl">
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
    const {
        uuidMapping,
        setUuidMapping,
    } = useAppContext();

    return (
        <div className="h-screen overflow-y-auto px-16 py-4 pb-18">
            <div className="h-full flex flex-col overflow-y-auto overflow-y-scroll pt-2 p-4 border border-base-300 bg-base-100 rounded-xl shadow-sm gap-2">
                <div className="px-2 pt-2">
                    <UuidTool />
                </div>
                <div className="flex flex-col gap-2">
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
