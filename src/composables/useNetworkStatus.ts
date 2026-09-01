/**
 * useNetworkStatus.ts - 网络状态检测 composable
 *
 * 功能：
 * - 监听浏览器在线/离线事件，实时反映网络连通状态
 * - 定期 ping AUR API 检测 AUR 服务是否可达（独立于浏览器网络状态）
 * - 提供 isOnline（浏览器网络）、isAurOnline（AUR 可达性）两种状态
 *
 * 状态说明：
 * - isOnline: true = 浏览器有网络，false = 无网络
 * - isAurOnline: true = AUR API 可达，false = 不可达，null = 未检测
 */
import { ref, onMounted, onUnmounted } from "vue";

/** 检测间隔（毫秒）—— 离线时不检测，在线时每30秒检测一次 */
const CHECK_INTERVAL_MS = 30_000;

export function useNetworkStatus() {
  /** 浏览器网络状态（对应 navigator.onLine） */
  const isOnline = ref<boolean>(navigator.onLine);
  /** AUR 服务可达性（null 表示尚未检测） */
  const isAurOnline = ref<boolean | null>(null);

  let checkTimer: ReturnType<typeof setInterval> | null = null;
  let pingAbort: AbortController | null = null;

  /**
   * 检测 AUR 连通性
   * 通过 HEAD 请求 AUR RPC API 端点判断服务是否可达
   * @returns true 表示 AUR 可达，false 表示不可达
   */
  async function checkAurConnectivity(): Promise<boolean> {
    // 取消上一次未完成的请求
    pingAbort?.abort();
    pingAbort = new AbortController();

    try {
      const resp = await fetch("https://aur.archlinux.org/rpc/v5", {
        method: "HEAD",
        signal: pingAbort.signal,
        keepalive: true,
      });
      // HTTP 2xx/4xx（API 端点存在但无 body）均视为可达
      return resp.status >= 200 && resp.status < 500;
    } catch {
      return false;
    }
  }

  /**
   * 执行一次 AUR 连通性检测并更新状态
   */
  async function doCheck() {
    if (!isOnline.value) {
      isAurOnline.value = null;
      return;
    }
    const reachable = await checkAurConnectivity();
    isAurOnline.value = reachable;
  }

  /** 启动定期检测定时器 */
  function startPeriodicCheck() {
    stopPeriodicCheck();
    doCheck(); // 立即检测一次
    checkTimer = setInterval(doCheck, CHECK_INTERVAL_MS);
  }

  /** 停止定期检测定时器 */
  function stopPeriodicCheck() {
    if (checkTimer !== null) {
      clearInterval(checkTimer);
      checkTimer = null;
    }
    pingAbort?.abort();
    pingAbort = null;
  }

  /** 在线事件回调 */
  function handleOnline() {
    isOnline.value = true;
    startPeriodicCheck();
  }

  /** 离线事件回调 */
  function handleOffline() {
    isOnline.value = false;
    isAurOnline.value = null;
    stopPeriodicCheck();
  }

  onMounted(() => {
    window.addEventListener("online", handleOnline);
    window.addEventListener("offline", handleOffline);
    if (isOnline.value) {
      startPeriodicCheck();
    }
  });

  onUnmounted(() => {
    stopPeriodicCheck();
    window.removeEventListener("online", handleOnline);
    window.removeEventListener("offline", handleOffline);
  });

  return { isOnline, isAurOnline, checkAurConnectivity };
}
