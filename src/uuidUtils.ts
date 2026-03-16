import { md5 } from "js-md5";

function playerNameToOfflineUUID(playerName: string): string {
    const hash = new Uint8Array(md5.array(`OfflinePlayer:${playerName}`));

    hash[6] = (hash[6] & 0x0f) | 0x30;
    hash[8] = (hash[8] & 0x3f) | 0x80;

    const hex = Array.from(hash).map(b => b.toString(16).padStart(2, "0")).join("");
    return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`;
}

export { playerNameToOfflineUUID };
