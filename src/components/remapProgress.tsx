import { invoke } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";
import { useAppContext } from "../utils/context";
import { listen } from "@tauri-apps/api/event";

function RemapProgress() {
    const [runnedAtLeastOnce, setRunnedAtLeastOnce] = useState(false);
    const [totalResult, setTotalResult] = useState(0);
    const [timeResult, setTimeResult] = useState(0);
    const [progress, setProgress] = useState([0, 0]);
    const [tasks, setTasks] = useState<string[]>([]);
    const [currentTask, setCurrentTask] = useState<string>("");
    const {
        onProgressing, setOnProgressing,
        worldPathState,
        uuidMapping,
    } = useAppContext();
    const [phase, setPhase] = useState(0);
    const phaseRef = useRef(phase);

    useEffect(() => {
        phaseRef.current = phase;
    }, [phase]);
    const [dataEachPhase, initPhaseData, startTaskEachPhase, finishTaskEachPhase] = [
        [
            (
                <ul className="whitespace-nowrap">
                    {tasks.map((task, index) => (
                        <li className="text-sm text-base-content/70" key={index}>{task}</li>
                    ))}
                </ul>
            ),
            (<>
                <span className="whitespace-nowrap text-sm text-base-content/70">{currentTask}</span>
            </>)
        ],
        [
            () => setTasks([]),
            () => setCurrentTask(""),
        ],
        [
            (taskName: string) => setTasks(prev => [...prev, taskName]),
            (taskName: string) => setCurrentTask(taskName),
        ],
        [
            (taskName: string) => setTasks(prev => prev.filter(task => task !== taskName)),
            (_: string) => { },
        ]
    ];

    const [elapsed, setElapsed] = useState(0)
    const timerRef = useRef<ReturnType<typeof setInterval> | null>(null)

    const startTimer = () => {
        setElapsed(0)
        timerRef.current = setInterval(() => {
            setElapsed(prev => prev + 1)
        }, 1000)
    }

    const stopTimer = () => {
        if (timerRef.current) {
            clearInterval(timerRef.current)
            timerRef.current = null
        }
    }

    const formatTime = (seconds: number) => {
        const h = Math.floor(seconds / 3600)
        const m = Math.floor((seconds % 3600) / 60)
        const s = seconds % 60

        if (h > 0) return `${h}h ${m}m ${s}s`
        if (m > 0) return `${m}m ${s}s`
        return `${s}s`
    }

    useEffect(() => {
        return () => stopTimer()
    }, [])

    useEffect(() => {

        const unlistenSetTotal = listen<number>('set-total', (event) => {
            console.log("Total tasks to process:", event.payload);
            setProgress([0, event.payload]);
        });

        const unlistenStartPhase = listen<number>('start-phase', (event) => {
            console.log("Starting phase:", event.payload);
            initPhaseData[event.payload]();
            setPhase(event.payload);
        });

        const unlistenStartTask = listen<string>('start-task', (event) => {
            startTaskEachPhase[phaseRef.current](event.payload);
        });

        const unlistenFinishTask = listen<string>('finish-task', (event) => {
            finishTaskEachPhase[phaseRef.current](event.payload);
            setProgress(prev => [prev[0] + 1, prev[1]]);
        });

        return () => {
            unlistenSetTotal.then(f => f());
            unlistenStartPhase.then(f => f());
            unlistenStartTask.then(f => f());
            unlistenFinishTask.then(f => f());
        };
    }, []);

    return (
        <div className="px-24 py-16 pb-32">
            <div className="relative min-h-[400px]">
                <div className="absolute inset-0 flex items-center justify-center pointer-events-none">
                    {
                        runnedAtLeastOnce ? (
                            <span className="text-xl font-bold text-base-content/40 z-0 text-center w-130">
                                转换完成<br /><br />
                                {`处理了 ${totalResult} 个任务，耗时 ${formatTime(timeResult)}`}<br /><br />
                            </span>
                        ) : (
                            <span className="text-xl font-bold text-base-content/40 z-0 text-center w-130">
                                在转换前，请务必备份你的世界存档。<br /><br />
                                同时，请关闭正在使用此世界的 Minecraft 游戏/服务器，以免造成数据损坏。<br /><br />
                                本工具不对转换过程中可能出现的问题负责。
                            </span>
                        )
                    }
                </div>
                <div className={`
                    relative z-10 flex flex-col pt-3 p-4 border border-base-300 bg-base-100 rounded-xl shadow-sm gap-2
                    transition-[max-height] duration-500 ease-in-out overflow-hidden
                    ${onProgressing ? "max-h-[400px]" : "max-h-24"}
                `}>
                    <button className={`btn ${onProgressing ? "btn-disabled" : ""}`} onClick={async () => {
                        setOnProgressing(true);
                        startTimer();
                        await invoke("process_world", {
                            worldPath: worldPathState.path,
                            uuidMap: Object.fromEntries(uuidMapping),
                        });
                        stopTimer();
                        setProgress(prev => {
                            setTotalResult(prev[1]);
                            return prev;
                        });
                        setElapsed(prev => {
                            setTimeResult(prev);
                            return prev;
                        });
                        setRunnedAtLeastOnce(true);
                        setOnProgressing(false);
                    }}>
                        开始转换
                    </button>
                    <div className="flex flex-row gap-2">
                        <label className="cursor-pointer label">
                            <span className="label-text">DEBUG</span>
                        </label>
                        <input type="checkbox" className="toggle" onChange={() => { }} />
                        <label className="cursor-pointer label">
                            <span className="label-text">DEBUG</span>
                        </label>
                        <input type="checkbox" className="toggle" defaultChecked={true} onChange={() => { }} />
                    </div>

                    <div>
                        <div className="flex justify-between">
                            <label className="label">{progress[0]} / {progress[1]}</label>
                            <label className="label">{formatTime(elapsed)}</label>
                        </div>
                        <progress className="progress" value={progress[0]} max={progress[1]} />
                    </div>
                    <div className="h-screen overflow-y-scroll overflow-x-scroll">
                        {
                            onProgressing &&
                            <div>
                                {dataEachPhase[phase]}
                            </div>
                        }
                    </div>

                </div>
            </div>

        </div>
    )
}

export default RemapProgress;
