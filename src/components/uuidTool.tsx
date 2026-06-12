import { useEffect, useState } from "react";
import { normalizeUUID, playerNameToOfflineUUID } from "../utils/uuidUtils";
import { cachePlayerName, getUuidByName } from "../utils/getAvatar";
import { useAppContext } from "../utils/context";
import toast from "react-hot-toast";

function UuidTool() {
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

    const handleCalculate = async () => {
        if (!playerName) {
            setOnlineUuid("");
            setOfflineUuid("");
            setQueried(false);
            return;
        }
        console.log(`fetching UUIDs for player name: ${playerName}`);

        setOfflineUuid(playerNameToOfflineUUID(playerName));
        setOnlineUuid("");
        setIsQuerying(true);
        setOnlineUuid(normalizeUUID(await getUuidByName(playerName)) ?? "");
        await cachePlayerName(playerName, null, setPlayerInfoMap);
        setIsQuerying(false);
        setQueried(true);
    };

    const copyToClipboard = (uuid: string) => {
        navigator.clipboard.writeText(uuid);
        toast.success("已复制");
    };

    const onlineAvatar = playerInfoMap[onlineUuid]?.avatar;
    const offlineAvatar = playerInfoMap[offlineUuid]?.avatar;

    return (
        <div className="collapse collapse-arrow bg-base-100 border-base-300 border">
            <input type="checkbox" />
            <div className="collapse-title font-semibold after:start-5 after:end-auto pe-4 ps-12">UUID 计算器</div>

            <div className="collapse-content text-sm flex flex-col gap-2">

                <label className="label" htmlFor="uuid-tool-player-name">玩家名称</label>
                <div className="flex flex-row gap-2">
                    <input
                        id="uuid-tool-player-name"
                        className="input input-bordered w-full flex-6"
                        type="text"
                        placeholder="输入玩家名称"
                        value={playerName}
                        onChange={e => setPlayerName(e.target.value)}
                        onKeyDown={e => e.key === "Enter" && handleCalculate()}
                    />
                    <button className="btn flex-4" onClick={handleCalculate}>计算</button>
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
