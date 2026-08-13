export const pkgTypes: Record<number, string> = {
  1: "编译安装",
  2: "二进制包",
  3: "Git 仓库",
  4: "AppImage",
};

export const checkerTypes: Record<number, string> = {
  1: "GitHub Tag",
  2: "GitHub Release",
  3: "Gitee",
  4: "GitLab",
  5: "重定向",
  6: "HTTP 页面",
  7: "手动",
  8: "浏览器(JS渲染)",
};

export const pkgTypeOptions = Object.entries(pkgTypes).map(([id, label]) => ({
  id: Number(id),
  label,
}));

export const checkerTypeOptions = Object.entries(checkerTypes).map(([id, label]) => ({
  id: Number(id),
  label,
}));

export interface SelectOption<T> {
  value: T;
  label: string;
}

export const packageTypeFilterOptions: SelectOption<number | null>[] = [
  { value: null, label: "全部" },
  ...pkgTypeOptions.map((o) => ({ value: o.id, label: o.label })),
];

export const checkerTypeFilterOptions: SelectOption<number | null>[] = [
  { value: null, label: "全部" },
  ...checkerTypeOptions.map((o) => ({ value: o.id, label: o.label })),
];