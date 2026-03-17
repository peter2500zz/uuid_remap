import { useState } from "react";
import style from "./uuidTool.module.css";
import { normalizeUUID, playerNameToOfflineUUID } from "../uuidUtils";
import { cachePlayerName, getUuidByName } from "../getAvatar";
import { useAppContext } from "../context";

function UuidTool() {
    const [onlineUuid, setOnlineUuid] = useState("");
    const [offlineUuid, setOfflineUuid] = useState("");
    const [playerName, setPlayerName] = useState("");
    const { 
        nameMapping,
        setNameMapping,
    } = useAppContext();

    const handlePlayerNameChange = async (name: string) => {
        setPlayerName(name);
        if (!name) {
            setOnlineUuid("");
            setOfflineUuid("");
            return;
        }
        console.log(`fetching UUIDs for player name: ${name}`);

        setOfflineUuid(playerNameToOfflineUUID(name));
        setOnlineUuid("正在查询...");
        setOnlineUuid(normalizeUUID(await getUuidByName(name)) || "不存在");
        cachePlayerName(name, null, setNameMapping);
    };

    return (
        <div className={style.container}>
            <span>UUID 计算器</span>
            <div>
                <label>玩家名称</label>
                <input
                    type="text"
                    placeholder="输入玩家名称"
                    value={playerName}
                    onChange={e => handlePlayerNameChange(e.target.value)}
                />
            </div>
            <div>
                <label>在线UUID</label>
                {nameMapping[onlineUuid]?.avatar && <img src={nameMapping[onlineUuid].avatar} alt="Avatar" />}
                <input
                    type="text"
                    placeholder="00000000-0000-0000-0000-000000000000"
                    value={onlineUuid}
                    readOnly
                />
            </div>
            <div>
                <label>离线UUID</label>
                {nameMapping[offlineUuid]?.avatar && <img src={nameMapping[offlineUuid].avatar} alt="Avatar" />}
                <input
                    type="text"
                    placeholder="00000000-0000-0000-0000-000000000000"
                    value={offlineUuid}
                    readOnly
                />
            </div>
        </div>
    )
}

export default UuidTool;
