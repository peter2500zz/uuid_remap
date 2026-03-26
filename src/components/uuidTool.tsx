import { useEffect, useRef, useState } from "react";
import style from "../styles/uuidTool.module.css";
import { normalizeUUID, playerNameToOfflineUUID } from "../utils/uuidUtils";
import { cachePlayerName, getUuidByName } from "../utils/getAvatar";
import { useAppContext } from "../utils/context";

function UuidTool() {
    const [onlineUuid, setOnlineUuid] = useState("");
    const [offlineUuid, setOfflineUuid] = useState("");
    const [playerName, setPlayerName] = useState("");
    const [ifFetchingAvatar, setIfFetchingAvatar] = useState(false);
    const {
        worldPathState,
        nameMapping,
        setNameMapping,
    } = useAppContext();
    const lastPathRef = useRef(worldPathState.path.trim());

    useEffect(() => {
        const currentPath = worldPathState.path.trim();
        if (currentPath !== lastPathRef.current) {
            setPlayerName("");
            setOnlineUuid("");
            setOfflineUuid("");
            lastPathRef.current = currentPath;
        }
    }, [worldPathState.path]);

    const handleCalculate = async () => {
        if (!playerName) {
            setOnlineUuid("");
            setOfflineUuid("");
            return;
        }
        console.log(`fetching UUIDs for player name: ${playerName}`);

        setOfflineUuid(playerNameToOfflineUUID(playerName));
        setOnlineUuid("正在查询...");
        setIfFetchingAvatar(true);
        setOnlineUuid(normalizeUUID(await getUuidByName(playerName)) ?? "不存在");
        cachePlayerName(playerName, null, setNameMapping);
        setIfFetchingAvatar(false);
    };

    const onlineAvatar = nameMapping[onlineUuid]?.avatar;
    const offlineAvatar = nameMapping[offlineUuid]?.avatar;

    return (
        <div className="collapse collapse-arrow bg-base-100 border-base-300 border">
            <input type="checkbox" />
            <div className="collapse-title font-semibold after:start-5 after:end-auto pe-4 ps-12">UUID 计算器</div>

            <div className="collapse-content text-sm">
                <div className={style.row}>
                    <label className={style.label}>玩家名称</label>
                    <div className={style.fieldGroup}>
                        <input
                            className="input input-bordered w-full"
                            type="text"
                            placeholder="输入玩家名称"
                            value={playerName}
                            onChange={e => setPlayerName(e.target.value)}
                            onKeyDown={e => e.key === "Enter" && handleCalculate()}
                        />
                        <button className="btn btn-outline" onClick={handleCalculate}>计算</button>
                    </div>
                </div>

                <div className={style.row}>
                    <label className={style.label}>在线UUID</label>
                    <div className={style.fieldGroup + " " + style.uuidFieldGroup}>
                        <div>
                            {onlineAvatar
                                ? <img className="w-8 h-8 rounded-md" src={onlineAvatar} alt="Online UUID Avatar" />
                                : <div className={`skeleton ${ifFetchingAvatar ? '' : 'animate-none'} ${style.avatarSkeleton}`} aria-hidden="true" />}
                        </div>
                        <input
                            className="input input-bordered w-full"
                            type="text"
                            placeholder="00000000-0000-0000-0000-000000000000"
                            value={onlineUuid}
                            readOnly
                        />
                        <button className="btn btn-outline" onClick={() => navigator.clipboard.writeText(onlineUuid)}>复制</button>
                    </div>
                </div>

                <div className={style.row}>
                    <label className={style.label}>离线UUID</label>
                    <div className={style.fieldGroup + " " + style.uuidFieldGroup}>
                        <div className={style.avatarSlot}>
                            {offlineAvatar
                                ? <img className="w-8 h-8 rounded-md" src={offlineAvatar} alt="Offline UUID Avatar" />
                                : <div className={`skeleton ${ifFetchingAvatar ? '' : 'animate-none'} ${style.avatarSkeleton}`} aria-hidden="true" />}
                        </div>
                        <input
                            className="input input-bordered w-full"
                            type="text"
                            placeholder="00000000-0000-0000-0000-000000000000"
                            value={offlineUuid}
                            readOnly
                        />
                        <button className="btn btn-outline" onClick={() => navigator.clipboard.writeText(offlineUuid)}>复制</button>
                    </div>
                </div>
            </div>
        </div>
    )
}

export default UuidTool;
