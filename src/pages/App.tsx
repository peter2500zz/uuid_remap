import { AppContext, PlayerData, WorldPathType } from "../utils/context";
import { useState } from "react";
import FolderSelect from "../components/folderSelect";
import UuidPairs from "../components/uuidPair";
import RemapProgress from "../components/remapProgress";


function App() {
	const [worldPathState, setWorldPathState] = useState({ path: "", type: "Invalid" as WorldPathType });
	const [uuidMapping, setUuidMapping] = useState<[string, string][]>([]);
	const [nameMapping, setNameMapping] = useState<Record<string, PlayerData>>({});

	const [cur, setCur] = useState(0);
	const [canNext, setCanNext] = useState(false);

	return (
		<main className="overflow-hidden w-full" data-theme="cupcake">
			<AppContext.Provider value={{
				worldPathState, setWorldPathState,
				uuidMapping, setUuidMapping,
				nameMapping, setNameMapping,
			}}>
				<div
					className="flex transition-transform duration-500 ease-in-out"
					style={{ transform: `translateX(-${cur * 100}%)` }}
				>
					<div className="min-w-full h-screen flex flex-col">
						<FolderSelect canNext={canNext} setCanNext={setCanNext} />
					</div>
					<div className="min-w-full h-screen flex flex-col">
						<UuidPairs />
					</div>
					<div className="min-w-full h-screen flex flex-col">
						<RemapProgress />
					</div>
				</div>

				<div className="fixed bottom-0 right-0 p-4 gap-2 flex">
					<button
						className={`btn ${cur === 0 ? "btn-disabled" : ""}`}
						onClick={() => {
							setCur(cur - 1);
						}}
					>
						Prev
					</button>
					<button className={`btn btn-primary ${(cur === 2 || !canNext) ? "btn-disabled" : ""}`} onClick={() => {
						setCur(cur + 1);
					}}>
						Next
					</button>
				</div>
				<ul className="steps fixed bottom-0 left-0">
					<li className={`step ${cur >= 0 ? "step-primary" : ""}`}>选择世界目录</li>
					<li className={`step ${cur >= 1 ? "step-primary" : ""}`}>设定UUID映射</li>
					<li className={`step ${cur >= 2 ? "step-primary" : ""}`}>原神启动</li>
				</ul>
			</AppContext.Provider>
		</main>
	);
}

export default App;
