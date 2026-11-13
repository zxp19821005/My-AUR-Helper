#!/usr/bin/env bash
# install-dev-icon.sh — 为 dev 模式安装 Linux 桌面项与 hicolor 图标
# 面板任务栏按钮通过 WM_CLASS 匹配 .desktop，再由 Icon= 从图标主题取图；
# dev 模式未安装打包产物，故需手工补齐这两项。
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
ICON_SRC="$ROOT_DIR/src-tauri/icons"
APP_ID="my-aur-helper"
BIN="$ROOT_DIR/src-tauri/target/debug/$APP_ID"
DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
DESKTOP_DIR="$DATA_HOME/applications"
ICON_ROOT="$DATA_HOME/icons/hicolor"
DESKTOP_FILE="$DESKTOP_DIR/$APP_ID.desktop"

if [[ ! -f "$ICON_SRC/icon.png" ]]; then
	echo "错误: 找不到图标源文件 $ICON_SRC/icon.png" >&2
	exit 1
fi

# 安装各尺寸图标到 hicolor 主题（尺寸与 src-tauri/icons 中已有资源对应）
install_one() {
	local size="$1" src="$2"
	local dest="$ICON_ROOT/${size}x${size}/apps/$APP_ID.png"
	install -Dm644 "$src" "$dest"
	echo "  $dest"
}

echo "安装 hicolor 图标:"
install_one 16 "$ICON_SRC/16x16.png"
install_one 24 "$ICON_SRC/24x24.png"
install_one 32 "$ICON_SRC/32x32.png"
install_one 48 "$ICON_SRC/48x48.png"
install_one 64 "$ICON_SRC/64x64.png"
install_one 96 "$ICON_SRC/96x96.png"
install_one 128 "$ICON_SRC/128x128.png"
install_one 256 "$ICON_SRC/256x256.png"
install_one 512 "$ICON_SRC/icon.png"

# 生成 .desktop（Exec 指向本机 dev 二进制，生产打包由 tauri bundle 自行生成）
install -d "$DESKTOP_DIR"
cat > "$DESKTOP_FILE" <<EOF
[Desktop Entry]
Type=Application
Version=1.0
Name=My AUR Helper
GenericName=AUR Package Manager
Comment=AUR 软件包管理工具
Exec="$BIN"
Icon=$APP_ID
Terminal=false
Categories=System;
StartupWMClass=$APP_ID
StartupNotify=true
EOF
echo "已生成: $DESKTOP_FILE"

# 校验与刷新缓存
if command -v desktop-file-validate >/dev/null 2>&1; then
	desktop-file-validate "$DESKTOP_FILE" && echo "desktop-file-validate: 通过"
fi
if command -v update-desktop-database >/dev/null 2>&1; then
	update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
	gtk-update-icon-cache -f -t "$ICON_ROOT" >/dev/null 2>&1 || true
fi

echo "完成。重启应用（或执行 xfce4-panel -r）后面板任务栏按钮即可显示新图标。"
