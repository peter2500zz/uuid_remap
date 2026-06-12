import { invoke } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";
import { useAppContext } from "../utils/context";
import { listen } from "@tauri-apps/api/event";
import toast from "react-hot-toast";

// 阶段 0：并行处理文件内容，同时展示所有进行中的任务
// 阶段 1：串行重命名文件，只展示当前任务
type Phase = 0 | 1;

function formatTime(seconds: number) {
    const h = Math.floor(seconds / 3600);
    const m = Math.floor((seconds % 3600) / 60);
    const s = seconds % 60;

    if (h > 0) return `${h}h ${m}m ${s}s`;
    if (m > 0) return `${m}m ${s}s`;
    return `${s}s`;
}

function RemapProgress() {
    const [lastRun, setLastRun] = useState<{ total: number; seconds: number } | null>(null);
    const [progress, setProgress] = useState({ done: 0, total: 0 });
    const [runningTasks, setRunningTasks] = useState<string[]>([]);
    const [currentTask, setCurrentTask] = useState("");
    const [phase, setPhase] = useState<Phase>(0);
    // 事件监听器只注册一次，需要用 ref 读取最新的阶段与总数
    const phaseRef = useRef<Phase>(0);
    const totalRef = useRef(0);
    const {
        onProgressing, setOnProgressing,
        worldPathState,
        uuidPairs,
    } = useAppContext();

    const [elapsed, setElapsed] = useState(0);
    const timerRef = useRef<ReturnType<typeof setInterval> | null>(null);
    const startTimeRef = useRef(0);

    const startTimer = () => {
        startTimeRef.current = Date.now();
        setElapsed(0);
        timerRef.current = setInterval(() => {
            setElapsed(Math.round((Date.now() - startTimeRef.current) / 1000));
        }, 1000);
    };

    const stopTimer = () => {
        if (timerRef.current) {
            clearInterval(timerRef.current);
            timerRef.current = null;
        }
    };

    useEffect(() => {
        return () => stopTimer();
    }, []);

    useEffect(() => {
        const unlisteners = [
            listen<number>('set-total', (event) => {
                console.log("Total tasks to process:", event.payload);
                totalRef.current = event.payload;
                setProgress({ done: 0, total: event.payload });
            }),
            listen<Phase>('start-phase', (event) => {
                console.log("Starting phase:", event.payload);
                phaseRef.current = event.payload;
                setPhase(event.payload);
                setRunningTasks([]);
                setCurrentTask("");
            }),
            listen<string>('start-task', (event) => {
                if (phaseRef.current === 0) {
                    setRunningTasks(prev => [...prev, event.payload]);
                } else {
                    setCurrentTask(event.payload);
                }
            }),
            listen<string>('finish-task', (event) => {
                if (phaseRef.current === 0) {
                    setRunningTasks(prev => prev.filter(task => task !== event.payload));
                }
                setProgress(prev => ({ ...prev, done: prev.done + 1 }));
            }),
        ];

        return () => {
            unlisteners.forEach(unlisten => unlisten.then(f => f()));
        };
    }, []);

    const handleStart = async () => {
        setOnProgressing(true);
        startTimer();

        try {
            await invoke("process_world", {
                worldPath: worldPathState.path,
                uuidMap: Object.fromEntries(uuidPairs),
            });
            setLastRun({
                total: totalRef.current,
                seconds: Math.round((Date.now() - startTimeRef.current) / 1000),
            });
        } catch (e) {
            toast.error(`转换失败: ${(e as Error).message || String(e)}`);
        } finally {
            stopTimer();
            setOnProgressing(false);
        }
    };

    return (
        <div className="px-24 py-16 pb-32">
            <div className="relative min-h-[400px]">
                <div className="absolute inset-0 flex items-center justify-center pointer-events-none">
                    {
                        lastRun ? (
                            <span className="text-xl font-bold text-base-content/40 z-0 text-center w-130">
                                转换完成<br /><br />
                                {`处理了 ${lastRun.total} 个任务，耗时 ${formatTime(lastRun.seconds)}`}<br /><br />
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
                    <button className="btn" disabled={onProgressing} onClick={handleStart}>
                        开始转换
                    </button>

                    <div>
                        <div className="flex justify-between">
                            <label className="label">{progress.done} / {progress.total}</label>
                            <label className="label">{formatTime(elapsed)}</label>
                        </div>
                        <progress className="progress" value={progress.done} max={progress.total} />
                    </div>
                    <div className="h-64 overflow-auto">
                        {
                            onProgressing && (
                                phase === 0 ? (
                                    <ul className="whitespace-nowrap">
                                        {runningTasks.map((task, index) => (
                                            <li className="text-sm text-base-content/70" key={index}>{task}</li>
                                        ))}
                                    </ul>
                                ) : (
                                    <span className="whitespace-nowrap text-sm text-base-content/70">{currentTask}</span>
                                )
                            )
                        }
                    </div>
                </div>
            </div>
        </div>
    )
}

export default RemapProgress;
