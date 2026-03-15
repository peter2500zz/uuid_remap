import { useEffect, useState } from "react";
import style from "./uuidPair.module.css";
import { useAppContext } from "../context";

function UuidPair({ index, oldUuid, newUuid }: {
    index: number;
    oldUuid: string;
    newUuid: string;
}) {
    const {
        setUuidMapping,
    } = useAppContext();

    // 给 input 提供用于修改的函数，因为左值也可以修改所以用索引定位了
    const changeUuid = (index: number, oldUuid: string, newUuid: string) => {
        setUuidMapping(prev => prev.map(([k, v], i) =>
            i === index ? [oldUuid, newUuid] : [k, v]
        ));
    };

    return (
        <div>
            <input value={oldUuid} onChange={e => changeUuid(index, e.target.value, newUuid)} />
            <button onClick={() => changeUuid(index, newUuid, oldUuid)}>↔</button>
            <input value={newUuid} onChange={e => changeUuid(index, oldUuid, e.target.value)} />
            <button onClick={() => setUuidMapping(prev => prev.filter((_, i) => i !== index))}>
                删除
            </button>
        </div>
    )
}

function UuidPairs() {
    const [display, setDisplay] = useState(false);
    const {
        worldPathState,
        uuidMapping,
        setUuidMapping,
    } = useAppContext();

    useEffect(() => {
        setDisplay(!!worldPathState.path);

    }, [worldPathState]);

    return (
        <div className={style.container}>
            <button onClick={() => setDisplay(!display)} disabled={!worldPathState.path}>
                设定UUID转换规则
            </button>

            {display && (
                <div>
                    {uuidMapping.map(([oldUuid, newUuid], index) => (
                        <UuidPair key={index} index={index} oldUuid={oldUuid} newUuid={newUuid} />
                    ))}
                    <button onClick={() => setUuidMapping(prev => [...prev, ["", ""]])}>
                        +
                    </button>
                </div>
            )}
        </div>
    )
}

export default UuidPairs;
