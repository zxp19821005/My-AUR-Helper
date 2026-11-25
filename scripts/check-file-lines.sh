#!/usr/bin/env bash
# check-file-lines.sh - 源码文件 300 行上限校验脚本
#
# 功能：统计 src/ 与 src-tauri/src/ 下 .rs / .vue / .ts 文件的行数，列出所有
#       超过 300 行（wc -l，含空行与注释）的文件，并区分「豁免」与「违规」。
# 工作流程：
#   1. find 收集源码文件，排除 target / node_modules / dist / .cargo / .rustup；
#   2. 逐文件 wc -l，≤300 行跳过；
#   3. 超限文件依次判定两类豁免：
#      a. src/bindings.ts —— tauri-specta 生成物（AGENTS.md 已登记豁免）；
#      b. 文件头 5 行内含 @LINE_LIMIT_EXEMPT 标注 —— 纯数据表 / 测试文件；
#   4. 存在任一非豁免超限文件则 exit 1；否则打印 OK 行并 exit 0。
# 适用场景：提交前本地自检、CI 门禁、以及多 AI 协作时防止文件规模再次失控。
# 用法：bash scripts/check-file-lines.sh
#   - exit 0：无违规（可存在豁免文件），打印 "OK: 仅 bindings.ts 豁免"
#   - exit 1：存在未豁免的超限文件，逐条打印 [超限] 前缀
# 注意：新增豁免请使用文件头 @LINE_LIMIT_EXEMPT 标注并注明理由，勿扩大生成物豁免面。

set -u

readonly LINE_LIMIT=300
readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
readonly GENERATED_EXEMPT="src/bindings.ts"

cd "${REPO_ROOT}" || exit 1

# 判断文件头 5 行内是否带有豁免标注（纯 bash 读取，避免管道 SIGPIPE）
has_exempt_marker() {
  local file="$1"
  local line
  local i=0
  while IFS= read -r line && (( i < 5 )); do
    if [[ "${line}" == *'@LINE_LIMIT_EXEMPT'* ]]; then
      return 0
    fi
    i=$((i + 1))
  done < "${file}"
  return 1
}

declare -a exempted=()
declare -a violations=()

while IFS= read -r -d '' file; do
  local_lines="$(wc -l < "${file}")"
  local_lines=$((local_lines))
  if (( local_lines <= LINE_LIMIT )); then
    continue
  fi
  if [[ "${file}" == "${GENERATED_EXEMPT}" || "${file}" == "./${GENERATED_EXEMPT}" ]]; then
    exempted+=("${file} (${local_lines} 行) [生成物豁免]")
    continue
  fi
  if has_exempt_marker "${file}"; then
    exempted+=("${file} (${local_lines} 行) [@LINE_LIMIT_EXEMPT 标注]")
    continue
  fi
  violations+=("${file} (${local_lines} 行)")
done < <(
  find src src-tauri/src -type f \( -name '*.rs' -o -name '*.vue' -o -name '*.ts' \) \
    -not -path '*/target/*' \
    -not -path '*/node_modules/*' \
    -not -path '*/dist/*' \
    -not -path '*/.cargo/*' \
    -not -path '*/.rustup/*' \
    -print0
)

for item in "${exempted[@]}"; do
  printf '%s\n' "${item}"
done

if (( ${#violations[@]} > 0 )); then
  for item in "${violations[@]}"; do
    printf '[超限] %s\n' "${item}"
  done
  exit 1
fi

printf 'OK: 仅 bindings.ts 豁免\n'
exit 0
