import { AppContext, PlayerData } from "../utils/context";
import { useState } from "react";
import FolderSelect from "../components/folderSelect";
import UuidPairs from "../components/uuidPair";
import RemapProgress from "../components/remapProgress";

function App() {
	const [worldPathState, setWorldPathState] = useState({ path: "", isValid: false });
	const [uuidMapping, setUuidMapping] = useState<[string, string][]>([]);
	const [nameMapping, setNameMapping] = useState<Record<string, PlayerData>>({});

	return (
		<main className="min-h-screen w-full bg-base-100 px-4 py-6 sm:px-6" data-theme="cupcake">
			<AppContext.Provider value={{
				worldPathState, setWorldPathState,
				uuidMapping, setUuidMapping,
				nameMapping, setNameMapping,
			}}>
				<div className="mx-auto flex w-full max-w-4xl flex-col items-center gap-4">
					<h1 className="text-3xl font-bold">Minecraft UUID 映射工具</h1>
					<FolderSelect />
					<UuidPairs />
					<RemapProgress />
				</div>
			</AppContext.Provider>
		</main>
	);
}

export default App;
