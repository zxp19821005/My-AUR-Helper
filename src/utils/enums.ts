export const pkgTypes: Record<number, string> = {
  1: "编译安装",
  2: "二进制包",
  3: "Git 仓库",
  4: "AppImage",
};

export const checkerTypes: Record<number, string> = {
  1: "GitHub Release",
  2: "GitHub Tag",
  3: "Gitee",
  4: "GitLab",
  5: "重定向",
  6: "HTTP 页面解析",
  7: "手动",
  8: "Git Describe",
};

export const pkgTypeOptions = Object.entries(pkgTypes).map(([id, label]) => ({
  id: Number(id),
  label,
}));

export const checkerTypeOptions = Object.entries(checkerTypes).map(([id, label]) => ({
  id: Number(id),
  label,
}));
