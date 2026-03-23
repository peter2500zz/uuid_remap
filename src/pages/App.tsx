import { AppContext, PlayerData } from "../utils/context";
import { useState } from "react";
import FolderSelect from "../components/folderSelect";
import UuidPairs from "../components/uuidPair";
import RemapProgress from "../components/remapProgress";


function App() {
	const [worldPathState, setWorldPathState] = useState({ path: "", isValid: false });
	const [uuidMapping, setUuidMapping] = useState<[string, string][]>([]);
	const [nameMapping, setNameMapping] = useState<Record<string, PlayerData>>({});

	const [cur, setCur] = useState(0);
	const [canNext, setCanNext] = useState(false);

	return (
		<main className="overflow-hidden min-h-screen w-full" data-theme="cupcake">
			<AppContext.Provider value={{
				worldPathState, setWorldPathState,
				uuidMapping, setUuidMapping,
				nameMapping, setNameMapping,
			}}>
				<div
					className="flex transition-transform duration-500 ease-in-out"
					style={{ transform: `translateX(-${cur * 100}%)` }}
				>
					<div className="min-w-full min-h-screen flex flex-col">
						<FolderSelect />
						<div className="mt-auto flex justify-end p-4">
							<button className="btn btn-primary" onClick={() => setCur(1)}>Next</button>
						</div>
					</div>
					<div className="min-w-full min-h-screen flex flex-col">
						<UuidPairs />
						<div className="mt-auto flex justify-end p-4">
							<button className="btn" onClick={() => setCur(0)}>Prev</button>
							<button className="btn btn-primary" onClick={() => setCur(2)}>Next</button>
						</div>
					</div>
					<div className="min-w-full min-h-screen flex flex-col">
						<RemapProgress />
						<div className="mt-auto flex justify-end p-4">
							<button className="btn" onClick={() => setCur(1)}>Prev</button>
						</div>
					</div>
				</div>
			</AppContext.Provider>
		</main>
	);
}

export default App;
