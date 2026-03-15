import { createContext, useContext, Dispatch, SetStateAction } from "react";

interface WorldPathState {
    path: string;
    isValid: boolean;
}

interface AppContextType {
    worldPathState: WorldPathState;
    setWorldPathState: Dispatch<SetStateAction<WorldPathState>>;
    uuidMapping: Record<string, string>;
    setUuidMapping: Dispatch<SetStateAction<Record<string, string>>>;
    nameMapping: Record<string, string>;
    setNameMapping: Dispatch<SetStateAction<Record<string, string>>>;
}

export const AppContext = createContext<AppContextType>(null!);

export function useAppContext() {
    return useContext(AppContext);
}
