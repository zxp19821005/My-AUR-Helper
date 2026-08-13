/**
 * usePopupWindow.ts - 打开独立 Tauri 子窗口的共享逻辑
 *
 * 功能：
 * - openPopup(label, url, title)：打开（或激活）一个独立窗口
 * - 若窗口已存在则取消最小化 → 显示 → 聚焦；否则创建新窗口
 * - Dashboard 与 PageToolbar 共用，避免重复实现
 *
 * 依赖：
 * - @tauri-apps/api/webviewWindow：Tauri 窗口 API
 */
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

/**
 * 打开（或激活）一个独立 Tauri 子窗口。
 *
 * 逻辑：
 * 1. 若窗口已存在（getByLabel 命中），先 unminimize 再 show + setFocus 重新激活，直接 return。
 * 2. 若窗口不存在才创建新窗口；创建后监听 tauri://error，极少数竞态下（label 已存在）
 *    补一次聚焦，避免窗口“创建但不可见”。
 *
 * @param label 窗口唯一标识（与 tauri.conf.json 中注册的 label 对应）
 * @param url 窗口内加载的路由路径
 * @param title 窗口标题
 */
export async function openPopup(label: string, url: string, title: string) {
  // 1) 窗口已存在：取消最小化 -> 显示 -> 聚焦（再次点击切换/激活窗口）
  try {
    const existing = await WebviewWindow.getByLabel(label);
    if (existing) {
      try {
        await existing.unminimize();
      } catch {
        // 部分平台不支持 unminimize，忽略
      }
      try {
        await existing.show();
        await existing.setFocus();
      } catch {
        // show/setFocus 失败，窗口可能已销毁，继续创建新窗口
      }
      return;
    }
  } catch {
    // getByLabel 失败，继续创建新窗口
  }

  // 2) 创建新窗口；duplicate-label 错误通过异步事件发出，不会被下方 try/catch 捕获
  try {
    const win = new WebviewWindow(label, {
      url,
      title,
      width: 900,
      height: 600,
      resizable: true,
      center: true,
    });
    // 极少数竞态下 label 已存在，create 会异步报错，这里补一次聚焦
    win.once("tauri://error", async () => {
      try {
        const existing = await WebviewWindow.getByLabel(label);
        if (existing) {
          try { await existing.unminimize(); } catch { /* ignore */ }
          await existing.show();
          await existing.setFocus();
        }
      } catch {
        // 忽略
      }
    });
  } catch (error) {
    console.error(`打开窗口 "${label}" 失败:`, error);
  }
}
