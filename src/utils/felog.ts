/**
 * felog.ts - 前端诊断日志工具
 *
 * 功能：
 * - 把前端关键时序（应用挂载、路由导航、数据请求）同时输出到浏览器控制台与终端
 * - 终端输出经 IPC 调用后端 frontend_log 命令，由 Rust 用 println! 打到 stdout
 *   （只进终端、不写文件），解决 dev 模式下 WebKitGTK 控制台不可见的问题
 *
 * 日志格式（前端生成本地时间戳，与后端日志同格式便于对照）：
 *   [前端] 2026-08-14 10:24:06.900 DEBUG [Dashboard] loadAll start  (+1234ms)
 * 其中 +1234ms 为相对首次日志的耗时（同一进程内时序），绝对时间戳用于跨进程对照。
 */
// 延迟加载 @tauri-apps/api/core：避免把整条 IPC 依赖链拉入首屏初始模块图。
// 白屏发生在 JS 执行之前，任何被 main.ts/App.vue 顶层 import 的模块都会在该窗口被 WebKitGTK 编译，
// 故此处用动态 import 把 invoke 推迟到首次真正打日志时才加载，不污染首屏图。
type InvokeFn = (cmd: string, args?: Record<string, unknown>) => Promise<unknown>;
let invokeFn: InvokeFn | null = null;
async function sendToTerminal(level: string, area: string, message: string, ts: string): Promise<void> {
  if (!invokeFn) {
    const mod = await import("@tauri-apps/api/core");
    invokeFn = mod.invoke as InvokeFn;
  }
  await invokeFn("frontend_log", { level, area, message, ts });
}

type Level = "DEBUG" | "INFO" | "WARN" | "ERROR";

let t0 = 0;

/** 生成本地时间戳，格式与后端 chrono::Local %Y-%m-%d %H:%M:%S%.3f 一致 */
function localNow(): string {
  const d = new Date();
  const p = (n: number, w = 2) => String(n).padStart(w, "0");
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())} ${p(d.getHours())}:${p(d.getMinutes())}:${p(d.getSeconds())}.${p(d.getMilliseconds(), 3)}`;
}

/**
 * 输出一条前端诊断日志（控制台 + 终端）。
 * @param level - 日志级别
 * @param area - 模块 / 区域标识
 * @param text - 日志正文
 */
export function felog(level: Level, area: string, text: string): void {
  if (t0 === 0) t0 = performance.now();
  const ts = localNow();
  const delta = Math.round(performance.now() - t0);
  const message = `${text}  (+${delta}ms)`;
  const line = `[前端] ${ts} ${level} [${area}] ${message}`;
  // 1) 浏览器控制台（devtools 可见）
  switch (level) {
    case "ERROR":
      console.error(line);
      break;
    case "WARN":
      console.warn(line);
      break;
    default:
      console.log(line);
  }
  // 2) 终端（经 IPC 转发到 Rust stdout，仅终端不写文件）。失败静默回退到 console。
  sendToTerminal(level, area, message, ts).catch(() => {
    /* 转发失败不影响前端运行，忽略 */
  });
}

// 便捷级别函数
export const feDebug = (area: string, text: string) => felog("DEBUG", area, text);
export const feInfo = (area: string, text: string) => felog("INFO", area, text);
export const feWarn = (area: string, text: string) => felog("WARN", area, text);
export const feError = (area: string, text: string) => felog("ERROR", area, text);
