# 通用组件使用指南

本文档介绍项目中所有通用组件的使用方法和最佳实践。

---

## 📦 基础组件（`/components/base/`）

### 1. StandardizedStatCard - 统计卡片

用于展示统计数据，支持趋势指示器和点击交互。

#### Props

| 属性 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `title` | `string` | - | 卡片标题 |
| `value` | `string \| number` | - | 统计数值 |
| `icon` | `Component` | - | 图标组件 |
| `trend` | `'up' \| 'down' \| 'neutral'` | `'neutral'` | 趋势方向 |
| `trendValue` | `string` | - | 趋势数值 |
| `color` | `string` | - | 主题色（CSS变量） |
| `clickable` | `boolean` | `false` | 是否可点击 |

#### 示例

```vue
<script setup lang="ts">
import { Package, TrendingUp } from "@lucide/vue";
import StandardizedStatCard from "@/components/base/StandardizedStatCard.vue";

const totalPackages = 1234;
</script>

<template>
  <StandardizedStatCard
    title="总包数"
    :value="totalPackages"
    :icon="Package"
    trend="up"
    trendValue="+12%"
    color="var(--accent)"
    clickable
    @click="navigateToPackages"
  />
</template>
```

---

### 2. StandardizedSearchBox - 搜索框

统一搜索输入框，支持防抖、快捷键和清空功能。

#### Props

| 属性 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `modelValue` | `string` | `""` | 搜索关键词（v-model） |
| `placeholder` | `string` | `"搜索..."` | 占位符文本 |
| `debounce` | `number` | `500` | 防抖延迟（毫秒） |
| `size` | `'sm' \| 'md' \| 'lg'` | `'md'` | 搜索框尺寸 |
| `clearable` | `boolean` | `true` | 是否显示清空按钮 |

#### Events

| 事件 | 参数 | 说明 |
|------|------|------|
| `update:modelValue` | `(value: string)` | 搜索关键词变化 |
| `search` | `(value: string)` | 触发搜索（防抖后） |
| `clear` | - | 清空搜索 |

#### 快捷键

- `Ctrl/Cmd + K` - 聚焦搜索框
- `Escape` - 清空并失焦

#### 示例

```vue
<script setup lang="ts">
import StandardizedSearchBox from "@/components/base/StandardizedSearchBox.vue";

const searchQuery = ref("");

function handleSearch(value: string) {
  console.log("搜索:", value);
  // 执行搜索逻辑
}
</script>

<template>
  <StandardizedSearchBox
    v-model="searchQuery"
    placeholder="搜索软件包..."
    :debounce="300"
    size="md"
    @search="handleSearch"
  />
</template>
```

---

### 3. StandardizedEmptyState - 空状态

统一空数据状态的展示。

#### Props

| 属性 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `icon` | `Component` | - | 图标组件 |
| `title` | `string` | `"暂无数据"` | 空状态标题 |
| `description` | `string` | - | 空状态描述 |
| `size` | `'sm' \| 'md' \| 'lg'` | `'md'` | 组件尺寸 |

#### Slots

| 插槽 | 说明 |
|------|------|
| `default` | 自定义内容区域 |
| `icon` | 自定义图标 |
| `title` | 自定义标题 |
| `description` | 自定义描述 |
| `actions` | 操作按钮区域 |

#### 示例

```vue
<script setup lang="ts">
import { Package } from "@lucide/vue";
import StandardizedEmptyState from "@/components/base/StandardizedEmptyState.vue";
</script>

<template>
  <StandardizedEmptyState
    :icon="Package"
    title="暂无软件包"
    description="点击添加按钮添加第一个软件包"
  >
    <template #actions>
      <button class="btn btn-primary" @click="addPackage">添加软件包</button>
    </template>
  </StandardizedEmptyState>
</template>
```

---

### 4. StandardizedMessage - 消息提示

统一消息提示样式，支持自动消失。

#### Props

| 属性 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `type` | `'success' \| 'error' \| 'warning' \| 'info'` | `'info'` | 消息类型 |
| `message` | `string` | `""` | 消息内容 |
| `duration` | `number` | `3000` | 自动消失时间（毫秒），0表示不自动消失 |
| `closable` | `boolean` | `true` | 是否显示关闭按钮 |
| `show` | `boolean` | `true` | 是否显示 |

#### Events

| 事件 | 说明 |
|------|------|
| `close` | 关闭消息 |
| `update:show` | 显示状态变化 |

#### 示例

```vue
<script setup lang="ts">
import StandardizedMessage from "@/components/base/StandardizedMessage.vue";

const showMessage = ref(true);
</script>

<template>
  <StandardizedMessage
    v-model:show="showMessage"
    type="success"
    message="保存成功"
    :duration="3000"
    closable
  />
</template>
```

---

### 5. StandardizedInput - 输入框

统一输入框样式，支持多种功能。

#### Props

| 属性 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `modelValue` | `string` | `""` | 输入值（v-model） |
| `type` | `'text' \| 'password' \| 'number' \| 'email' \| 'url'` | `'text'` | 输入类型 |
| `placeholder` | `string` | `""` | 占位符 |
| `size` | `'sm' \| 'md' \| 'lg'` | `'md'` | 输入框尺寸 |
| `disabled` | `boolean` | `false` | 是否禁用 |
| `clearable` | `boolean` | `false` | 是否显示清空按钮 |
| `prefix` | `Component` | - | 前置图标 |
| `suffix` | `Component` | - | 后置图标 |
| `error` | `boolean` | `false` | 是否显示错误状态 |
| `success` | `boolean` | `false` | 是否显示成功状态 |

#### Events

| 事件 | 参数 | 说明 |
|------|------|------|
| `update:modelValue` | `(value: string)` | 值变化 |
| `input` | `(value: string)` | 输入事件 |
| `change` | `(value: string)` | 变化事件 |
| `clear` | - | 清空事件 |

#### 示例

```vue
<script setup lang="ts">
import { User, Mail } from "@lucide/vue";
import StandardizedInput from "@/components/base/StandardizedInput.vue";

const username = ref("");
const email = ref("");
</script>

<template>
  <StandardizedInput
    v-model="username"
    :prefix="User"
    placeholder="请输入用户名"
    clearable
  />

  <StandardizedInput
    v-model="email"
    :prefix="Mail"
    type="email"
    placeholder="请输入邮箱"
    :error="!isValidEmail"
  />
</template>
```

---

### 6. StandardizedSelect - 下拉选择框

统一选择框样式。

#### Props

| 属性 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `modelValue` | `string \| number \| null` | `null` | 选中值（v-model） |
| `options` | `Array<{value, label}>` | `[]` | 选项列表 |
| `placeholder` | `string` | `"请选择"` | 占位符 |
| `size` | `'sm' \| 'md' \| 'lg'` | `'md'` | 选择框尺寸 |
| `disabled` | `boolean` | `false` | 是否禁用 |
| `prefix` | `Component` | - | 前置图标 |

#### Events

| 事件 | 参数 | 说明 |
|------|------|------|
| `update:modelValue` | `(value: string \| number)` | 值变化 |
| `change` | `(value: string \| number)` | 变化事件 |

#### 示例

```vue
<script setup lang="ts">
import { Palette } from "@lucide/vue";
import StandardizedSelect from "@/components/base/StandardizedSelect.vue";

const theme = ref("dark");

const themeOptions = [
  { value: "dark", label: "深色" },
  { value: "light", label: "浅色" },
];
</script>

<template>
  <StandardizedSelect
    v-model="theme"
    :options="themeOptions"
    :prefix="Palette"
    placeholder="选择主题"
  />
</template>
```

---

### 7. StandardizedCard - 卡片容器

通用卡片容器，支持标题、副标题和状态。

#### Props

| 属性 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `title` | `string` | `""` | 卡片标题 |
| `subtitle` | `string` | `""` | 卡片副标题 |
| `variant` | `'' \| 'variant-compact' \| 'variant-wide'` | `""` | 卡片变体 |
| `layout` | `'' \| 'layout-table' \| 'layout-flow'` | `""` | 布局方式 |

#### Slots

| 插槽 | 说明 |
|------|------|
| `default` | 卡片内容 |
| `status` | 状态区域 |

#### 示例

```vue
<script setup lang="ts">
import StandardizedCard from "@/components/base/StandardizedCard.vue";
</script>

<template>
  <StandardizedCard
    title="软件包信息"
    subtitle="来自 AUR 仓库"
    layout="table"
  >
    <table class="info-table">
      <tbody>
        <tr>
          <td class="label">版本</td>
          <td class="value">1.0.0</td>
        </tr>
      </tbody>
    </table>
  </StandardizedCard>
</template>
```

---

### 8. StandardizedButton - 按钮

通用按钮组件，支持多种样式和尺寸。

#### Props

| 属性 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `variant` | `'primary' \| 'secondary' \| 'outline' \| 'danger' \| 'ghost'` | `'primary'` | 按钮样式 |
| `size` | `'sm' \| 'md' \| 'lg'` | `'md'` | 按钮尺寸 |
| `disabled` | `boolean` | `false` | 是否禁用 |
| `loading` | `boolean` | `false` | 是否加载中 |

#### 示例

```vue
<script setup lang="ts">
import StandardizedButton from "@/components/base/StandardizedButton.vue";
</script>

<template>
  <StandardizedButton variant="primary" size="md">
    保存
  </StandardizedButton>

  <StandardizedButton variant="danger" :loading="saving">
    删除
  </StandardizedButton>
</template>
```

---

### 9. StandardizedBadge - 徽章

通用徽章组件，支持多种类型和样式。

#### Props

| 属性 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `type` | `'primary' \| 'secondary' \| 'success' \| 'warning' \| 'danger' \| 'info' \| 'neutral'` | `'neutral'` | 徽章类型 |
| `size` | `'sm' \| 'md' \| 'lg'` | `'md'` | 徽章尺寸 |
| `variant` | `'filled' \| 'outlined' \| 'soft'` | `'filled'` | 徽章样式 |
| `rounded` | `boolean` | `true` | 是否圆角 |
| `dot` | `boolean` | `false` | 是否显示圆点 |
| `text` | `string` | - | 徽章文本 |

#### 示例

```vue
<script setup lang="ts">
import StandardizedBadge from "@/components/base/StandardizedBadge.vue";
</script>

<template>
  <StandardizedBadge type="success" text="已最新" />
  <StandardizedBadge type="warning" text="需更新" dot />
</template>
```

---

## 🎨 通用组件（`/components/common/`）

### 1. StandardizedTable - 增强数据表格

功能强大的数据表格组件，支持分页、排序、选择等。

#### Props

| 属性 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `columns` | `Column[]` | - | 列配置 |
| `data` | `any[]` | - | 数据源 |
| `pageSize` | `number` | `50` | 每页显示行数（0表示不分页） |
| `searchQuery` | `string` | `""` | 搜索关键词 |
| `searchFields` | `string[]` | `[]` | 搜索字段 |
| `rowKey` | `string` | `'id'` | 行唯一标识字段 |
| `showCheckbox` | `boolean` | `false` | 是否显示复选框列 |
| `showIndex` | `boolean` | `false` | 是否显示序号列 |
| `emptyText` | `string` | `'暂无数据'` | 空状态提示文本 |
| `loading` | `boolean` | `false` | 是否显示加载状态 |
| `striped` | `boolean` | `false` | 是否显示条纹 |
| `compact` | `boolean` | `false` | 是否紧凑模式 |
| `hoverable` | `boolean` | `true` | 是否启用悬停效果 |
| `clickable` | `boolean` | `false` | 是否启用行点击 |

#### Column 接口

```typescript
interface Column {
  key: string;              // 字段名
  title: string;            // 列标题
  width?: string;           // 列宽度
  formatter?: (value: any, row: any) => string;  // 格式化函数
  align?: 'left' | 'center' | 'right';  // 对齐方式
  sortable?: boolean;       // 是否可排序
  hidden?: boolean;         // 是否隐藏
}
```

#### Events

| 事件 | 参数 | 说明 |
|------|------|------|
| `selection-change` | `(selectedRows: any[])` | 选择变化事件 |
| `row-click` | `(row: any)` | 行点击事件 |
| `page-change` | `(page: number)` | 页码变化事件 |
| `sort-change` | `(key: string, direction: 'asc' \| 'desc' \| null)` | 排序变化事件 |

#### Slots

| 插槽 | 参数 | 说明 |
|------|------|------|
| `cell-{key}` | `{ row, value, index }` | 自定义单元格内容 |
| `actions` | `{ row, index }` | 操作列内容 |
| `empty` | - | 自定义空状态 |

#### 示例

```vue
<script setup lang="ts">
import StandardizedTable from "@/components/common/StandardizedTable.vue";
import type { Column } from "@/components/common/StandardizedTable.vue";

const columns: Column[] = [
  { key: "pkgname", title: "包名", sortable: true },
  { key: "aur_version", title: "AUR 版本" },
  { key: "upstream_version", title: "上游版本", sortable: true },
  { key: "id", title: "操作", width: "120px", align: "center" },
];

const packages = ref([...]);
const searchQuery = ref("");
const selectedRows = ref([]);

function handleSelectionChange(rows: any[]) {
  selectedRows.value = rows;
}

function handleRowClick(row: any) {
  console.log("点击行:", row);
}
</script>

<template>
  <StandardizedTable
    :columns="columns"
    :data="packages"
    :pageSize="50"
    :searchQuery="searchQuery"
    :searchFields="['pkgname']"
    showCheckbox
    showIndex
    striped
    hoverable
    @selection-change="handleSelectionChange"
    @row-click="handleRowClick"
  >
    <template #cell-pkgname="{ row }">
      <strong :class="{ 'pkg-outdated': row.is_outdated }">
        {{ row.pkgname }}
      </strong>
    </template>

    <template #actions="{ row }">
      <button @click="edit(row)">编辑</button>
      <button @click="delete(row)">删除</button>
    </template>
  </StandardizedTable>
</template>
```

---

### 2. StandardizedModal - 增强模态框

功能强大的模态框组件，支持多种尺寸和交互方式。

#### Props

| 属性 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `show` | `boolean` | `false` | 是否显示 |
| `title` | `string` | `""` | 模态框标题 |
| `width` | `'sm' \| 'md' \| 'lg' \| 'xl' \| 'full' \| string` | `'md'` | 模态框宽度 |
| `hideHeader` | `boolean` | `false` | 是否隐藏头部 |
| `hideFooter` | `boolean` | `false` | 是否隐藏底部 |
| `closable` | `boolean` | `true` | 是否允许关闭 |
| `closeOnEsc` | `boolean` | `true` | 是否允许ESC键关闭 |
| `closeOnOverlay` | `boolean` | `true` | 是否允许点击遮罩关闭 |
| `scrollable` | `boolean` | `true` | 内容是否可滚动 |
| `draggable` | `boolean` | `false` | 是否可拖拽 |

#### Events

| 事件 | 说明 |
|------|------|
| `close` | 关闭模态框 |
| `update:show` | 显示状态变化 |

#### Slots

| 插槽 | 说明 |
|------|------|
| `default` | 模态框主体内容 |
| `header` | 自定义头部 |
| `footer` | 自定义底部操作区 |
| `close` | 自定义关闭按钮 |
| `error` | 错误提示区域 |

#### 示例

```vue
<script setup lang="ts">
import StandardizedModal from "@/components/common/StandardizedModal.vue";

const showModal = ref(false);
const formError = ref("");

function handleSave() {
  // 保存逻辑
  showModal.value = false;
}
</script>

<template>
  <StandardizedModal
    v-model:show="showModal"
    title="编辑软件包"
    width="lg"
    @close="handleClose"
  >
    <template v-if="formError" #error>
      {{ formError }}
    </template>

    <form @submit.prevent="handleSave">
      <!-- 表单内容 -->
    </form>

    <template #footer>
      <button class="btn btn-secondary" @click="showModal = false">取消</button>
      <button class="btn btn-primary" @click="handleSave">保存</button>
    </template>
  </StandardizedModal>
</template>
```

---

## 🎯 最佳实践

### 1. 组件导入规范

```typescript
// ✅ 推荐：使用别名导入
import StandardizedCard from "@/components/base/StandardizedCard.vue";
import StandardizedTable from "@/components/common/StandardizedTable.vue";

// ❌ 不推荐：使用相对路径
import StandardizedCard from "../../components/base/StandardizedCard.vue";
```

### 2. 类型安全

```typescript
// ✅ 推荐：使用TypeScript类型
import type { Column } from "@/components/common/StandardizedTable.vue";

const columns: Column[] = [
  { key: "name", title: "名称" },
];

// ❌ 不推荐：使用any
const columns: any[] = [...];
```

### 3. 样式变量

始终使用CSS变量，不要硬编码颜色值：

```css
/* ✅ 推荐 */
color: var(--accent);
background: var(--bg-card);

/* ❌ 不推荐 */
color: #6c63ff;
background: #2a2d4a;
```

### 4. 响应式设计

所有组件都应考虑响应式适配：

```vue
<style scoped>
@media (max-width: 768px) {
  /* 移动端样式 */
}
</style>
```

### 5. 事件命名

使用kebab-case命名事件：

```vue
<!-- ✅ 推荐 -->
@selection-change="handler"
@row-click="handler"

<!-- ❌ 不推荐 -->
@selectionChange="handler"
@rowClick="handler"
```

---

## 📝 迁移指南

### 迁移状态

✅ **迁移已完成（2026-07-30）**：旧组件 `DataTable.vue` 和 `Modal.vue` 已全部删除，项目统一使用 `StandardizedTable` 和 `StandardizedModal`。

**说明**：
- `StandardizedModal` 的 `width` 支持预设值（`sm`/`md`/`lg`/`xl`/`full`）和任意像素值（如 `"720px"`，通过内联样式应用）
- 新增代码请直接使用 Standardized 系列组件

---

## 🔧 故障排除

### 问题：组件样式不生效

**解决方案：**
1. 检查是否正确导入组件
2. 检查CSS变量是否正确定义
3. 确保使用`<style scoped>`

### 问题：TypeScript类型错误

**解决方案：**
1. 确保导入正确的类型定义
2. 检查Props类型是否匹配
3. 使用`as`类型断言（必要时）

### 问题：事件不触发

**解决方案：**
1. 检查事件名称是否正确（kebab-case）
2. 确保emit正确调用
3. 检查事件处理器是否正确绑定

---

## 📚 相关资源

- [Vue 3 官方文档](https://cn.vuejs.org/)
- [TypeScript 官方文档](https://www.typescriptlang.org/zh/)
- [Lucide 图标库](https://lucide.dev/)

---

**最后更新**: 2026-07-30
**维护者**: AI Assistant