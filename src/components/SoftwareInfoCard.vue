<script setup lang="ts">
import { computed } from "vue";
import type { SoftwareDetail, Language } from "../types";
import { pkgTypeOptions, checkerTypeOptions } from "../utils/enums";
import StandardizedCard from "./base/StandardizedCard.vue";
import StandardizedBadge from "./base/StandardizedBadge.vue";

const props = defineProps<{
  detail: SoftwareDetail;
}>();

const pkgTypeName = computed(() => {
  return pkgTypeOptions.find(t => t.id === props.detail.package_type_id)?.label || '未知';
});

const checkerTypeName = computed(() => {
  return checkerTypeOptions.find(c => c.id === props.detail.checker_type_id)?.label || '未知';
});

const languages = ref<Language[]>([]);

onMounted(async () => {
  try {
    languages.value = await invoke<Language[]>("get_languages");
  } catch {
    // ignore
  }
});

function getLanguageNames(ids: number[] | null | undefined): string {
  if (!ids || ids.length === 0) return '—';
  return ids
    .map(id => languages.value.find(l => l.id === id)?.name)
    .filter(Boolean)
    .join(', ') || '—';
}

function formatTimestamp(ts: number | null): string {
  if (!ts) return "—";
  return new Date(ts * 1000).toLocaleDateString("zh-CN", {
    year: "numeric", month: "2-digit", day: "2-digit",
  });
}

const statusOptions = computed(() => [
  { value: "latest", text: "已最新", className: "status-badge-success" },
  { value: "outdated", text: "需更新", className: "status-badge-warning" },
]);
</script>

<template>
  <StandardizedCard
    :title="detail.pkgname"
    :subtitle="detail.aur_pkgdesc || '暂无描述'"
    :status="detail.is_outdated ? 'outdated' : 'latest'"
    :statusOptions="statusOptions"
    layout="table"
  >
    <div class="badge-row">
      <span class="type-tag">{{ pkgTypeName }}</span>
      <span class="type-tag">{{ checkerTypeName }}</span>
    </div>
    
    <table class="info-table">
      <tbody>
        <tr>
          <td class="label">上游地址</td>
          <td class="value url-value">
            <a v-if="detail.upstream_url" :href="detail.upstream_url" target="_blank">
              {{ detail.upstream_url }}
            </a>
            <span v-else>未设置</span>
          </td>
        </tr>
        <tr>
          <td class="label">版本提取关键字</td>
          <td class="value">
            <code v-if="detail.version_extract_regex">{{ detail.version_extract_regex }}</code>
            <span v-else class="empty">未设置</span>
          </td>
        </tr>
        <tr>
          <td class="label">编程语言</td>
          <td class="value">{{ getLanguageNames(detail.language_ids) }}</td>
        </tr>
        <tr>
          <td class="label">AUR 版本</td>
          <td class="value">{{ detail.aur_version || '—' }}</td>
        </tr>
        <tr>
          <td class="label">上游版本</td>
          <td class="value">{{ detail.upstream_version || '—' }}</td>
        </tr>
        <tr>
          <td class="label">AUR 最后更新</td>
          <td class="value">{{ formatTimestamp(detail.aur_last_updated) }}</td>
        </tr>
        <tr>
          <td class="label">上游最后检查</td>
          <td class="value">{{ formatTimestamp(detail.upstream_last_checked) }}</td>
        </tr>
      </tbody>
    </table>
  </StandardizedCard>
</template>