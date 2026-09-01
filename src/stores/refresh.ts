/**
 * refresh.ts - 全局刷新事件总线
 *
 * 用途：
 * - 提供一个轻量级的全局事件触发器
 * - TabBar 的"刷新"按钮触发全局 refresh 事件
 * - 各路由页面订阅 refresh 事件并执行自身的 fetchView
 *
 * 设计原因：
 * - 不使用 provide/inject 是因为 TabBar 和 RouterView 跨层级太远
 * - 不使用 mitt/event-bus 是因为避免引入新依赖
 * - Pinia 已经在项目中使用，且天然跨组件共享
 */
import { defineStore } from "pinia";
import { ref, onUnmounted } from "vue";

export const useRefreshStore = defineStore("refresh", () => {
  // 当前活跃的路由路径 - 用于区分不同页面的刷新
  const activePath = ref<string>("");

  // 订阅者回调映射：path -> Set<callback>
  const subscribers = new Map<string, Set<() => void | Promise<void>>>();

  /**
   * 订阅当前路由的刷新事件
   * @param handler - 刷新回调函数
   * @returns unsubscribe 函数
   */
  function subscribe(handler: () => void | Promise<void>) {
    const path = activePath.value;
    if (!path) return () => {};
    if (!subscribers.has(path)) {
      subscribers.set(path, new Set());
    }
    subscribers.get(path)!.add(handler);
    return () => {
      subscribers.get(path)?.delete(handler);
    };
  }

  /**
   * 触发当前路由的刷新事件
   */
  async function trigger() {
    const path = activePath.value;
    if (!path) return;
    const handlers = subscribers.get(path);
    if (!handlers) return;
    // 并发执行所有订阅者
    await Promise.allSettled([...handlers].map((h) => h()));
  }

  /**
   * 设置当前活跃路由
   * 由 App.vue 监听路由变化时调用
   */
  function setActivePath(path: string) {
    activePath.value = path;
  }

  return { activePath, subscribe, trigger, setActivePath };
});

/**
 * 在组件中订阅当前路由的刷新事件
 * @returns unsubscribe 函数
 */
export function useRefreshSubscription(handler: () => void | Promise<void>) {
  const store = useRefreshStore();
  const unsubscribe = store.subscribe(handler);
  onUnmounted(() => {
    unsubscribe();
  });
  return unsubscribe;
}
