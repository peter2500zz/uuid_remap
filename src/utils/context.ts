import { createContext, useContext, Dispatch, SetStateAction } from "react";
import { UuidPair } from "./uuidUtils";

type WorldPathType = "Server" | "World" | "WorldButHasServer" | "Invalid" | "InvalidButForce" | "NotExist";

interface WorldPathState {
    path: string;
    type: WorldPathType;
}

type PlayerMode = "Online" | "Offline" | "NotMatch";

interface PlayerData {
    avatar: string | null;
    name: string;
    mode: PlayerMode;
}

/// 以 UUID 为键的玩家信息缓存（名字、头像等）
type PlayerInfoMap = Record<string, PlayerData>;

interface AppContextType {
    onProgressing: boolean;
    setOnProgressing: Dispatch<SetStateAction<boolean>>;
    worldPathState: WorldPathState;
    setWorldPathState: Dispatch<SetStateAction<WorldPathState>>;
    uuidPairs: UuidPair[];
    setUuidPairs: Dispatch<SetStateAction<UuidPair[]>>;
    playerInfoMap: PlayerInfoMap;
    setPlayerInfoMap: Dispatch<SetStateAction<PlayerInfoMap>>;
}

export const AppContext = createContext<AppContextType>(null!);

export function useAppContext() {
    return useContext(AppContext);
}

export type { WorldPathState, PlayerData, PlayerInfoMap, WorldPathType };
