# 前端模块化重构方案

## 一、重复模块识别与分类

### 1.1 重复的枚举值映射

| 文件 | 重复内容 |
|------|---------|
| `SoftwareDetailModal.vue` | `pkgTypes`, `checkerTypes` |
| `composables/useSoftwareForm.ts` | `pkgTypes`, `checkerTypes` |

### 1.2 重复的图标映射

| 文件 | 重复内容 |
|------|---------|
| `Sidebar.vue` | `iconMap` 对象 |
| `TabBar.vue` | `iconMap` 对象（部分相同） |

### 1.3 重复的样式定义

| 文件 | 重复样式 |
|------|---------|
| `SoftwareFormModal.css` | `.modal-*`, `.btn-*`, `.form-*`, `.checkbox-label` |
| `styles.css` | 相同的 `.modal-*`, `.btn-*`, `.form-*`, `.checkbox-label` |

### 1.4 重复的接口定义

| 文件 | 重复内容 |
|------|---------|
| `composables/footer.ts` | `FooterState` 接口 |
| `composables/packageActions.ts` | `FooterState` 接口（不同定义） |

### 1.5 重复的弹窗组件逻辑

| 文件 | 重复逻辑 |
|------|---------|
| `SoftwareFormModal.vue` | 弹窗基础结构（overlay → modal → header/body/footer） |
| `SoftwareDetailModal.vue` | 相同的弹窗基础结构 |

---

## 二、模块化文件组织结构设计

### 2.1 优化后的目录结构

```
src/
├── components/
│   ├── common/                    # 通用基础组件
│   │   ├── Modal.vue              # 通用弹窗容器组件
│   │   ├── ModalHeader.vue        # 弹窗头部组件
│   │   ├── ModalBody.vue          # 弹窗内容组件
│   │   ├── ModalFooter.vue        # 弹窗底部组件
│   │   ├── IconButton.vue         # 通用图标按钮
│   │   └── DataTable.vue          # 通用数据表格
│   ├── layout/                    # 布局组件
│   │   ├── Sidebar.vue            # 侧边栏
│   │   ├── TabBar.vue             # 标签栏
│   │   ├── PageToolbar.vue        # 页面工具栏
│   │   └── BottomToolbar.vue      # 底部工具栏
│   ├── modals/                    # 弹窗业务组件
│   │   ├── SoftwareFormModal.vue
│   │   ├── SoftwareDetailModal.vue
│   │   └── SettingsPopup.vue
│   └── ...
├── composables/
│   ├── useModal.ts                # 弹窗逻辑 hook
│   ├── useTable.ts                # 表格逻辑 hook
│   ├── usePagination.ts           # 分页逻辑 hook
│   ├── footer.ts                  # Footer 状态管理
│   ├── packageActions.ts          # 包操作逻辑
│   └── useSoftwareForm.ts         # 软件表单逻辑
├── utils/                         # 工具函数（新增）
│   ├── enums.ts                   # 枚举值映射（pkgTypes, checkerTypes）
│   ├── icons.ts                   # 图标组件映射
│   └── constants.ts               # 常量定义
├── types/
│   └── index.ts                   # 类型定义（合并 FooterState）
└── assets/
    └── styles.css                 # 全局样式（移除重复的组件样式）
```

---

## 三、具体重构方案

### 3.1 创建枚举值映射工具文件

**文件**: `src/utils/enums.ts`

```typescript
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
};

export const pkgTypeOptions = Object.entries(pkgTypes).map(([id, label]) => ({
  id: Number(id),
  label,
}));

export const checkerTypeOptions = Object.entries(checkerTypes).map(([id, label]) => ({
  id: Number(id),
  label,
}));
```

### 3.2 创建图标映射工具文件

**文件**: `src/utils/icons.ts`

```typescript
import {
  LayoutDashboard,
  Package,
  HardDrive,
  Globe,
  Settings,
  FileText,
  Code,
  ScrollText,
  Database,
} from "@lucide/vue";

export const iconMap: Record<string, any> = {
  LayoutDashboard,
  Package,
  HardDrive,
  Database,
  Globe,
  Settings,
  FileText,
  Code,
  ScrollText,
};

export type IconName = keyof typeof iconMap;
```

### 3.3 创建通用弹窗组件

**文件**: `src/components/common/Modal.vue`

```vue
<script setup lang="ts">
defineProps<{
  show: boolean;
  title?: string;
}>();

const emit = defineEmits<{
  close: [];
}>();
</script>

<template>
  <Teleport to="body">
    <div v-if="show" class="modal-overlay" @click.self="emit('close')">
      <div class="modal">
        <div class="modal-header">
          <h3 v-if="title">{{ title }}</h3>
          <slot name="header"></slot>
          <button class="modal-close" @click="emit('close')">
            <slot name="close">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
            </slot>
          </button>
        </div>
        <div v-if="$slots.error" class="modal-error">
          <slot name="error"></slot>
        </div>
        <div class="modal-body">
          <slot></slot>
        </div>
        <div v-if="$slots.footer" class="modal-footer">
          <slot name="footer"></slot>
        </div>
      </div>
    </div>
  </Teleport>
</template>
```

### 3.4 合并 FooterState 接口

**修改**: `src/types/index.ts`

```typescript
export interface FooterState {
  infoText: string;
  showPagination: boolean;
  totalRecords: number;
  currentPage: number;
  pageSize: number;
  onPageChange: ((page: number) => void) | null;
  progress: { current: number; total: number; message?: string } | null;
}
```

**删除**: `composables/packageActions.ts` 中的 `FooterState` 接口定义

### 3.5 清理重复样式

**修改**: 删除 `SoftwareFormModal.css` 中的重复样式，保留组件特有样式

**修改**: `styles.css` 作为唯一的全局样式源，确保包含所有通用样式

### 3.6 更新引用

#### Sidebar.vue
```typescript
import { iconMap } from "../utils/icons";
```

#### TabBar.vue
```typescript
import { iconMap } from "../utils/icons";
```

#### SoftwareDetailModal.vue
```typescript
import { pkgTypes, checkerTypes } from "../utils/enums";
```

#### useSoftwareForm.ts
```typescript
import { pkgTypeOptions, checkerTypeOptions } from "../utils/enums";
```

#### packageActions.ts
```typescript
import type { FooterState } from "../types";
```

---

## 四、模块依赖关系

```
┌─────────────────────────────────────────────────────────────────┐
│                         视图层 (views/)                         │
│   PackageList.vue  PackageDetail.vue  BackupManager.vue ...     │
└───────────────────────────┬─────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                       组件层 (components/)                       │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐   │
│  │ 通用组件      │  │ 布局组件      │  │ 业务弹窗组件          │   │
│  │ Modal        │  │ Sidebar      │  │ SoftwareFormModal    │   │
│  │ IconButton   │  │ TabBar       │  │ SoftwareDetailModal  │   │
│  │ DataTable    │  │ PageToolbar  │  └──────────────────────┘   │
│  └──────┬───────┘  │ BottomToolbar│                              │
│         │          └──────┬───────┘                              │
│         │                 │                                      │
└─────────┼─────────────────┼─────────────────────────────────────┘
          │                 │
          ▼                 ▼
┌─────────────────────────────────────────────────────────────────┐
│                    组合式函数层 (composables/)                    │
│  useModal  useTable  usePagination  useSoftwareForm             │
│  footer    packageActions                                        │
└───────────────────────────┬─────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                       工具层 (utils/)                            │
│  enums.ts        icons.ts       constants.ts                     │
│  - pkgTypes      - iconMap      - API endpoints                  │
│  - checkerTypes  - IconName     - Storage keys                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## 五、实施步骤

| 步骤 | 内容 | 文件 |
|------|------|------|
| 1 | 创建 `src/utils/enums.ts` | 枚举值映射 |
| 2 | 创建 `src/utils/icons.ts` | 图标组件映射 |
| 3 | 创建 `src/components/common/Modal.vue` | 通用弹窗组件 |
| 4 | 更新 `src/types/index.ts` | 合并 FooterState 接口 |
| 5 | 更新 `composables/packageActions.ts` | 使用 types/index.ts 中的 FooterState |
| 6 | 更新 `Sidebar.vue` | 使用 utils/icons.ts |
| 7 | 更新 `TabBar.vue` | 使用 utils/icons.ts |
| 8 | 更新 `SoftwareDetailModal.vue` | 使用 utils/enums.ts |
| 9 | 更新 `useSoftwareForm.ts` | 使用 utils/enums.ts |
| 10 | 清理 `SoftwareFormModal.css` | 删除重复样式 |
| 11 | 更新 `SoftwareFormModal.vue` | 使用通用 Modal 组件 |
| 12 | 更新 `SoftwareDetailModal.vue` | 使用通用 Modal 组件 |

---

## 六、验证方案

1. **编译检查**: `pnpm vue-tsc --noEmit`
2. **运行检查**: `pnpm tauri dev`
3. **功能验证**:
   - 弹窗显示正常
   - 枚举值显示正确
   - 图标渲染正常
   - Footer 状态同步正常
   - 表单提交正常
