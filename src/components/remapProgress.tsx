import { useEffect, useRef, useState } from "react";
import { useAppContext } from "../utils/context";
import { listen } from "@tauri-apps/api/event";
import { FileProcessError, FinishTaskData, processWorld, processErrorText } from "../utils/ipc";
import toast from "react-hot-toast";
import { useI18n } from "../i18n/context";

// 阶段 0：并行处理文件内容，同时展示所有进行中的任务
// 阶段 1：串行重命名文件，只展示当前任务
type Phase = 0 | 1;

// 有内容才出现的日志标签页：error = 处理失败，unsupported = 无法识别而未处理。
// 与普通日志（任务实时出现/消失）不同，这些数据在转换结束后保留，直到下一次开始
type LogTab = "error" | "unsupported";
const LOG_TABS: LogTab[] = ["error", "unsupported"];

interface TaskIssue {
    path: string;
    errors: FileProcessError[];
}

// 单个标签页最多渲染的条目数，防止异常文件极多时 DOM 失控
const MAX_ISSUE_LINES = 500;

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
    // 各标签页累积的异常条目；activeTab 为 null 时显示普通日志
    const [issues, setIssues] = useState<Record<LogTab, TaskIssue[]>>({ error: [], unsupported: [] });
    const [activeTab, setActiveTab] = useState<LogTab | null>(null);
    // 事件监听器只注册一次，需要用 ref 读取最新的阶段与总数
    const phaseRef = useRef<Phase>(0);
    const totalRef = useRef(0);
    const {
        onProgressing, setOnProgressing,
        worldPathState,
    } = useAppContext();
    const { t } = useI18n();

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
            listen<FinishTaskData>('finish-task', (event) => {
                const { path, result } = event.payload;
                if (phaseRef.current === 0) {
                    setRunningTasks(prev => prev.filter(task => task !== path));
                }
                // 异常结果进对应标签页；Success/NoChange 只计入进度
                if (result.kind === "Unsupported" || result.kind === "Error") {
                    const tab: LogTab = result.kind === "Error" ? "error" : "unsupported";
                    setIssues(prev => ({ ...prev, [tab]: [...prev[tab], { path, errors: result.data }] }));
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
        // 上一轮的标签页数据作废：清空并回到普通日志模式
        setIssues({ error: [], unsupported: [] });
        setActiveTab(null);
        startTimer();

        try {
            // 交换表已在进入本页前提交到后端 state，这里只需传存档路径
            await processWorld(worldPathState.path);
            setLastRun({
                total: totalRef.current,
                seconds: Math.round((Date.now() - startTimeRef.current) / 1000),
            });
        } catch (e) {
            toast.error(t("remapProgress.failed", { message: processErrorText(e) }));
        } finally {
            stopTimer();
            setOnProgressing(false);
        }
    };

    return (
        <div className="flex-1 flex flex-col justify-center px-24 pb-32">
            <div className="relative min-h-[400px]">
                <div className="absolute inset-x-0 top-28 bottom-0 flex items-center justify-center pointer-events-none">
                    {
                        lastRun ? (
                            <span className="text-xl font-bold text-base-content/40 z-0 text-center w-130">
                                {t("remapProgress.done")}<br /><br />
                                {t("remapProgress.summary", { total: lastRun.total, time: formatTime(lastRun.seconds) })}<br /><br />
                            </span>
                        ) : (
                            <span className="text-xl font-bold text-base-content/40 z-0 text-center w-130">
                                {t("remapProgress.warning.line1")}<br /><br />
                                {t("remapProgress.warning.line2")}<br /><br />
                                {t("remapProgress.warning.line3")}
                            </span>
                        )
                    }
                </div>
                <div className={`
                    relative z-10 flex flex-col pt-3 p-4 border border-base-300 bg-base-100 rounded-xl shadow-sm gap-2
                    transition-[max-height] duration-500 ease-in-out overflow-hidden
                    ${onProgressing || activeTab ? "max-h-[400px]" : "max-h-24"}
                `}>
                    <button className="btn" disabled={onProgressing} onClick={handleStart}>
                        {t("remapProgress.start")}
                    </button>

                    <div>
                        <div className="flex justify-between items-center">
                            {/* 进度数字与时间是固定元素，标签按钮从进度数字右侧开始出现；
                                这一行在卡片收起时也可见，因此结束后仍可点开错误/未处理页 */}
                            <div className="flex items-center gap-1 min-w-0">
                                <label className="label">{progress.done} / {progress.total}</label>
                                {/* h-5 压到与 label 行同高，避免收起态 max-h-24 裁掉进度条底部 */}
                                {LOG_TABS.map(tab => issues[tab].length > 0 && (
                                    <button
                                        key={tab}
                                        className={`
                                            btn btn-xs h-5 ${activeTab === tab ? "btn-active" : "btn-ghost"}
                                            ${tab === "error" ? "text-error" : "text-warning"}
                                        `}
                                        onClick={() => setActiveTab(prev => prev === tab ? null : tab)}
                                    >
                                        {t(`remapProgress.tab.${tab}`)} ({issues[tab].length})
                                    </button>
                                ))}
                            </div>
                            <label className="label">{formatTime(elapsed)}</label>
                        </div>
                        <progress className="progress" value={progress.done} max={progress.total} />
                    </div>
                    <div className="h-64 overflow-auto">
                        {activeTab ? (
                            // 标签页日志：转换结束后不清空，直到下一次开始
                            <ul className="whitespace-nowrap">
                                {issues[activeTab].slice(0, MAX_ISSUE_LINES).map((issue, index) => (
                                    <li className="text-sm text-base-content/70" key={index}>
                                        {issue.path}
                                        <ul className="pl-4 text-xs text-base-content/50">
                                            {issue.errors.map((err, errIndex) => (
                                                <li key={errIndex}>{err.kind}: {err.data}</li>
                                            ))}
                                        </ul>
                                    </li>
                                ))}
                                {issues[activeTab].length > MAX_ISSUE_LINES && (
                                    <li className="text-sm text-base-content/50">
                                        {t("remapProgress.tab.more", { count: issues[activeTab].length - MAX_ISSUE_LINES })}
                                    </li>
                                )}
                            </ul>
                        ) : (
                            // 普通日志模式：任务实时出现/消失，仅运行中显示
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
                        )}
                    </div>
                </div>
            </div>
        </div>
    )
}

export default RemapProgress;
