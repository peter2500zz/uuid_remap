import { AppContext, PlayerData, WorldPathType } from "../utils/context";
import { useState } from "react";
import FolderSelect from "../components/folderSelect";
import UuidPairs, { hasDuplicates, hasInvalidUUID } from "../components/uuidPair";
import RemapProgress from "../components/remapProgress";
import toast, { Toaster } from "react-hot-toast";


function App() {
	const [worldPathState, setWorldPathState] = useState({ path: "", type: "Invalid" as WorldPathType });
	const [uuidMapping, setUuidMapping] = useState<[string, string][]>([]);
	const [nameMapping, setNameMapping] = useState<Record<string, PlayerData>>({});

	const [cur, setCur] = useState(0);
	const canNext = [
		worldPathState.type === "Server" || worldPathState.type === "World" || worldPathState.type === "WorldButHasServer" || worldPathState.type === "InvalidButForce",
		!hasDuplicates(uuidMapping) && !hasInvalidUUID(uuidMapping) && uuidMapping.length > 0,
		true,
	];

	return (
		<main className="overflow-hidden w-full" data-theme="cupcake">
			<AppContext.Provider value={{
				worldPathState, setWorldPathState,
				uuidMapping, setUuidMapping,
				nameMapping, setNameMapping,
			}}>
				<div><Toaster/></div>
				<div
					className="flex transition-transform duration-500 ease-in-out"
					style={{ transform: `translateX(-${cur * 100}%)` }}
				>
					<div className="min-w-full h-screen flex flex-col">
						<FolderSelect />
					</div>
					<div className="min-w-full h-screen flex flex-col">
						<UuidPairs />
					</div>
					<div className="min-w-full h-screen flex flex-col">
						<RemapProgress />
					</div>
				</div>

				<div className="fixed bottom-0 right-0 p-4 gap-2 flex">
					<button className="btn" onClick={() => {
						toast("Hello World");
					}}>
						DEBUG
					</button>
					<button
						className={`btn ${cur === 0 ? "btn-disabled" : ""}`}
						onClick={() => {
							setCur(cur - 1);
						}}
					>
						返回
					</button>
					<button className={`btn btn-primary ${(cur >= canNext.length - 1 || !canNext[cur]) ? "btn-disabled" : ""}`} onClick={() => {
						setCur(cur + 1);
					}}>
						下一步
					</button>
				</div>
				<ul className="steps fixed bottom-0 left-0 w-80 pb-1">
					<li className={`step ${cur >= 0 ? "step-primary" : ""}`}>选择存档</li>
					<li className={`step ${cur >= 1 ? "step-primary" : ""}`}>设定映射</li>
					<li className={`step ${cur >= 2 ? "step-primary" : ""}`}>应用修改</li>
				</ul>
			</AppContext.Provider>
		</main>
	);
}

export default App;
