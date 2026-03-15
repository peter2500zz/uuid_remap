import { useState } from "react";
import FolderSelect from "./components/folderSelect";
import "./App.css";
import UuidPairs from "./components/uuidPair";
import { AppContext } from "./context";

function App() {
	const [worldPathState, setWorldPathState] = useState({ path: "", isValid: false });
	const [uuidMapping, setUuidMapping] = useState<{ [key: string]: string }>({});
	const [nameMapping, setNameMapping] = useState<{ [key: string]: string }>({});

	return (
		<main className="container">
			<div>
				<AppContext.Provider value={{
					worldPathState, setWorldPathState,
					uuidMapping, setUuidMapping,
					nameMapping, setNameMapping,
				}}>
					<FolderSelect />
					<UuidPairs />
				</AppContext.Provider>
			</div>
		</main>
	);
}

export default App;
