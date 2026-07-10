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
  formLicenseId: Ref<number | null>,
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

  function selectLicense(licId: number | null) {
    formLicenseId.value = licId;
    licenseDropdownOpen.value = false;
    licenseSearch.value = "";
  }

  function getSelectedLicenseLabel(): string {
    if (formLicenseId.value === null) return "未设置";
    const lic = licenses.value.find((l: License) => l.id === formLicenseId.value);
    return lic ? `${lic.spdx_id} — ${lic.full_name}` : "未设置";
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