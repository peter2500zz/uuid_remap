import { AppContext, PlayerInfoMap, WorldPathState } from "../utils/context";
import { UuidPair, isMappingReady } from "../utils/uuidUtils";
import { useState } from "react";
import FolderSelect from "../components/folderSelect";
import UuidPairs from "../components/uuidPair";
import RemapProgress from "../components/remapProgress";
import { Toaster } from "react-hot-toast";

const STEPS = ["选择存档", "设定交换", "应用修改"] as const;

function App() {
	const [onProgressing, setOnProgressing] = useState(false);
	const [worldPathState, setWorldPathState] = useState<WorldPathState>({ path: "", type: "Invalid" });
	const [uuidPairs, setUuidPairs] = useState<UuidPair[]>([]);
	const [playerInfoMap, setPlayerInfoMap] = useState<PlayerInfoMap>({});

	const [step, setStep] = useState(0);
	const canGoBack = [
		false,
		true,
		!onProgressing,
	][step];
	const canGoNext = [
		["Server", "World", "WorldButHasServer", "InvalidButForce"].includes(worldPathState.type),
		isMappingReady(uuidPairs),
		false,
	][step];

	return (
		<main className="overflow-hidden w-full" data-theme="cupcake">
			<AppContext.Provider value={{
				onProgressing, setOnProgressing,
				worldPathState, setWorldPathState,
				uuidPairs, setUuidPairs,
				playerInfoMap, setPlayerInfoMap,
			}}>
				<Toaster />
				<div
					className="flex transition-transform duration-500 ease-in-out"
					style={{ transform: `translateX(-${step * 100}%)` }}
				>
					{[FolderSelect, UuidPairs, RemapProgress].map((Page, index) => (
						// 非当前页设为 inert，防止 Tab 聚焦到屏幕外的元素时浏览器强行滚动容器
						<div key={index} className="min-w-full h-screen flex flex-col" inert={step !== index}>
							<Page />
						</div>
					))}
				</div>

				<div className="fixed bottom-0 right-0 p-4 gap-2 flex">
					<button
						className="btn"
						disabled={!canGoBack}
						onClick={() => setStep(prev => prev - 1)}
					>
						返回
					</button>
					<button
						className="btn btn-primary"
						disabled={!canGoNext}
						onClick={() => setStep(prev => prev + 1)}
					>
						下一步
					</button>
				</div>
				<ul className="steps fixed bottom-0 left-0 w-80 pb-1">
					{STEPS.map((name, index) => (
						<li key={name} className={`step ${step >= index ? "step-primary" : ""}`}>{name}</li>
					))}
				</ul>
			</AppContext.Provider>
		</main>
	);
}

export default App;
