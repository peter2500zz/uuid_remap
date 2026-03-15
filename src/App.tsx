import { useState } from "react";
import FolderSelect from "./components/folderSelect";
import "./App.css";
import UuidPairs from "./components/uuidPair";
import { AppContext } from "./context";

function App() {
	const [worldPathState, setWorldPathState] = useState({ path: "", isValid: false });
	const [uuidMapping, setUuidMapping] = useState<[string, string][]>([]);
	const [nameMapping, setNameMapping] = useState<Record<string, string>>({});

	return (
		<main className="container">
			<div>
				<AppContext.Provider value={{
					worldPathState, setWorldPathState,
					uuidMapping, setUuidMapping,
					nameMapping, setNameMapping,
				}}>
					<button onClick={() => {
						console.log("Debug - worldPathState:", worldPathState);
						console.log("Debug - uuidMapping:", uuidMapping);
						console.log("Debug - nameMapping:", nameMapping);
					}}>Debug</button>
					<FolderSelect />
					<UuidPairs />
				</AppContext.Provider>
			</div>
		</main>
	);
}

export default App;
