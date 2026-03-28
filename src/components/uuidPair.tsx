import { useAppContext } from "../utils/context";
import { isValidUUID, normalizeUUID } from "../utils/uuidUtils";
import { open, save } from '@tauri-apps/plugin-dialog';
import { cachePlayerName } from "../utils/getAvatar";
import { fetch } from '@tauri-apps/plugin-http';
import UuidTool from "./uuidTool";
import { useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import toast from "react-hot-toast";


// 检测某个特定字符串是否出现超过一次
function isStringDuplicated(uuidMapping: [string, string][], target: string): boolean {
    let count = 0;
    for (const [a, b] of uuidMapping) {
        if (a === target) count++;
        if (b === target) count++;
        if (count > 1) return true;
    }
    return false;
}

// 检测整个 mapping 中是否存在任意重复字符串
export function hasDuplicates(uuidMapping: [string, string][]): boolean {
    const seen = new Set<string>();
    for (const [a, b] of uuidMapping) {
        for (const s of [a, b]) {
            if (seen.has(s)) return true;
            seen.add(s);
        }
    }
    return false;
}

// 检测整个 mapping 中是否存在无效UUID
export function hasInvalidUUID(uuidMapping: [string, string][]): boolean {
    for (const [a, b] of uuidMapping) {
        for (const s of [a, b]) {
            if (!isValidUUID(s)) return true;
        }
    }
    return false;
}

function AvaterAndInput({ showAvatar, uuid, onChange }: { showAvatar: boolean, uuid: string, onChange: (newUuid: string) => void }) {
    const {
        uuidMapping,
        nameMapping,
    } = useAppContext();
    const info = nameMapping[uuid];

    const isDuplicated = useMemo(
        () => isStringDuplicated(uuidMapping, uuid),
        [uuidMapping, uuid]
    );

    return (
        <div className="flex flex-col gap-1 w-full" >

            {
                showAvatar && (info?.avatar ? (
                    <div className="flex flex-row items-center gap-2 h-10">
                        <img className="w-8 h-8 rounded-md" src={info.avatar} alt="Avatar" />
                        <span>{info.name}
                            {
                                info.mode === "NotMatch" &&
                                <div className="tooltip tooltip-top" data-tip="与名字不匹配的 UUID">
                                    <span className="pl-1 underline font-bold">?</span>
                                </div>
                            }
                        </span>


                    </div>
                ) : (
                    <div className="h-10 w-full" />
                ))
            }
            <div className="tooltip tooltip-top" data-tip={
                (() => {
                    if (!uuid) return "UUID 不能为空";
                    if (!isValidUUID(uuid)) return "无效的 UUID 格式";
                    if (isDuplicated) return "UUID 重复出现";
                    return "";
                })()
            }>
                <input
                    className={`input input-bordered w-full ${(!isValidUUID(uuid) || isDuplicated) ? "border-error" : ""}`}
                    placeholder="UUID v4"
                    value={uuid}
                    onChange={e => onChange(e.target.value)}
                />
            </div>
        </div>
    )
}

function UuidPair({ index, oldUuid: leftUuid, newUuid: rightUuid }: {
    index: number;
    oldUuid: string;
    newUuid: string;
}) {
    const {
        setUuidMapping,
        nameMapping,
        setNameMapping,
    } = useAppContext();

    const showAvatarRow = !!(nameMapping[leftUuid]?.avatar || nameMapping[rightUuid]?.avatar);

    // 给 input 提供用于修改的函数，因为左值也可以修改所以用索引定位了
    const changeUuid = (index: number, uuid: string, side: "Left" | "Right") => {
        const normalized = normalizeUUID(uuid);

        if (isValidUUID(uuid) && normalized && !nameMapping[normalized]) {
            console.log(`Find new valid UUID: ${normalized}, fetching player name and avatar...`);

            fetch(`https://sessionserver.mojang.com/session/minecraft/profile/${normalized}`)
                .then(res => res.ok ? res.json().catch(() => console.warn(`Not a online player: ${normalized}`)) : null)
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
            <AvaterAndInput showAvatar={showAvatarRow} uuid={leftUuid} onChange={uuid => changeUuid(index, uuid, "Left")} />
            <button className="btn" onClick={() => swapUuid(index)}>↔</button>
            <AvaterAndInput showAvatar={showAvatarRow} uuid={rightUuid} onChange={uuid => changeUuid(index, uuid, "Right")} />
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
        nameMapping,
    } = useAppContext();

    return (
        <div className="h-screen overflow-y-auto px-16 py-4 pb-18">
            <div className="h-full flex flex-col overflow-y-auto overflow-y-scroll pt-2 p-4 border border-base-300 bg-base-100 rounded-xl shadow-sm gap-2">
                <div className="pt-2">
                    <UuidTool />
                </div>
                <div className="flex flex-col gap-2">
                    {
                        uuidMapping.map(([oldUuid, newUuid], index) => (
                            <UuidPair key={index} index={index} oldUuid={oldUuid} newUuid={newUuid} />
                        ))
                    }
                </div>
                <div className="flex flex-row gap-2 px-2">
                    <div className="tooltip tooltip-right" data-tip="从一个 JSON 文件中导入 UUID 映射">
                        <button
                            className={`btn flex-none`} onClick={async () => {
                                const selected = await open({
                                    multiple: false,
                                    directory: false,
                                    filters: [{
                                        name: 'JSON 文件',
                                        extensions: ['json', 'jsonc']
                                    }, {
                                        name: '所有文件',
                                        extensions: ['*']
                                    }]
                                });

                                if (selected) {
                                    try {
                                        const uuidMap = await invoke<Record<string, string>>("import_uuid_map", { path: selected });
                                        setUuidMapping(prev => [...prev, ...Object.entries(uuidMap)]);
                                        toast.success("导入成功");
                                    } catch (e) {
                                        toast.error(`导入失败: ${(e as Error).message || String(e)}`);
                                    }
                                }
                            }}>
                            导入
                        </button>
                    </div>
                    <button className="btn flex-1" onClick={() => {
                        setUuidMapping(prev => [...prev, ["", ""]])
                    }}>
                        +
                    </button>
                    <div className="tooltip tooltip-left" data-tip="导出 UUID 映射，可以给 CLI 版本使用">
                        <button
                            className={`btn flex-none ${!hasDuplicates(uuidMapping) && !hasInvalidUUID(uuidMapping) && uuidMapping.length > 0 ? "btn-primary" : "btn-disabled"}`} onClick={async () => {
                                const selected = await save({
                                    filters: [{
                                        name: 'JSONC 文件',
                                        extensions: ['jsonc']
                                    }, {
                                        name: '所有文件',
                                        extensions: ['*']
                                    }],
                                    defaultPath: 'uuid_map.jsonc'
                                });

                                if (selected) {
                                    try {
                                        await invoke("export_uuid_map", { uuidMap: Object.fromEntries(uuidMapping), nameMap: nameMapping, path: selected });
                                        toast.success("导出成功");
                                    } catch (e) {
                                        toast.error(`导出失败: ${(e as Error).message || String(e)}`);
                                    }
                                }
                            }}>
                            导出
                        </button>
                    </div>
                </div>
            </div>
        </div>
    )
}

export default UuidPairs;
