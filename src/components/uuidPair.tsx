import { useAppContext } from "../utils/context";
import { isValidUUID, isUuidDuplicated, isMappingReady, normalizeUUID, UuidPair } from "../utils/uuidUtils";
import { open, save } from '@tauri-apps/plugin-dialog';
import { cachePlayerName } from "../utils/getAvatar";
import { fetch } from '@tauri-apps/plugin-http';
import UuidTool from "./uuidTool";
import { useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import toast from "react-hot-toast";

function uuidErrorMessage(uuid: string, isDuplicated: boolean): string | null {
    if (!uuid) return "UUID 不能为空";
    if (!isValidUUID(uuid)) return "无效的 UUID 格式";
    if (isDuplicated) return "UUID 重复出现";
    return null;
}

function AvatarAndInput({ showAvatar, uuid, onChange }: {
    showAvatar: boolean;
    uuid: string;
    onChange: (newUuid: string) => void;
}) {
    const { uuidPairs, playerInfoMap } = useAppContext();
    const info = playerInfoMap[uuid];

    const isDuplicated = useMemo(
        () => isUuidDuplicated(uuidPairs, uuid),
        [uuidPairs, uuid]
    );
    const error = uuidErrorMessage(uuid, isDuplicated);

    return (
        <div className="flex flex-col gap-1 w-full">
            {
                showAvatar && (info?.avatar ? (
                    <div className="flex flex-row items-center gap-2 h-10">
                        <img className="w-8 h-8 rounded-md" src={info.avatar} alt={`${info.name} 的头像`} />
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
            <div className={error ? "tooltip tooltip-top" : ""} data-tip={error ?? undefined}>
                <input
                    className={`input input-bordered w-full ${error ? "border-error" : ""}`}
                    placeholder="UUID v4"
                    value={uuid}
                    onChange={e => onChange(e.target.value)}
                />
            </div>
        </div>
    )
}

// 交换是双向的，左右两个 UUID 没有方向之分
function UuidPairRow({ index, pair }: { index: number; pair: UuidPair }) {
    const {
        setUuidPairs,
        playerInfoMap,
        setPlayerInfoMap,
    } = useAppContext();
    const [leftUuid, rightUuid] = pair;

    const showAvatarRow = !!(playerInfoMap[leftUuid]?.avatar || playerInfoMap[rightUuid]?.avatar);

    // 输入了一个尚未见过的有效 UUID 时，尝试拉取对应的在线玩家信息
    const fetchProfileIfNeeded = (uuid: string) => {
        if (!isValidUUID(uuid)) return;
        const normalized = normalizeUUID(uuid);
        if (!normalized || playerInfoMap[normalized]) return;

        console.log(`Find new valid UUID: ${normalized}, fetching player name and avatar...`);
        fetch(`https://sessionserver.mojang.com/session/minecraft/profile/${normalized}`)
            .then(res => res.ok ? res.json().catch(() => console.warn(`Not a online player: ${normalized}`)) : null)
            .then(data => {
                if (!data) return;
                cachePlayerName(data.name, null, setPlayerInfoMap);
            });
    };

    const changeSide = (side: 0 | 1, uuid: string) => {
        fetchProfileIfNeeded(uuid);

        const normalized = normalizeUUID(uuid) ?? uuid;
        setUuidPairs(prev => prev.map((p, i) => {
            if (i !== index) return p;
            const next: UuidPair = [...p];
            next[side] = normalized;
            return next;
        }));
    };

    return (
        <div className="flex flex-row items-end border-base-300 border gap-2 p-2 rounded-xl">
            <AvatarAndInput showAvatar={showAvatarRow} uuid={leftUuid} onChange={uuid => changeSide(0, uuid)} />
            <div className="tooltip tooltip-top h-10 flex items-center px-1" data-tip="两个 UUID 将互相交换">
                <span className="font-bold text-base-content/60 select-none">↔</span>
            </div>
            <AvatarAndInput showAvatar={showAvatarRow} uuid={rightUuid} onChange={uuid => changeSide(1, uuid)} />
            <button className="btn btn-outline btn-error" onClick={() => setUuidPairs(prev => prev.filter((_, i) => i !== index))}>
                删除
            </button>
        </div>
    )
}

function UuidPairs() {
    const {
        uuidPairs,
        setUuidPairs,
        playerInfoMap,
    } = useAppContext();

    const handleImport = async () => {
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
        if (!selected) return;

        try {
            const uuidMap = await invoke<Record<string, string>>("import_uuid_map", { path: selected });
            setUuidPairs(prev => [...prev, ...Object.entries(uuidMap)]);
            toast.success("导入成功");
        } catch (e) {
            toast.error(`导入失败: ${(e as Error).message || String(e)}`);
        }
    };

    const handleExport = async () => {
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
        if (!selected) return;

        try {
            await invoke("export_uuid_map", { uuidMap: Object.fromEntries(uuidPairs), nameMap: playerInfoMap, path: selected });
            toast.success("导出成功");
        } catch (e) {
            toast.error(`导出失败: ${(e as Error).message || String(e)}`);
        }
    };

    return (
        <div className="h-screen overflow-y-auto px-16 py-4 pb-18">
            <div className="h-full flex flex-col overflow-y-auto pt-2 p-4 border border-base-300 bg-base-100 rounded-xl shadow-sm gap-2">
                <div className="pt-2">
                    <UuidTool />
                </div>
                <div className="flex flex-col gap-2">
                    {
                        uuidPairs.map((pair, index) => (
                            <UuidPairRow key={index} index={index} pair={pair} />
                        ))
                    }
                </div>
                <div className="flex flex-row gap-2 px-2">
                    <div className="tooltip tooltip-right" data-tip="从一个 JSON 文件中导入 UUID 交换表">
                        <button className="btn flex-none" onClick={handleImport}>
                            导入
                        </button>
                    </div>
                    <button className="btn flex-1" onClick={() => setUuidPairs(prev => [...prev, ["", ""]])}>
                        +
                    </button>
                    <div className="tooltip tooltip-left" data-tip="导出 UUID 交换表，可以给 CLI 版本使用">
                        <button
                            className={`btn flex-none ${isMappingReady(uuidPairs) ? "btn-primary" : ""}`}
                            disabled={!isMappingReady(uuidPairs)}
                            onClick={handleExport}
                        >
                            导出
                        </button>
                    </div>
                </div>
            </div>
        </div>
    )
}

export default UuidPairs;
