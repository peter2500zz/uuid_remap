import { useState } from "react";
import FolderSelect from "./components/folderSelect";
import "./App.css";
import UuidPairs from "./components/uuidPair";

function App() {
	const [worldPathState, setWorldPathState] = useState({ path: "", isValid: false });

	return (
		<main className="container">
			<div>
				<FolderSelect worldPathState={worldPathState} setWorldPathState={(path, isValid) => {
					setWorldPathState({ path, isValid });
				}} />
				<UuidPairs worldPath={worldPathState.path} />
			</div>
		</main>
	);
}

export default App;
