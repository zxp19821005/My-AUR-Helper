<!--
  CacheManager.vue - 缓存管理页面

  功能：
  - 扫描指定缓存目录中的 .pkg.tar.zst 包文件
  - 显示扫描结果

  数据来源：
  - scan_pkg_files_cmd: 扫描包文件
  - get_setting: 获取缓存目录设置
-->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { CachePackage, Setting } from "../types";
import { useToolbarStore } from "../stores/toolbar";
import PageToolbar from "../components/PageToolbar.vue";

const toolbar = useToolbarStore();

/** 缓存目录 */
const cacheDir = ref("");

/** 扫描到的包文件列表 */
const packages = ref<CachePackage[]>([]);

/** 扫描中 */
const scanning = ref(false);

/** 加载设置 */
onMounted(async () => {
  try {
    const setting = await invoke<Setting | null>("get_setting", { key: "backup_dir" });
    if (setting) cacheDir.value = setting.value;
  } catch { /* ignore */ }
  toolbar.setInfo(`缓存目录: ${cacheDir.value || "未设置"}`);
});

/** 扫描缓存目录 */
async function scanCache() {
  if (!cacheDir.value) return;
  scanning.value = true;
  toolbar.setProgress(0, 1);
  try {
    packages.value = await invoke<CachePackage[]>("scan_pkg_files_cmd", { directory: cacheDir.value });
    toolbar.setInfo(`缓存目录: ${cacheDir.value}  |  找到 ${packages.value.length} 个包文件`);
  } catch { /* ignore */ }
  scanning.value = false;
  toolbar.clearProgress();
}

function formatSize(bytes: number): string {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return (bytes / Math.pow(1024, i)).toFixed(1) + " " + units[i];
}
</script>

<template>
  <div>
    <PageToolbar>
      <input v-model="cacheDir" placeholder="缓存目录路径" class="input" style="width: 320px" />
      <button class="btn btn-primary" @click="scanCache" :disabled="scanning || !cacheDir">
        {{ scanning ? "扫描中..." : "扫描缓存" }}
      </button>
    </PageToolbar>

    <div v-if="packages.length" class="card">
      <h3>缓存文件 ({{ packages.length }})</h3>
      <div style="margin-top: 0.5rem; display: flex; flex-wrap: wrap; gap: 0.5rem">
        <div v-for="pkg in packages" :key="pkg.filename"
          style="padding: 0.5rem; border: 1px solid var(--border); border-radius: 6px; min-width: 220px; font-size: 0.8125rem">
          <strong>{{ pkg.name }}</strong>
          <div style="color: var(--text-secondary); margin-top: 0.25rem">
            {{ pkg.version }}-{{ pkg.pkgrel }} · {{ pkg.arch }}
            <span v-if="pkg.epoch"> (epoch: {{ pkg.epoch }})</span>
          </div>
          <div style="color: var(--text-secondary); font-size: 0.75rem">
            {{ formatSize(pkg.size) }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
