<!--
  SettingsDynamicSection.vue - 动态设置分区

  功能：
  - 根据 category 加载对应分类的设置项（来自 get_settings）
  - 采用草稿模型：编辑仅修改本地 draft，点击「保存设置」才写入数据库
  - 「重置设置」撤销未保存修改，恢复到上次保存值
  - 修复令牌输入框过窄问题（password-wrapper 撑满）

  适用分类：list / aur / checker / backup
-->
<script setup lang="ts">
import { ref, onMounted, nextTick, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { Setting } from "../../types";
import { useSettingsStore } from "../../stores/settings";
import { useSettingsDraft } from "../../composables/useSettingsDraft";
import StandardizedCard from "../base/StandardizedCard.vue";
import StandardizedInput from "../base/StandardizedInput.vue";
import SettingRow from "./SettingRow.vue";
import SettingsActionBar from "./SettingsActionBar.vue";
import { Icon } from "../../icons";

const props = defineProps<{
  /** 设置分类 */
  category: string;
}>();

const settingsStore = useSettingsStore();

/** 初始空数组，挂载后从数据库加载 */
const { draft, dirty, saving, reset, commit } = useSettingsDraft<Setting[]>([]);

const loading = ref(true);
const message = ref("");
const passwordVisible = ref<Record<string, boolean>>({});

const categoryLabels: Record<string, string> = {
  list: "列表设置",
  aur: "AUR 设置",
  checker: "上游检查器设置",
  backup: "备份管理设置",
};

const title = computed(() => categoryLabels[props.category] || props.category);

onMounted(async () => {
  await load();
  initTextareas();
});

async function load() {
  loading.value = true;
  try {
    const all = await invoke<Setting[]>("get_settings");
    const filtered = all.filter((s) => s.category === props.category);
    // 用加载结果同时初始化 saved 与 draft
    draft.value = filtered.map((s) => ({ ...s }));
    commit();
  } catch (e) {
    message.value = "加载失败: " + String(e);
  } finally {
    loading.value = false;
  }
}

async function handleSave() {
  saving.value = true;
  try {
    // 设置项数量很少，直接全量保存（走 store.setSetting 集中写入并同步缓存）
    await Promise.all(
      draft.value.map((item) => settingsStore.setSetting(item.key, item.value)),
    );
    await settingsStore.refreshAllSettings();
    commit();
    showMessage("已保存");
  } catch (e) {
    message.value = "保存失败: " + String(e);
  } finally {
    saving.value = false;
  }
}

function showMessage(text: string) {
  message.value = text;
  setTimeout(() => {
    if (message.value === text) message.value = "";
  }, 2000);
}

function isTokenKey(key: string): boolean {
  return key.includes("token");
}

function togglePassword(key: string) {
  passwordVisible.value[key] = !passwordVisible.value[key];
}

function inputType(s: Setting): string {
  if (!isTokenKey(s.key)) return "text";
  return passwordVisible.value[s.key] ? "text" : "password";
}

function onDraftInput(key: string, value: string) {
  const item = draft.value.find((x) => x.key === key);
  if (item) item.value = value;
}

/** 自动调整 textarea 高度 */
function autoResize(event: Event) {
  const el = event.target as HTMLTextAreaElement;
  el.style.height = "auto";
  el.style.height = el.scrollHeight + "px";
}

function initTextareas() {
  nextTick(() => {
    document
      .querySelectorAll(".dynamic-section-root .settings-textarea")
      .forEach((el) => {
        const ta = el as HTMLTextAreaElement;
        ta.style.height = "auto";
        ta.style.height = ta.scrollHeight + "px";
      });
  });
}
</script>

<template>
  <div class="dynamic-section-root">
    <div v-if="message" class="settings-message">{{ message }}</div>

    <StandardizedCard v-if="loading" title="加载中">
      <p class="loading-text">正在加载设置...</p>
    </StandardizedCard>

    <StandardizedCard
      v-else-if="draft.length > 0"
      :title="title"
    >
      <div v-for="s in draft" :key="s.key" class="settings-row-wrapper">
        <SettingRow :label="s.description || s.key" :description="s.key">
          <!-- Token 类型输入框（带密码显示/隐藏），撑满宽度 -->
          <template v-if="isTokenKey(s.key)">
            <div class="password-wrapper">
              <StandardizedInput
                :type="inputType(s) as any"
                :modelValue="s.value"
                size="md"
                @update:modelValue="(val: string) => onDraftInput(s.key, val)"
              />
              <button
                class="toggle-password"
                @click="togglePassword(s.key)"
                type="button"
                :title="passwordVisible[s.key] ? '隐藏' : '显示'"
              >
                <component :is="Icon.show" v-if="passwordVisible[s.key]" :size="18" />
                <component :is="Icon.hide" v-else :size="18" />
              </button>
            </div>
          </template>
          <!-- 普通输入框 - 使用 textarea 支持多行显示 -->
          <template v-else>
            <textarea
              :value="s.value"
              class="settings-textarea"
              rows="1"
              @input="onDraftInput(s.key, ($event.target as HTMLTextAreaElement).value); autoResize($event)"
            ></textarea>
          </template>
        </SettingRow>
      </div>
    </StandardizedCard>

    <StandardizedCard v-else title="暂无设置">
      <p class="empty-text">当前分类下暂无设置项</p>
    </StandardizedCard>

    <SettingsActionBar
      :dirty="dirty"
      :saving="saving"
      @save="handleSave"
      @reset="reset"
    />
  </div>
</template>

<style scoped>
.settings-row-wrapper {
  width: 100%;
}

.settings-message {
  padding: 0.75rem 1rem;
  margin-bottom: 1rem;
  background-color: rgba(76, 175, 125, 0.1);
  color: var(--success);
  border-radius: 8px;
  font-size: 0.875rem;
}

.loading-text,
.empty-text {
  color: var(--text-secondary);
}

.password-wrapper {
  position: relative;
  display: flex;
  align-items: center;
  flex: 1 1 auto;
  min-width: 0;
}

.password-wrapper :deep(.standardized-input) {
  padding-right: 2.5rem;
  width: 100%;
}

.toggle-password {
  position: absolute;
  right: 0.5rem;
  top: 50%;
  transform: translateY(-50%);
  background: none;
  border: none;
  cursor: pointer;
  color: var(--text-secondary);
  padding: 0.25rem;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  transition: all 0.15s;
}

.toggle-password:hover {
  color: var(--text-primary);
  background-color: var(--hover-bg, rgba(128, 128, 128, 0.1));
}

.settings-textarea {
  width: 100%;
  padding: 0.5rem 0.75rem;
  border: 1px solid var(--border);
  border-radius: 8px;
  background-color: var(--bg-primary);
  color: var(--text-primary);
  font-size: 0.875rem;
  font-family: inherit;
  line-height: 1.5;
  resize: none;
  overflow: hidden;
  min-height: 2.5rem;
  word-break: break-all;
}

.settings-textarea:focus {
  border-color: var(--accent);
  outline: none;
  box-shadow: 0 0 0 2px rgba(108, 99, 255, 0.15);
}

.settings-textarea::placeholder {
  color: var(--text-muted);
}
</style>
