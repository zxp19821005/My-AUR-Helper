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
import { computed, onMounted, ref, watch } from "vue";                   // Vue 核心 API
import { useRouter } from "vue-router";                      // 路由 API：用于跳转到详情页
import { invoke } from "@tauri-apps/api/core";                // Tauri IPC 调用
import { usePackageStore } from "../stores/packages";         // 软件包状态 Store
import { useToolbarStore } from "../stores/toolbar";
import PageToolbar from "../components/PageToolbar.vue";

const router = useRouter();
const pkgStore = usePackageStore();
const toolbar = useToolbarStore();

const pageSize = 50;
const currentPage = ref(1);

onMounted(() => {
  pkgStore.fetchPackages();
});

/** 检查器类型显示文本映射 */
const checkerText: Record<string, string> = {
  github_release: "GitHub Release",
  github_tag: "GitHub Tag",
  gitee: "Gitee",
  gitlab: "GitLab",
  redirect: "重定向",
  http: "HTTP",
  manual: "手动",
};

/** 检查所有软件包的上游版本 */
async function checkAll() {
  pkgStore.loading = true;
  toolbar.setProgress(0, 1);
  try {
    await invoke("check_all_upstream");
    await pkgStore.fetchPackages();
  } finally {
    pkgStore.loading = false;
    toolbar.clearProgress();
  }
}

/** 统计摘要 */
const summary = computed(() => {
  const p = pkgStore.packages;
  return {
    total: p.length,
    outdated: p.filter((x) => x.is_outdated).length,
    upToDate: p.filter((x) => !x.is_outdated).length,
  };
});

/** 当前页数据 */
const pageData = computed(() => {
  const start = (currentPage.value - 1) * pageSize;
  return pkgStore.packages.slice(start, start + pageSize);
});

/** 跳转到指定页 */
function goToPage(page: number) {
  currentPage.value = page;
}

/** 同步统计到底部工具栏 */
watch(summary, (s) => {
  toolbar.setInfo(`总计: ${s.total}  |  已最新: ${s.upToDate}  |  需更新: ${s.outdated}`);
  toolbar.setPagination(s.total, currentPage.value, pageSize, goToPage);
}, { immediate: true });
watch(currentPage, (p) => {
  toolbar.setPagination(summary.value.total, p, pageSize, goToPage);
});
</script>

<template>
  <div>
    <PageToolbar>
      <button class="btn btn-primary" @click="checkAll" :disabled="pkgStore.loading">
        {{ pkgStore.loading ? "检查中..." : "检查全部更新" }}
      </button>
    </PageToolbar>

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
          <tr v-for="pkg in pageData" :key="pkg.pkgname" @click="router.push(`/packages/${pkg.pkgname}`)">
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
