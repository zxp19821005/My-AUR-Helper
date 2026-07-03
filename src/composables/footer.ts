import { type InjectionKey } from "vue";

export interface FooterState {
  infoText: string;
  showPagination: boolean;
  totalRecords: number;
  currentPage: number;
  pageSize: number;
  onPageChange: ((page: number) => void) | null;
  progress: { current: number; total: number } | null;
}

export const defaultFooterState = (): FooterState => ({
  infoText: "",
  showPagination: false,
  totalRecords: 0,
  currentPage: 1,
  pageSize: 50,
  onPageChange: null,
  progress: null,
});

export const FOOTER_KEY: InjectionKey<FooterState> = Symbol("footer");
