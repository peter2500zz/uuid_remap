import { AppContext, PlayerInfoMap, WorldPathState } from "../utils/context";
import { UuidPair, isMappingReady } from "../utils/uuidUtils";
import { useState } from "react";
import FolderSelect from "../components/folderSelect";
import UuidPairs from "../components/uuidPair";
import RemapProgress from "../components/remapProgress";
import { Toaster } from "react-hot-toast";
import { useI18n } from "../i18n/context";

function App() {
	const { t } = useI18n();
	const STEPS = [t("steps.selectWorld"), t("steps.configureMapping"), t("steps.apply")];

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
						{t("nav.back")}
					</button>
					<button
						className="btn btn-primary"
						disabled={!canGoNext}
						onClick={() => setStep(prev => prev + 1)}
					>
						{t("nav.next")}
					</button>
				</div>
				<ul className="steps fixed bottom-0 left-0 min-w-80 w-fit pb-1">
					{STEPS.map((name, index) => (
						<li key={name} className={`step whitespace-nowrap ${step >= index ? "step-primary" : ""}`}>{name}</li>
					))}
				</ul>
			</AppContext.Provider>
		</main>
	);
}

export default App;
