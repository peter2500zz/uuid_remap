import { createContext, useContext, Dispatch, SetStateAction } from "react";

interface WorldPathState {
    path: string;
    isValid: boolean;
}

type PlayerMode = "Online" | "Offline" | "NotMatch";

interface PlayerData {
    avatar: string | null;
    name: string;
    mode: PlayerMode;
}

interface AppContextType {
    worldPathState: WorldPathState;
    setWorldPathState: Dispatch<SetStateAction<WorldPathState>>;
    uuidMapping: [string, string][];
    setUuidMapping: Dispatch<SetStateAction<[string, string][]>>;
    nameMapping: Record<string, PlayerData>;
    setNameMapping: Dispatch<SetStateAction<Record<string, PlayerData>>>;
}

export const AppContext = createContext<AppContextType>(null!);

export function useAppContext() {
    return useContext(AppContext);
}

export type { WorldPathState, PlayerData };
