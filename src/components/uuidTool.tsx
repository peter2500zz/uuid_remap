import { useEffect, useState } from "react";
import { normalizeUUID, playerNameToOfflineUUID } from "../utils/uuidUtils";
import { cachePlayerName, getUuidByName } from "../utils/getAvatar";
import { useAppContext } from "../utils/context";
import toast from "react-hot-toast";

/// 来自外部（如交换列表）的计算请求，seq 用于区分对同一名字的重复请求
interface CalculatorRequest {
    name: string;
    seq: number;
}

// 贴在卡片顶边的标题栏 + 覆盖在列表上方的下拉面板，
// 展开/收起不改变列表的布局和滚动位置
function UuidTool({ calcRequest }: { calcRequest?: CalculatorRequest | null }) {
    const [isOpen, setIsOpen] = useState(false);
    const [onlineUuid, setOnlineUuid] = useState("");
    const [offlineUuid, setOfflineUuid] = useState("");
    const [playerName, setPlayerName] = useState("");
    const [isQuerying, setIsQuerying] = useState(false);
    const [queried, setQueried] = useState(false);
    const {
        worldPathState,
        playerInfoMap,
        setPlayerInfoMap,
    } = useAppContext();

    // 换了存档之后清空计算结果
    useEffect(() => {
        setPlayerName("");
        setOnlineUuid("");
        setOfflineUuid("");
        setQueried(false);
    }, [worldPathState.path]);

    const calculate = async (name: string) => {
        if (!name) {
            setOnlineUuid("");
            setOfflineUuid("");
            setQueried(false);
            return;
        }
        console.log(`fetching UUIDs for player name: ${name}`);

        setOfflineUuid(playerNameToOfflineUUID(name));
        setOnlineUuid("");
        setIsQuerying(true);
        setOnlineUuid(normalizeUUID(await getUuidByName(name)) ?? "");
        await cachePlayerName(name, null, setPlayerInfoMap);
        setIsQuerying(false);
        setQueried(true);
    };

    // 外部请求：展开计算器，填入名字并执行一次计算
    useEffect(() => {
        if (!calcRequest) return;
        setIsOpen(true);
        setPlayerName(calcRequest.name);
        calculate(calcRequest.name);
    }, [calcRequest]);

    const copyToClipboard = (uuid: string) => {
        navigator.clipboard.writeText(uuid);
        toast.success("已复制");
        // 复制即拿到了结果，自动收起计算器
        setIsOpen(false);
    };

    const onlineAvatar = playerInfoMap[onlineUuid]?.avatar;
    const offlineAvatar = playerInfoMap[offlineUuid]?.avatar;

    return (
        <div className="relative flex-none border-b border-base-300">
            <button
                className="w-full flex items-center gap-2 px-4 h-12 font-semibold text-left transition-colors hover:bg-base-200"
                aria-expanded={isOpen}
                onClick={() => setIsOpen(prev => !prev)}
            >
                <span className={`text-xs transition-transform duration-200 ${isOpen ? "rotate-90" : ""}`}>▶</span>
                UUID 计算器
            </button>

            {/* 关闭时 invisible：既参与过渡动画，又不可见、不可聚焦 */}
            <div className={`
                absolute top-full inset-x-0 z-20 bg-base-100 border-b border-base-300 shadow-lg p-4 text-sm flex flex-col gap-2
                origin-top transition-all duration-200 ease-out
                ${isOpen ? "visible opacity-100 translate-y-0" : "invisible opacity-0 -translate-y-2"}
            `}>
                    <label className="label" htmlFor="uuid-tool-player-name">玩家名称</label>
                    <div className="flex flex-row gap-2">
                        <input
                            id="uuid-tool-player-name"
                            className="input input-bordered w-full flex-6"
                            type="text"
                            placeholder="输入玩家名称"
                            value={playerName}
                            onChange={e => setPlayerName(e.target.value)}
                            onKeyDown={e => e.key === "Enter" && calculate(playerName)}
                        />
                        <button className="btn flex-4" onClick={() => calculate(playerName)}>计算</button>
                    </div>

                    <label className="label" htmlFor="uuid-tool-online-uuid">在线UUID</label>
                    <div className="flex gap-2 items-center">
                        <div className="flex-shrink-0">
                            {onlineAvatar
                                ? <img className="w-8 h-8 rounded-md" src={onlineAvatar} alt="在线玩家头像" />
                                : <div className={`skeleton w-8 h-8 rounded-md ${isQuerying ? "" : "animate-none"}`} aria-hidden="true" />}
                        </div>
                        <input
                            id="uuid-tool-online-uuid"
                            className="input input-bordered w-full"
                            type="text"
                            placeholder={isQuerying ? "正在查询..." : queried && !onlineUuid ? "不存在该在线玩家" : "UUID v4"}
                            value={onlineUuid}
                            readOnly
                        />
                        <button className="btn" disabled={!onlineUuid} onClick={() => copyToClipboard(onlineUuid)}>复制</button>
                    </div>

                    <label className="label" htmlFor="uuid-tool-offline-uuid">离线UUID</label>
                    <div className="flex gap-2 items-center">
                        <div className="flex-shrink-0">
                            {offlineAvatar
                                ? <img className="w-8 h-8 rounded-md" src={offlineAvatar} alt="离线玩家头像" />
                                : <div className={`skeleton w-8 h-8 rounded-md ${isQuerying ? "" : "animate-none"}`} aria-hidden="true" />}
                        </div>
                        <input
                            id="uuid-tool-offline-uuid"
                            className="input input-bordered w-full"
                            type="text"
                            placeholder="UUID v4"
                            value={offlineUuid}
                            readOnly
                        />
                        <button className="btn" disabled={!offlineUuid} onClick={() => copyToClipboard(offlineUuid)}>复制</button>
                    </div>
                </div>
        </div>
    )
}

export default UuidTool;
export type { CalculatorRequest };
