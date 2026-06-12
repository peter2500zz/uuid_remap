import { useAppContext } from "../utils/context";
import { createUuidPair, isValidUUID, isUuidDuplicated, isMappingReady, normalizeUUID, UuidPair } from "../utils/uuidUtils";
import { open, save } from '@tauri-apps/plugin-dialog';
import { cachePlayerByUuid } from "../utils/getAvatar";
import UuidTool, { CalculatorRequest } from "./uuidTool";
import { useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import toast from "react-hot-toast";

function uuidErrorMessage(uuid: string, isDuplicated: boolean): string | null {
    if (!uuid) return "UUID 不能为空";
    if (!isValidUUID(uuid)) return "无效的 UUID 格式";
    if (isDuplicated) return "UUID 重复出现";
    return null;
}

function AvatarAndInput({ showAvatar, uuid, onChange, onSendToCalculator }: {
    showAvatar: boolean;
    uuid: string;
    onChange: (newUuid: string) => void;
    onSendToCalculator: (playerName: string) => void;
}) {
    const { uuidPairs, playerInfoMap } = useAppContext();
    const info = playerInfoMap[uuid];

    const isDuplicated = useMemo(
        () => isUuidDuplicated(uuidPairs, uuid),
        [uuidPairs, uuid]
    );
    const error = uuidErrorMessage(uuid, isDuplicated);

    return (
        <div className="flex flex-col w-full">
            {/* 常驻容器 + 高度过渡，头像出现/消失时行高平滑变化而不是突变 */}
            <div className={`overflow-hidden transition-[height] duration-300 ease-in-out ${showAvatar ? "h-11" : "h-0"}`}>
                {info?.mode === "Loading" ? (
                    <div className="flex flex-row items-center gap-2 h-10">
                        <div className="skeleton w-8 h-8 rounded-md" aria-hidden="true" />
                        <span className="text-sm text-base-content/50">查询中...</span>
                    </div>
                ) : info?.avatar ? (
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
                        <button
                            className="btn btn-xs btn-ghost border border-base-300"
                            onClick={() => onSendToCalculator(info.name)}
                        >
                            计算
                        </button>
                    </div>
                ) : null}
            </div>
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
function UuidPairRow({ pair, onSendToCalculator }: {
    pair: UuidPair;
    onSendToCalculator: (playerName: string) => void;
}) {
    const {
        setUuidPairs,
        playerInfoMap,
        setPlayerInfoMap,
    } = useAppContext();
    const { left: leftUuid, right: rightUuid } = pair;

    // 任意一侧有头像或正在查询时整行显示头像区，保证左右输入框对齐；
    // 删除按钮锚定在卡片顶部，因此各行高度不一致也不影响连续删除
    const sideHasInfo = (uuid: string) => {
        const info = playerInfoMap[uuid];
        return !!info && (!!info.avatar || info.mode === "Loading");
    };
    const showAvatarRow = sideHasInfo(leftUuid) || sideHasInfo(rightUuid);

    // 输入了一个尚未见过的有效 UUID 时，尝试拉取对应的在线玩家信息
    const fetchProfileIfNeeded = (uuid: string) => {
        if (!isValidUUID(uuid)) return;
        const normalized = normalizeUUID(uuid);
        if (!normalized || playerInfoMap[normalized]) return;

        cachePlayerByUuid(normalized, setPlayerInfoMap);
    };

    const changeSide = (side: "left" | "right", uuid: string) => {
        fetchProfileIfNeeded(uuid);

        const normalized = normalizeUUID(uuid) ?? uuid;
        setUuidPairs(prev => prev.map(p => {
            if (p.id !== pair.id) return p;
            return side === "left" ? { ...p, left: normalized } : { ...p, right: normalized };
        }));
    };

    return (
        <div className="relative flex flex-row items-end border-base-300 border gap-2 p-2 pr-12 rounded-xl transition-colors hover:border-base-content/20">
            <AvatarAndInput showAvatar={showAvatarRow} uuid={leftUuid} onChange={uuid => changeSide("left", uuid)} onSendToCalculator={onSendToCalculator} />
            <div className="tooltip tooltip-top h-10 flex items-center px-1" data-tip="两个 UUID 将互相交换">
                <span className="font-bold text-base-content/60 select-none">↔</span>
            </div>
            <AvatarAndInput showAvatar={showAvatarRow} uuid={rightUuid} onChange={uuid => changeSide("right", uuid)} onSendToCalculator={onSendToCalculator} />
            {/* 锚定在卡片右上角：删除后下一张卡片顶部正好补位，连续删除时点击位置不漂移 */}
            <button
                className="btn btn-sm btn-circle btn-ghost text-error absolute top-2 right-2"
                aria-label="删除这一对 UUID"
                onClick={() => setUuidPairs(prev => prev.filter(p => p.id !== pair.id))}
            >
                ✕
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

    const [calcRequest, setCalcRequest] = useState<CalculatorRequest | null>(null);
    const sendToCalculator = (playerName: string) =>
        setCalcRequest(prev => ({ name: playerName, seq: (prev?.seq ?? 0) + 1 }));

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
            setUuidPairs(prev => [...prev, ...Object.entries(uuidMap).map(([left, right]) => createUuidPair(left, right))]);
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
            await invoke("export_uuid_map", { uuidMap: Object.fromEntries(uuidPairs.map(p => [p.left, p.right])), nameMap: playerInfoMap, path: selected });
            toast.success("导出成功");
        } catch (e) {
            toast.error(`导出失败: ${(e as Error).message || String(e)}`);
        }
    };

    return (
        <div className="h-screen px-16 py-4 pb-18">
            <div className="h-full flex flex-col border border-base-300 bg-base-100 rounded-xl shadow-sm overflow-hidden">
                {/* 计算器贴合卡片顶边，展开的面板覆盖在列表上方，不影响列表布局与滚动 */}
                <UuidTool calcRequest={calcRequest} />
                <div className="flex-1 min-h-0 overflow-y-auto flex flex-col gap-2 p-4 pb-2">
                    {
                        uuidPairs.map(pair => (
                            <UuidPairRow key={pair.id} pair={pair} onSendToCalculator={sendToCalculator} />
                        ))
                    }
                </div>
                <div className="flex flex-row gap-2 px-4 pt-2 pb-4">
                    <div className="tooltip tooltip-right" data-tip="从一个 JSON 文件中导入 UUID 交换表">
                        <button className="btn flex-none" onClick={handleImport}>
                            导入
                        </button>
                    </div>
                    <button className="btn flex-1" onClick={() => setUuidPairs(prev => [...prev, createUuidPair()])}>
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
