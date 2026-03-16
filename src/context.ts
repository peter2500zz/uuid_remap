import { createContext, useContext, Dispatch, SetStateAction } from "react";

interface WorldPathState {
    path: string;
    isValid: boolean;
}

type PlayerMode = "Online" | "Offline" | "NotMatch";

interface PLayerData {
    avatar: string | null;
    name: string;
    mode: PlayerMode;
}

interface AppContextType {
    worldPathState: WorldPathState;
    setWorldPathState: Dispatch<SetStateAction<WorldPathState>>;
    uuidMapping: [string, string][];
    setUuidMapping: Dispatch<SetStateAction<[string, string][]>>;
    nameMapping: Record<string, PLayerData>;
    setNameMapping: Dispatch<SetStateAction<Record<string, PLayerData>>>;
}

export const AppContext = createContext<AppContextType>(null!);

export function useAppContext() {
    return useContext(AppContext);
}

export type { WorldPathState, PLayerData };
