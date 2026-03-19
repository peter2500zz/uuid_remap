import { useEffect, useRef, useState } from "react";
import style from "../styles/uuidTool.module.css";
import { normalizeUUID, playerNameToOfflineUUID } from "../utils/uuidUtils";
import { cachePlayerName, getUuidByName } from "../utils/getAvatar";
import { useAppContext } from "../utils/context";

function UuidTool() {
    const [onlineUuid, setOnlineUuid] = useState("");
    const [offlineUuid, setOfflineUuid] = useState("");
    const [playerName, setPlayerName] = useState("");
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
        setOnlineUuid(normalizeUUID(await getUuidByName(playerName)) ?? "不存在");
        cachePlayerName(playerName, null, setNameMapping);
    };

    const onlineAvatar = nameMapping[onlineUuid]?.avatar;
    const offlineAvatar = nameMapping[offlineUuid]?.avatar;

    return (
        <div className={style.container}>
            <div className={style.header}>UUID 计算器</div>

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
                <div className={style.fieldGroup}>
                    <div className={style.avatarSlot}>
                        {onlineAvatar
                            ? <img className={style.avatar} src={onlineAvatar} alt="Online UUID Avatar" />
                            : <div className={`skeleton ${style.avatarSkeleton}`} aria-hidden="true" />}
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
                <div className={style.fieldGroup}>
                    <div className={style.avatarSlot}>
                        {offlineAvatar
                            ? <img className={style.avatar} src={offlineAvatar} alt="Offline UUID Avatar" />
                            : <div className={`skeleton ${style.avatarSkeleton}`} aria-hidden="true" />}
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
    )
}

export default UuidTool;
