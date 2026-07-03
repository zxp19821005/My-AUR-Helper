#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"
clear

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BLUE='\033[0;34m'
NC='\033[0m'

info()  { echo -e "${CYAN}[INFO]${NC}  $1"; }
ok()    { echo -e "${GREEN}[OK]${NC}    $1"; }
warn()  { echo -e "${YELLOW}[WARN]${NC}  $1"; }
err()   { echo -e "${RED}[ERROR]${NC} $1"; }

# ---------- Cache 加速 ----------

export RUSTC_WRAPPER="${RUSTC_WRAPPER:-sccache}"
export CC="${CC:-ccache gcc}"
export CXX="${CXX:-ccache g++}"

# 确保 sccache 守护进程在运行
ensure_sccache() {
    if command -v sccache &>/dev/null; then
        sccache --start-server &>/dev/null || true
    fi
}

# 显示 sccache 统计
show_cache_stats() {
    if command -v sccache &>/dev/null; then
        local blue="${BLUE}" nc="${NC}"
        sccache --show-stats 2>/dev/null | \
            awk -v blue="$blue" -v nc="$nc" '
                /^Cache hits[ \t]+[0-9]/ { hits=$3 }
                /^Cache misses[ \t]+[0-9]/ { misses=$3 }
                /^Cache hits rate \(Rust\)/ { rate=$5 }
                END {
                    printf "  %s%s%s hits=%s misses=%s rust_rate=%s\n", blue, "sccache:", nc, hits?hits:0, misses?misses:0, rate?rate:"-"
                }'
    fi
}

cache_help() {
    cat <<EOF
环境变量:
  RUSTC_WRAPPER=sccache   Rust 编译缓存（默认启用，设空值可禁用）
  CC="ccache gcc"         C 编译缓存
  CXX="ccache g++"        C++ 编译缓存

手动清理缓存:
  sccache --clear
  ccache --clear
EOF
}

usage() {
    cat <<EOF
用法: ./scripts/dev.sh <command>

命令:
  dev         启动开发模式（Tauri dev，前端热重载 + 后端热重载）
  build       生产构建（pnpm build + tauri build）
  check       类型检查（vue-tsc + cargo check）
  lint        Lint 检查（eslint + cargo clippy）
  stats       显示编译缓存统计
  clean       清理构建产物
EOF
    exit 0
}

ensure_sccache

case "${1:-help}" in
    dev)
        info "启动 Tauri 开发模式（前端热重载 + 后端热重载）${RUSTC_WRAPPER:+| 缓存: $RUSTC_WRAPPER}"
        pnpm tauri dev
        ok "开发服务器已关闭"
        show_cache_stats
        ;;

    build)
        info "检查前端类型..."
        pnpm vue-tsc --noEmit
        ok "前端类型检查通过"

        info "构建前端..."
        pnpm vite build
        ok "前端构建完成"

        info "构建 Tauri 生产版本...${RUSTC_WRAPPER:+ 缓存: $RUSTC_WRAPPER}"
        pnpm tauri build
        ok "生产构建完成，产物在 src-tauri/target/release/"
        show_cache_stats
        ;;

    check)
        info "运行前端类型检查 (vue-tsc)..."
        pnpm vue-tsc --noEmit
        ok "前端类型检查通过"

        info "运行后端类型检查 (cargo check)${RUSTC_WRAPPER:+ 缓存: $RUSTC_WRAPPER}..."
        cargo check --manifest-path src-tauri/Cargo.toml
        ok "后端类型检查通过"
        show_cache_stats
        ;;

    lint)
        info "运行前端 Lint (eslint)..."
        pnpm lint
        ok "前端 Lint 通过"

        info "运行后端 Lint (cargo clippy)..."
        cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings 2>/dev/null || \
        cargo clippy --manifest-path src-tauri/Cargo.toml
        ok "后端 Lint 通过"
        ;;

    stats)
        echo -e "${CYAN}sccache 统计:${NC}"
        if command -v sccache &>/dev/null; then
            sccache --show-stats 2>&1
        else
            warn "sccache 未安装"
        fi
        echo
        echo -e "${CYAN}ccache 统计:${NC}"
        if command -v ccache &>/dev/null; then
            ccache --show-stats 2>&1
        else
            warn "ccache 未安装"
        fi
        ;;

    clean)
        info "清理 sccache 缓存..."
        sccache --clear &>/dev/null || true

        info "清理前端构建产物..."
        rm -rf dist

        info "清理后端构建产物..."
        cargo clean --manifest-path src-tauri/Cargo.toml

        info "清理 node_modules 缓存..."
        rm -rf node_modules/.vite

        ok "清理完成"
        ;;

    help|*)
        usage
        ;;
esac
