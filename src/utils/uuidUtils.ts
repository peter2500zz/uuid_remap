import { md5 } from "js-md5";

/// 一对将被双向互换的 UUID，左右没有方向之分
type UuidPair = [left: string, right: string];

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
    const clean = uuid.replace(/-/g, "");
    if (!/^[0-9a-f]{32}$/i.test(clean)) return null;
    return `${clean.slice(0, 8)}-${clean.slice(8, 12)}-${clean.slice(12, 16)}-${clean.slice(16, 20)}-${clean.slice(20)}`;
}

// 检测某个 UUID 在交换列表中是否出现超过一次
function isUuidDuplicated(uuidPairs: UuidPair[], target: string): boolean {
    let count = 0;
    for (const [left, right] of uuidPairs) {
        if (left === target) count++;
        if (right === target) count++;
        if (count > 1) return true;
    }
    return false;
}

// 检测交换列表中是否存在任意重复的 UUID
function hasDuplicates(uuidPairs: UuidPair[]): boolean {
    const seen = new Set<string>();
    for (const pair of uuidPairs) {
        for (const uuid of pair) {
            if (seen.has(uuid)) return true;
            seen.add(uuid);
        }
    }
    return false;
}

// 检测交换列表中是否存在无效 UUID
function hasInvalidUUID(uuidPairs: UuidPair[]): boolean {
    return uuidPairs.some(pair => pair.some(uuid => !isValidUUID(uuid)));
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
};
export type { UuidPair };
