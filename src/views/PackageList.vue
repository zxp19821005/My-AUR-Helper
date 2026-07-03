<!--
  PackageList.vue - 软件包列表页面

  功能：
  - 显示所有软件包列表
  - 显示统计摘要（总数、已最新、需更新）
  - 支持检查全部更新
  - 点击行可查看软件包详情

  数据来源：
  - packages store: 软件包列表
  - check_all_upstream: 检查所有上游版本
-->
<script setup lang="ts">
import { computed, onMounted } from "vue";                   // Vue 核心 API
import { useRouter } from "vue-router";                      // 路由 API：用于跳转到详情页
import { invoke } from "@tauri-apps/api/core";                // Tauri IPC 调用
import { usePackageStore } from "../stores/packages";         // 软件包状态 Store
import PageToolbar from "../components/PageToolbar.vue";

const router = useRouter();        // 路由实例
const pkgStore = usePackageStore(); // 软件包 Store 实例

/** 组件挂载时获取软件包列表 */
onMounted(() => {
  pkgStore.fetchPackages();
});

/** 检查器类型显示文本映射 - 将后端返回的类型值转换为可读的中英文名称 */
const checkerText: Record<string, string> = {
  github_release: "GitHub Release",
  github_tag: "GitHub Tag",
  gitee: "Gitee",
  gitlab: "GitLab",
  redirect: "重定向",
  http: "HTTP",
  manual: "手动",
};

/** 检查所有软件包的上游版本 - 批量触发版本检查并刷新列表 */
async function checkAll() {
  pkgStore.loading = true;
  try {
    await invoke("check_all_upstream");           // 调用后端批量检查命令
    await pkgStore.fetchPackages();                // 刷新列表获取最新状态
  } finally {
    pkgStore.loading = false;
  }
}

/** 统计摘要 - 通过计算属性实时汇总软件包数据 */
const summary = computed(() => {
  const p = pkgStore.packages;
  return {
    total: p.length,                            // 总数量
    outdated: p.filter((x) => x.is_outdated).length,   // 需要更新的数量
    upToDate: p.filter((x) => !x.is_outdated).length,  // 已最新的数量
  };
});
</script>

<template>
  <div>
    <PageToolbar>
      <button class="btn btn-primary" @click="checkAll" :disabled="pkgStore.loading">
        {{ pkgStore.loading ? "检查中..." : "检查全部更新" }}
      </button>
    </PageToolbar>

    <!-- 统计摘要行 - 显示总计、已最新、需更新的数量 -->
    <div style="display: flex; gap: 1rem; margin-bottom: 1rem; font-size: 0.875rem; color: var(--text-secondary)">
      <span>总计: {{ summary.total }}</span>
      <span style="color: var(--success)">已最新: {{ summary.upToDate }}</span>
      <span style="color: var(--warning)">需更新: {{ summary.outdated }}</span>
    </div>

    <!-- 软件包表格 - 显示所有软件包的简要信息 -->
    <div class="card">
      <table class="pkg-table">
        <thead>
          <tr>
            <th>包名</th>
            <th>类型</th>
            <th>检查器</th>
            <th>状态</th>
            <th>操作</th>
          </tr>
        </thead>
        <tbody>
          <!-- 每行对应一个软件包，点击整行跳转到详情页 -->
          <tr v-for="pkg in pkgStore.packages" :key="pkg.pkgname" @click="router.push(`/packages/${pkg.pkgname}`)">
            <td><strong>{{ pkg.pkgname }}</strong></td>
            <td>{{ pkg.package_type_id }}</td>
            <td>{{ checkerText[pkg.checker_type_id] || pkg.checker_type_id }}</td>
            <td>
              <!-- 状态徽章 - 根据是否过期显示不同颜色 -->
              <span class="status-badge" :class="pkg.is_outdated ? 'status-update_available' : 'status-up_to_date'">
                {{ pkg.is_outdated ? "需更新" : "已最新" }}
              </span>
            </td>
            <td>
              <!-- 检查按钮 - 单独检查此包，使用 stopPropagation 防止触发行跳转 -->
              <button class="btn btn-outline" style="padding: 0.25rem 0.5rem; font-size: 0.75rem"
                @click.stop="pkgStore.checkVersion(pkg.pkgname)">
                检查
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<style scoped>
/* 软件包表格样式 - 全宽表格 */
.pkg-table {
  width: 100%;
  border-collapse: collapse;
}

/* 表头样式 - 灰色小号文字 */
.pkg-table th {
  text-align: left;
  padding: 0.75rem;
  color: var(--text-secondary);
  font-weight: 600;
  font-size: 0.75rem;
  text-transform: uppercase;
  border-bottom: 1px solid var(--border);
}

/* 单元格样式 */
.pkg-table td {
  padding: 0.75rem;
  border-bottom: 1px solid var(--border);
  font-size: 0.875rem;
}

/* 行悬停效果 */
.pkg-table tbody tr {
  cursor: pointer;
  transition: background-color 0.15s;
}

.pkg-table tbody tr:hover {
  background-color: rgba(108, 99, 255, 0.05);
}
</style>
