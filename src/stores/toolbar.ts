import { defineStore } from "pinia";
import { ref } from "vue";

export const useToolbarStore = defineStore("toolbar", () => {
  const infoText = ref("");
  const showPagination = ref(false);
  const totalRecords = ref(0);
  const currentPage = ref(1);
  const pageSize = ref(50);
  const onPageChange = ref<((page: number) => void) | null>(null);
  const progress = ref<{ current: number; total: number } | null>(null);

  function setInfo(text: string) {
    infoText.value = text;
  }

  function setPagination(total: number, page: number, size: number, cb: (page: number) => void) {
    totalRecords.value = total;
    currentPage.value = page;
    pageSize.value = size;
    onPageChange.value = cb;
    showPagination.value = total > size;
  }

  function clearPagination() {
    showPagination.value = false;
    totalRecords.value = 0;
    currentPage.value = 1;
    onPageChange.value = null;
  }

  function setProgress(current: number, total: number) {
    progress.value = { current, total };
  }

  function clearProgress() {
    progress.value = null;
  }

  function reset() {
    infoText.value = "";
    clearPagination();
    clearProgress();
  }

  return {
    infoText, showPagination, totalRecords, currentPage, pageSize, onPageChange, progress,
    setInfo, setPagination, clearPagination, setProgress, clearProgress, reset,
  };
});
