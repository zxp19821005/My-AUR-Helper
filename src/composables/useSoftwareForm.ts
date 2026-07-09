/**
 * useSoftwareForm.ts - 软件包表单逻辑
 *
 * 提供软件包添加/编辑表单的状态管理和操作
 */
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { SoftwareDetail, License, Language } from "../types";
import { pkgTypeOptions, checkerTypeOptions } from "../utils/enums";

export interface SoftwareForm {
  pkgname: string;
  upstream_url: string;
  package_type_id: number;
  checker_type_id: number;
  is_outdated: boolean;
  check_test_versions: boolean;
  check_binary_files: boolean;
  auto_check_enabled: boolean;
  license_id: number | null;
  language_id: number | null;
  version_extract_regex: string;
}

const defaultForm: SoftwareForm = {
  pkgname: "",
  upstream_url: "",
  package_type_id: 1,
  checker_type_id: 1,
  is_outdated: false,
  check_test_versions: false,
  check_binary_files: false,
  auto_check_enabled: false,
  license_id: null,
  language_id: null,
  version_extract_regex: "",
};

export const pkgTypes = pkgTypeOptions;
export const checkerTypes = checkerTypeOptions;

export function useSoftwareForm() {
  const saving = ref(false);
  const error = ref("");
  const detail = ref<SoftwareDetail | null>(null);
  const licenses = ref<License[]>([]);
  const languages = ref<Language[]>([]);
  const form = ref<SoftwareForm>({ ...defaultForm });

  function isDirty(original: SoftwareDetail | null): boolean {
    if (!original) return false;
    return (
      form.value.upstream_url !== (original.upstream_url ?? "") ||
      form.value.package_type_id !== original.package_type_id ||
      form.value.checker_type_id !== original.checker_type_id ||
      form.value.is_outdated !== original.is_outdated ||
      form.value.check_test_versions !== original.check_test_versions ||
      form.value.check_binary_files !== original.check_binary_files ||
      form.value.auto_check_enabled !== original.auto_check_enabled ||
      form.value.license_id !== original.license_id ||
      form.value.language_id !== original.language_id ||
      form.value.version_extract_regex !== (original.version_extract_regex ?? "")
    );
  }

  function canSave(mode: string, dirty: boolean): boolean {
    if (mode === "add") return form.value.pkgname.trim().length > 0;
    return dirty;
  }

  async function loadEnums() {
    try {
      licenses.value = await invoke<License[]>("get_licenses");
      languages.value = await invoke<Language[]>("get_languages");
    } catch {
      // ignore
    }
  }

  async function loadSoftware(pkgname: string): Promise<boolean> {
    try {
      const data = await invoke<SoftwareDetail | null>("get_software_detail", { pkgname });
      detail.value = data;
      if (data) {
        form.value = {
          pkgname: data.pkgname,
          upstream_url: data.upstream_url ?? "",
          package_type_id: data.package_type_id,
          checker_type_id: data.checker_type_id,
          is_outdated: data.is_outdated,
          check_test_versions: data.check_test_versions,
          check_binary_files: data.check_binary_files,
          auto_check_enabled: data.auto_check_enabled,
          license_id: data.license_id,
          language_id: data.language_id,
          version_extract_regex: data.version_extract_regex ?? "",
        };
      }
      return true;
    } catch (e) {
      error.value = String(e);
      return false;
    }
  }

  function resetForm(mode: string) {
    if (mode === "edit" && detail.value) {
      form.value = {
        pkgname: detail.value.pkgname,
        upstream_url: detail.value.upstream_url ?? "",
        package_type_id: detail.value.package_type_id,
        checker_type_id: detail.value.checker_type_id,
        is_outdated: detail.value.is_outdated,
        check_test_versions: detail.value.check_test_versions,
        check_binary_files: detail.value.check_binary_files,
        auto_check_enabled: detail.value.auto_check_enabled,
        license_id: detail.value.license_id,
        language_id: detail.value.language_id,
        version_extract_regex: detail.value.version_extract_regex ?? "",
      };
    } else {
      form.value = { ...defaultForm };
    }
    error.value = "";
  }

  function autoDetectFromPkgname() {
    const name = form.value.pkgname.trim().toLowerCase();
    if (!name) return;

    if (name.endsWith("-bin")) {
      form.value.package_type_id = 2;
      form.value.check_binary_files = true;
    } else if (name.endsWith("-git")) {
      form.value.package_type_id = 3;
    } else if (name.endsWith("-appimage")) {
      form.value.package_type_id = 4;
    } else {
      form.value.package_type_id = 1;
      form.value.check_binary_files = false;
    }
  }

  async function save(mode: string): Promise<boolean> {
    saving.value = true;
    error.value = "";
    try {
      if (mode === "add") {
        await invoke("add_software", {
          pkgname: form.value.pkgname.trim(),
          upstreamUrl: form.value.upstream_url || null,
          packageType: form.value.package_type_id,
          checkerType: form.value.checker_type_id,
          checkTestVersions: form.value.check_test_versions,
          checkBinaryFiles: form.value.check_binary_files,
          autoCheckEnabled: form.value.auto_check_enabled,
          licenseId: form.value.license_id,
          languageId: form.value.language_id,
          versionExtractRegex: form.value.version_extract_regex || null,
        });
      } else {
        if (!detail.value?.software_id) return false;
        await invoke("update_software", {
          softwareId: detail.value.software_id,
          pkgname: detail.value.pkgname,
          upstreamUrl: form.value.upstream_url || null,
          packageType: form.value.package_type_id,
          checkerType: form.value.checker_type_id,
          isOutdated: form.value.is_outdated,
          checkTestVersions: form.value.check_test_versions,
          checkBinaryFiles: form.value.check_binary_files,
          autoCheckEnabled: form.value.auto_check_enabled,
          licenseId: form.value.license_id,
          languageId: form.value.language_id,
          versionExtractRegex: form.value.version_extract_regex || null,
        });
      }
      return true;
    } catch (e) {
      error.value = String(e);
      return false;
    } finally {
      saving.value = false;
    }
  }

  async function init(mode: string, pkgname?: string) {
    error.value = "";
    loadEnums();
    if (mode === "edit" && pkgname) {
      await loadSoftware(pkgname);
    }
    resetForm(mode);
  }

  return {
    saving,
    error,
    detail,
    licenses,
    languages,
    form,
    isDirty,
    canSave,
    loadEnums,
    loadSoftware,
    resetForm,
    autoDetectFromPkgname,
    save,
    init,
  };
}
