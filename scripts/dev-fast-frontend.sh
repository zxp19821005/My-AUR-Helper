#!/usr/bin/env bash
# dev-fast-frontend.sh - 快速开发模式的前端服务
#
# 功能：
# - 首次完整构建（保证 preview 启动时 dist 产物存在）
# - vite build --watch 监听源码变化增量重建
# - vite preview 在 4173 端口提供打包产物服务
#
# 背景：
# Tauri dev 默认走 Vite dev server（不打包），WebKitGTK 首次进入路由时
# 需逐个加载编译数十~数百个零散 ESM 模块，在内存压力大的系统上页面
# 切换可达 15-30 秒。本脚本改用打包产物（~20 个 chunk），加载速度接近
# release 版。代价：无 HMR 热重载，改前端代码后需手动刷新（Ctrl+R）。
set -euo pipefail

cd "$(dirname "$0")/.."

# 进程组清理：Tauri dev 退出时终止本脚本的全部子进程
trap 'kill 0' EXIT

echo "[dev-fast] 首次构建前端产物..."
pnpm vite build

echo "[dev-fast] 启动 watch 增量重建 + preview (http://localhost:4173)"
pnpm vite build --watch &
pnpm vite preview --port 4173 --strictPort &
wait
