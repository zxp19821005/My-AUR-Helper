/**
 * useLicenseSelect.ts - License 可搜索下拉框逻辑
 *
 * 功能：
 * - 管理 License 搜索过滤
 * - 控制下拉框开关和外部点击关闭
 * - 提供选中 License 的显示标签
 */
import { ref, computed, onMounted, onUnmounted, type Ref } from "vue";
import type { License } from "../types";

export function useLicenseSelect(
  licenses: Ref<License[]>,
  formLicenseIds: Ref<string | null>,
  searchableSelectRef: Ref<HTMLElement | null>
) {
  const licenseSearch = ref("");
  const licenseDropdownOpen = ref(false);

  const filteredLicenses = computed(() => {
    if (!licenseSearch.value.trim()) return licenses.value;
    const keyword = licenseSearch.value.toLowerCase();
    return licenses.value.filter(
      (lic: License) =>
        lic.spdx_id.toLowerCase().includes(keyword) ||
        lic.full_name.toLowerCase().includes(keyword)
    );
  });

  function selectLicense(licSpdxId: string | null) {
    if (licSpdxId === null) {
      // 清空所有 license
      formLicenseIds.value = null;
    } else {
      // 切换单个 license 的选中状态
      let currentList: string[] = [];
      if (formLicenseIds.value) {
        try {
          const parsed = JSON.parse(formLicenseIds.value);
          if (Array.isArray(parsed)) {
            currentList = parsed;
          }
        } catch {
          // Not valid JSON, ignore
        }
      }
      
      const idx = currentList.indexOf(licSpdxId);
      if (idx >= 0) {
        currentList.splice(idx, 1);
      } else {
        currentList.push(licSpdxId);
      }
      
      formLicenseIds.value = currentList.length > 0 ? JSON.stringify(currentList) : null;
    }
    
    licenseDropdownOpen.value = false;
    licenseSearch.value = "";
  }

  function getSelectedLicenseLabel(): string {
    if (!formLicenseIds.value) return "未设置";
    // Try to parse JSON array and display
    try {
      const parsed = JSON.parse(formLicenseIds.value);
      if (Array.isArray(parsed)) {
        if (parsed.length === 0) return "未设置";
        return parsed.join(", ");
      }
    } catch {
      // Not JSON, return as is
      return formLicenseIds.value;
    }
    return "未设置";
  }

  function handleClickOutside(event: MouseEvent) {
    if (searchableSelectRef.value && !searchableSelectRef.value.contains(event.target as Node)) {
      licenseDropdownOpen.value = false;
    }
  }

  onMounted(() => {
    document.addEventListener("click", handleClickOutside);
  });

  onUnmounted(() => {
    document.removeEventListener("click", handleClickOutside);
  });

  return {
    licenseSearch,
    licenseDropdownOpen,
    filteredLicenses,
    selectLicense,
    getSelectedLicenseLabel,
  };
}