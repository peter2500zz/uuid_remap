import { useEffect, useRef, useState } from "react";
import { normalizeUUID, playerNameToOfflineUUID } from "../utils/uuidUtils";
import { cachePlayerName, getUuidByName } from "../utils/getAvatar";
import { useAppContext } from "../utils/context";
import toast from "react-hot-toast";
import { useI18n } from "../i18n/context";

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
    const rootRef = useRef<HTMLDivElement>(null);
    const {
        worldPathState,
        playerInfoMap,
        setPlayerInfoMap,
    } = useAppContext();
    const { t } = useI18n();

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

    // 展开时点击计算器以外的区域（如背后的列表）自动收起，点击本身照常生效
    useEffect(() => {
        if (!isOpen) return;

        const onPointerDown = (event: PointerEvent) => {
            if (rootRef.current && !rootRef.current.contains(event.target as Node)) {
                setIsOpen(false);
            }
        };
        document.addEventListener("pointerdown", onPointerDown);
        return () => document.removeEventListener("pointerdown", onPointerDown);
    }, [isOpen]);

    const copyToClipboard = (uuid: string) => {
        navigator.clipboard.writeText(uuid);
        toast.success(t("uuidTool.copied"));
        // 复制即拿到了结果，自动收起计算器
        setIsOpen(false);
    };

    const onlineAvatar = playerInfoMap[onlineUuid]?.avatar;
    const offlineAvatar = playerInfoMap[offlineUuid]?.avatar;

    return (
        <div ref={rootRef} className="relative flex-none border-b border-base-300">
            <button
                className="w-full flex items-center gap-2 px-4 h-12 font-semibold text-left transition-colors hover:bg-base-200"
                aria-expanded={isOpen}
                onClick={() => setIsOpen(prev => !prev)}
            >
                <span className={`text-xs transition-transform duration-200 ${isOpen ? "rotate-90" : ""}`}>▶</span>
                {t("uuidTool.title")}
            </button>

            {/* 裁剪容器：面板在其中整体滑入滑出，底部留白用于显示阴影和圆角 */}
            <div className="absolute top-full inset-x-0 z-20 overflow-hidden pb-6 pointer-events-none">
                {/* 关闭时 invisible：既参与过渡动画，又不可见、不可聚焦；
                    位移在 -100% 基础上再多移出 1.5rem，让向下投射的阴影也随滑动连续移出，
                    避免过渡结束 visibility 翻转时残留的阴影带突然消失 */}
                <div className={`
                    pointer-events-auto bg-base-100 border border-t-0 border-base-300 rounded-b-xl shadow-lg p-4 text-sm flex flex-col gap-2
                    transition-all duration-300 ease-out
                    ${isOpen ? "visible translate-y-0" : "invisible -translate-y-[calc(100%+1.5rem)]"}
                `}>
                    <label className="label" htmlFor="uuid-tool-player-name">{t("uuidTool.playerName")}</label>
                    <div className="flex flex-row gap-2">
                        <input
                            id="uuid-tool-player-name"
                            className="input input-bordered w-full flex-6"
                            type="text"
                            placeholder={t("uuidTool.playerNamePlaceholder")}
                            value={playerName}
                            onChange={e => setPlayerName(e.target.value)}
                            onKeyDown={e => e.key === "Enter" && calculate(playerName)}
                        />
                        <button className="btn flex-4" onClick={() => calculate(playerName)}>{t("uuidTool.calculate")}</button>
                    </div>

                    <label className="label" htmlFor="uuid-tool-online-uuid">{t("uuidTool.onlineUuid")}</label>
                    <div className="flex gap-2 items-center">
                        <div className="flex-shrink-0">
                            {onlineAvatar
                                ? <img className="w-8 h-8 rounded-md" src={onlineAvatar} alt={t("uuidTool.onlineAvatarAlt")} />
                                : <div className={`skeleton w-8 h-8 rounded-md ${isQuerying ? "" : "animate-none"}`} aria-hidden="true" />}
                        </div>
                        <input
                            id="uuid-tool-online-uuid"
                            className="input input-bordered w-full"
                            type="text"
                            placeholder={isQuerying ? t("uuidTool.querying") : queried && !onlineUuid ? t("uuidTool.noOnlinePlayer") : "UUID v4"}
                            value={onlineUuid}
                            readOnly
                        />
                        <button className="btn" disabled={!onlineUuid} onClick={() => copyToClipboard(onlineUuid)}>{t("uuidTool.copy")}</button>
                    </div>

                    <label className="label" htmlFor="uuid-tool-offline-uuid">{t("uuidTool.offlineUuid")}</label>
                    <div className="flex gap-2 items-center">
                        <div className="flex-shrink-0">
                            {offlineAvatar
                                ? <img className="w-8 h-8 rounded-md" src={offlineAvatar} alt={t("uuidTool.offlineAvatarAlt")} />
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
                        <button className="btn" disabled={!offlineUuid} onClick={() => copyToClipboard(offlineUuid)}>{t("uuidTool.copy")}</button>
                    </div>
                </div>
            </div>
        </div>
    )
}

export default UuidTool;
export type { CalculatorRequest };
