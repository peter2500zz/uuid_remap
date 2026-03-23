import { Dispatch, SetStateAction } from "react";
import { PlayerData } from "./context";
import { fetch } from '@tauri-apps/plugin-http';
import { normalizeUUID, playerNameToOfflineUUID } from "./uuidUtils";

function toGrayscale(base64: string): Promise<string> {
    return new Promise((resolve) => {
        const img = new Image();
        img.onload = () => {
            const canvas = document.createElement('canvas');
            canvas.width = img.width;
            canvas.height = img.height;
            const ctx = canvas.getContext('2d')!;
            ctx.drawImage(img, 0, 0);

            const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
            const data = imageData.data;
            for (let i = 0; i < data.length; i += 4) {
                const gray = data[i] * 0.299 + data[i + 1] * 0.587 + data[i + 2] * 0.114;
                data[i] = data[i + 1] = data[i + 2] = gray;
            }
            ctx.putImageData(imageData, 0, 0);
            resolve(canvas.toDataURL());
        };
        img.src = base64;
    });
}

async function overlayImageRegions(url: string): Promise<string> {
    // 加载原始图像
    const img = await loadImage(url);

    // 创建用于截取底层（从 8,8 开始的 8x8 区域）
    const bottomCanvas = document.createElement("canvas");
    bottomCanvas.width = 8;
    bottomCanvas.height = 8;
    const bottomCtx = bottomCanvas.getContext("2d")!;
    bottomCtx.drawImage(img, 8, 8, 8, 8, 0, 0, 8, 8);

    // 创建用于截取顶层（从 40,8 开始的 8x8 区域）
    const topCanvas = document.createElement("canvas");
    topCanvas.width = 8;
    topCanvas.height = 8;
    const topCtx = topCanvas.getContext("2d")!;
    topCtx.drawImage(img, 40, 8, 8, 8, 0, 0, 8, 8);

    // 合并画布：先绘制底层，再以 source-over 叠加顶层（不混合，直接覆盖不透明像素）
    const mergedCanvas = document.createElement("canvas");
    mergedCanvas.width = 8;
    mergedCanvas.height = 8;
    const mergedCtx = mergedCanvas.getContext("2d")!;

    // 关闭抗锯齿，保持像素精确
    mergedCtx.imageSmoothingEnabled = false;

    // 绘制底层
    mergedCtx.drawImage(bottomCanvas, 0, 0);

    // 叠加顶层：使用 source-over，顶层不透明像素直接覆盖底层，透明区域保留底层
    mergedCtx.globalCompositeOperation = "source-over";
    mergedCtx.drawImage(topCanvas, 0, 0);

    // 将结果转为 HTMLImageElement
    return await canvasToImage(mergedCanvas);
}

// 辅助：加载图片为 HTMLImageElement
function loadImage(url: string): Promise<HTMLImageElement> {
    return new Promise((resolve, reject) => {
        const img = new Image();
        img.crossOrigin = "anonymous"; // 支持跨域
        img.onload = () => resolve(img);
        img.onerror = reject;
        img.src = url;
    });
}

// 辅助：将 canvas 转为 HTMLImageElement
function canvasToImage(canvas: HTMLCanvasElement): Promise<string> {
    return new Promise((resolve) => {
        resolve(canvas.toDataURL("image/png"));
    });
}

const uuidCache = new Map<string, string | null>();

async function getUuidByName(name: string): Promise<string | null> {
    if (uuidCache.has(name)) return uuidCache.get(name)!;
    const response = await fetch(`https://api.mojang.com/users/profiles/minecraft/${name}`);
    const result = response.ok ? ((await response.json()).id ?? null) : null;
    uuidCache.set(name, result);
    return result;
}

// 封装的头像获取函数
async function getPlayerAvatar(uuid: string): Promise<string | null> {

    const response = await fetch(`https://sessionserver.mojang.com/session/minecraft/profile/${uuid}`);
    if (!response.ok) return null;
    const data = await response.json();

    const details = JSON.parse(atob(data.properties[0].value));

    return await overlayImageRegions(details.textures.SKIN.url);
}

async function cachePlayerName(
    playerName: string,
    specifiedUuid: string | null,
    setNameMapping: Dispatch<SetStateAction<Record<string, PlayerData>>>
) {
    console.log(`正在处理玩家: ${playerName}，指定 UUID: ${specifiedUuid}`);
    const onlineUuid = await getUuidByName(playerName);
    const offlineUuid = playerNameToOfflineUUID(playerName);
    const normalizedOnlineUuid = onlineUuid ? normalizeUUID(onlineUuid) : null;
    const normalizedOfflineUuid = normalizeUUID(offlineUuid);
    const normalizedSpecifiedUuid = specifiedUuid ? normalizeUUID(specifiedUuid) : null;
    const avatar = normalizedOnlineUuid ? await getPlayerAvatar(normalizedOnlineUuid) : null;
    const grayscaleAvatar = avatar ? await toGrayscale(avatar) : null;

    setNameMapping(prev => {
        const updates: Record<string, PlayerData> = {};

        if (normalizedOfflineUuid && (!prev[normalizedOfflineUuid] || prev[normalizedOfflineUuid].mode !== "Offline")) {
            updates[normalizedOfflineUuid] = { name: playerName, avatar: grayscaleAvatar, mode: "Offline" };
        }

        if (normalizedOnlineUuid && (!prev[normalizedOnlineUuid] || prev[normalizedOnlineUuid].mode !== "Online")) {
            updates[normalizedOnlineUuid] = { name: playerName, avatar: avatar, mode: "Online" };
        }

        if (normalizedSpecifiedUuid && normalizedSpecifiedUuid !== normalizedOnlineUuid && normalizedSpecifiedUuid !== normalizedOfflineUuid && !prev[normalizedSpecifiedUuid]) {
            updates[normalizedSpecifiedUuid] = { name: playerName, avatar: grayscaleAvatar, mode: "NotMatch" };
        }

        return Object.keys(updates).length > 0 ? { ...prev, ...updates } : prev;
    });
}

export { getPlayerAvatar, getUuidByName, cachePlayerName };
