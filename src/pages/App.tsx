import { AppContext, PlayerInfoMap, WorldPathState } from "../utils/context";
import { UuidPair, isMappingReady } from "../utils/uuidUtils";
import { updateUuidMap, updateErrorText } from "../utils/ipc";
import { useRef, useState } from "react";
import FolderSelect from "../components/folderSelect";
import UuidPairs from "../components/uuidPair";
import RemapProgress from "../components/remapProgress";
import toast, { Toaster } from "react-hot-toast";
import { useI18n } from "../i18n/context";

function App() {
	const { t } = useI18n();
	const STEPS = [t("steps.selectWorld"), t("steps.configureMapping"), t("steps.apply")];

	const [onProgressing, setOnProgressing] = useState(false);
	const [worldPathState, setWorldPathState] = useState<WorldPathState>({ path: "", type: "Invalid" });
	const [uuidPairs, setUuidPairs] = useState<UuidPair[]>([]);
	const [playerInfoMap, setPlayerInfoMap] = useState<PlayerInfoMap>({});

	const [step, setStep] = useState(0);
	// 提交进行中时忽略重复点击，防止双击把步骤推进两次
	const commitInFlight = useRef(false);
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

	// 离开交换设定页前把草稿全量提交到后端 state（转换用的是后端持有的映射）。
	// 格式/重复问题已被本地校验拦下，这里的失败只是两边判定不一致时的兜底
	const goNext = async () => {
		if (step === 1) {
			if (commitInFlight.current) return;
			commitInFlight.current = true;
			try {
				await updateUuidMap(uuidPairs);
			} catch (e) {
				toast.error(t("uuidPair.commitFailed", { message: updateErrorText(e) }));
				return;
			} finally {
				commitInFlight.current = false;
			}
		}
		setStep(prev => prev + 1);
	};

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
						onClick={goNext}
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
