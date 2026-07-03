import { md5 } from "js-md5";

/// 一对将被双向互换的 UUID，左右没有方向之分
///
/// id 仅用于 React 列表渲染的稳定标识（用 index 作 key 会在删除时
/// 让后续行复用错误的 DOM 节点），不参与业务逻辑
interface UuidPair {
    id: string;
    left: string;
    right: string;
}

let nextPairId = 0;

function createUuidPair(left = "", right = ""): UuidPair {
    return { id: `pair-${nextPairId++}`, left, right };
}

function playerNameToOfflineUUID(playerName: string): string {
    const hash = new Uint8Array(md5.array(`OfflinePlayer:${playerName}`));

    hash[6] = (hash[6] & 0x0f) | 0x30;
    hash[8] = (hash[8] & 0x3f) | 0x80;

    const hex = Array.from(hash).map(b => b.toString(16).padStart(2, "0")).join("");
    return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`;
}

function isValidUUID(uuid: string): boolean {
    return /^[0-9a-f]{8}-?[0-9a-f]{4}-?[0-9a-f]{4}-?[0-9a-f]{4}-?[0-9a-f]{12}$/i.test(uuid);
}

function normalizeUUID(uuid: string | null): string | null {
    if (!uuid) return null;
    const clean = uuid.replace(/-/g, "").toLowerCase();
    if (!/^[0-9a-f]{32}$/.test(clean)) return null;
    return `${clean.slice(0, 8)}-${clean.slice(8, 12)}-${clean.slice(12, 16)}-${clean.slice(16, 20)}-${clean.slice(20)}`;
}

// 查重前先归一化，大小写/连字符变体视为同一个 UUID，与后端 Uuid 的解析行为一致
function canonicalUUID(uuid: string): string {
    return normalizeUUID(uuid) ?? uuid;
}

// 检测某个 UUID 在交换列表中是否出现超过一次
function isUuidDuplicated(uuidPairs: UuidPair[], target: string): boolean {
    const canonTarget = canonicalUUID(target);
    let count = 0;
    for (const { left, right } of uuidPairs) {
        if (canonicalUUID(left) === canonTarget) count++;
        if (canonicalUUID(right) === canonTarget) count++;
        if (count > 1) return true;
    }
    return false;
}

// 检测交换列表中是否存在任意重复的 UUID
function hasDuplicates(uuidPairs: UuidPair[]): boolean {
    const seen = new Set<string>();
    for (const { left, right } of uuidPairs) {
        for (const uuid of [left, right]) {
            const canon = canonicalUUID(uuid);
            if (seen.has(canon)) return true;
            seen.add(canon);
        }
    }
    return false;
}

// 检测交换列表中是否存在无效 UUID
function hasInvalidUUID(uuidPairs: UuidPair[]): boolean {
    return uuidPairs.some(({ left, right }) => !isValidUUID(left) || !isValidUUID(right));
}

// 交换列表非空且没有重复/无效 UUID 时才可以应用
function isMappingReady(uuidPairs: UuidPair[]): boolean {
    return uuidPairs.length > 0 && !hasDuplicates(uuidPairs) && !hasInvalidUUID(uuidPairs);
}

export {
    playerNameToOfflineUUID,
    isValidUUID,
    normalizeUUID,
    isUuidDuplicated,
    hasDuplicates,
    hasInvalidUUID,
    isMappingReady,
    createUuidPair,
};
export type { UuidPair };
