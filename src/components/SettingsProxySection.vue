<!--
  SettingsProxySection.vue - 代理管理设置组件

  功能：
  - 配置代理文件下载 URL
  - 配置四类代理的测试 URL
  - 支持保存和重置为默认值
  - 输入合法性校验

  响应式设计：
  - PC端：标签和输入框水平排列
  - 平板/手机端：垂直排列，输入框占满宽度
-->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useSettingsStore } from "../stores/settings";

const settingsStore = useSettingsStore();

/** 设置项 */
const settings = ref({
  proxy_download_url: "",
  proxy_test_download_url: "",
  proxy_test_clone_url: "",
  proxy_test_raw_url: "",
  proxy_test_ssh_url: "",
});

/** 默认值 */
const defaults = {
  proxy_download_url: "https://update.greasyfork.org/scripts/412245/Github%20%E5%A2%9E%E5%BC%BA%20-%20%E9%AB%98%E9%80%9F%E4%B8%8B%E8%BD%BD.user.js",
  proxy_test_download_url: "https://github.com/zxp19821005/My_AUR_Files/releases/latest/download/README.md",
  proxy_test_clone_url: "https://github.com/zxp19821005/My_AUR_Files.git",
  proxy_test_raw_url: "https://raw.githubusercontent.com/zxp19821005/My_AUR_Files/main/README.md",
  proxy_test_ssh_url: "ssh://git@ssh.github.com:443/zxp19821005/My_AUR_Files",
};

/** 加载状态 */
const loading = ref(false);
/** 消息提示 */
const message = ref("");
/** 消息类型 */
const messageType = ref<"success" | "error" | "warning">("success");

onMounted(async () => {
  await loadSettings();
});

/** 加载设置 */
async function loadSettings() {
  loading.value = true;
  try {
    settings.value.proxy_download_url = await settingsStore.getSetting("proxy_download_url", defaults.proxy_download_url);
    settings.value.proxy_test_download_url = await settingsStore.getSetting("proxy_test_download_url", defaults.proxy_test_download_url);
    settings.value.proxy_test_clone_url = await settingsStore.getSetting("proxy_test_clone_url", defaults.proxy_test_clone_url);
    settings.value.proxy_test_raw_url = await settingsStore.getSetting("proxy_test_raw_url", defaults.proxy_test_raw_url);
    settings.value.proxy_test_ssh_url = await settingsStore.getSetting("proxy_test_ssh_url", defaults.proxy_test_ssh_url);
  } catch (e) {
    showMessage("加载设置失败: " + String(e), "error");
  } finally {
    loading.value = false;
  }
}

/** 验证 URL 格式 */
function isValidUrl(url: string): boolean {
  if (!url) return false;
  try {
    new URL(url);
    return true;
  } catch {
    return false;
  }
}

/** 保存单个设置 */
async function saveSetting(key: keyof typeof settings.value) {
  const value = settings.value[key];
  if (!isValidUrl(value)) {
    showMessage("请输入有效的 URL 格式", "error");
    return;
  }

  try {
    await settingsStore.setSetting(key, value);
    showMessage("保存成功", "success");
  } catch (e) {
    showMessage("保存失败: " + String(e), "error");
  }
}

/** 保存所有设置 */
async function saveAllSettings() {
  // 验证所有 URL
  for (const [key, value] of Object.entries(settings.value)) {
    if (!isValidUrl(value)) {
      showMessage(`请输入有效的 ${key} URL 格式`, "error");
      return;
    }
  }

  loading.value = true;
  try {
    for (const [key, value] of Object.entries(settings.value)) {
      await settingsStore.setSetting(key, value);
    }
    showMessage("所有设置已保存", "success");
  } catch (e) {
    showMessage("保存失败: " + String(e), "error");
  } finally {
    loading.value = false;
  }
}

/** 重置为默认值 */
async function resetToDefaults() {
  if (!confirm("确定要重置所有代理设置为默认值吗？")) return;

  settings.value = { ...defaults };
  await saveAllSettings();
}

/** 重置单个设置为默认值 */
async function resetSingleSetting(key: keyof typeof defaults) {
  settings.value[key] = defaults[key];
  await saveSetting(key);
}

/** 显示消息 */
function showMessage(text: string, type: "success" | "error" | "warning" = "success") {
  message.value = text;
  messageType.value = type;
  setTimeout(() => (message.value = ""), 3000);
}
</script>

<template>
  <div class="proxy-settings">
    <h3 class="page-title">代理管理设置</h3>
    <p class="page-desc">
      配置代理文件下载地址和各类代理的测试地址。
    </p>

    <div v-if="message" class="message" :class="`message-${messageType}`">
      {{ message }}
    </div>

    <div class="settings-card">
      <!-- 代理文件下载 URL -->
      <div class="setting-row">
        <div class="setting-label">
          <strong>代理文件下载地址</strong>
          <span class="setting-desc">用于下载代理规则 JS 文件</span>
        </div>
        <div class="setting-input-group">
          <input
            v-model="settings.proxy_download_url"
            type="text"
            class="text-input"
            placeholder="输入代理文件下载 URL"
          />
          <button class="btn btn-outline btn-sm" @click="resetSingleSetting('proxy_download_url')" title="重置为默认值">
            重置
          </button>
        </div>
      </div>

      <!-- 下载代理测试 URL -->
      <div class="setting-row">
        <div class="setting-label">
          <strong>下载代理测试地址</strong>
          <span class="setting-desc">用于测试下载代理的连通性</span>
        </div>
        <div class="setting-input-group">
          <input
            v-model="settings.proxy_test_download_url"
            type="text"
            class="text-input"
            placeholder="输入下载代理测试 URL"
          />
          <button class="btn btn-outline btn-sm" @click="resetSingleSetting('proxy_test_download_url')" title="重置为默认值">
            重置
          </button>
        </div>
      </div>

      <!-- 克隆代理测试 URL -->
      <div class="setting-row">
        <div class="setting-label">
          <strong>克隆代理测试地址</strong>
          <span class="setting-desc">用于测试克隆代理的连通性</span>
        </div>
        <div class="setting-input-group">
          <input
            v-model="settings.proxy_test_clone_url"
            type="text"
            class="text-input"
            placeholder="输入克隆代理测试 URL"
          />
          <button class="btn btn-outline btn-sm" @click="resetSingleSetting('proxy_test_clone_url')" title="重置为默认值">
            重置
          </button>
        </div>
      </div>

      <!-- RAW 代理测试 URL -->
      <div class="setting-row">
        <div class="setting-label">
          <strong>RAW 代理测试地址</strong>
          <span class="setting-desc">用于测试 RAW 代理的连通性</span>
        </div>
        <div class="setting-input-group">
          <input
            v-model="settings.proxy_test_raw_url"
            type="text"
            class="text-input"
            placeholder="输入 RAW 代理测试 URL"
          />
          <button class="btn btn-outline btn-sm" @click="resetSingleSetting('proxy_test_raw_url')" title="重置为默认值">
            重置
          </button>
        </div>
      </div>

      <!-- SSH 代理测试 URL -->
      <div class="setting-row">
        <div class="setting-label">
          <strong>SSH 代理测试地址</strong>
          <span class="setting-desc">用于测试 SSH 代理的连通性</span>
        </div>
        <div class="setting-input-group">
          <input
            v-model="settings.proxy_test_ssh_url"
            type="text"
            class="text-input"
            placeholder="输入 SSH 代理测试 URL"
          />
          <button class="btn btn-outline btn-sm" @click="resetSingleSetting('proxy_test_ssh_url')" title="重置为默认值">
            重置
          </button>
        </div>
      </div>
    </div>

    <!-- 操作按钮 -->
    <div class="setting-actions">
      <button class="btn btn-primary" @click="saveAllSettings" :disabled="loading">
        {{ loading ? "保存中..." : "保存所有设置" }}
      </button>
      <button class="btn btn-secondary" @click="resetToDefaults" :disabled="loading">
        重置为默认值
      </button>
    </div>
  </div>
</template>

<style scoped>
.proxy-settings {
  width: 100%;
  min-height: 100%;
}

.page-title {
  font-size: 1.25rem;
  margin-bottom: 0.5rem;
  color: var(--text-primary);
}

.page-desc {
  color: var(--text-secondary);
  font-size: 0.8125rem;
  margin-bottom: 1rem;
}

.message {
  padding: 0.5rem 1rem;
  margin-bottom: 1rem;
  border-radius: 6px;
  font-size: 0.875rem;
}

.message-success {
  background-color: rgba(76, 175, 125, 0.1);
  color: var(--success);
}

.message-error {
  background-color: rgba(231, 76, 60, 0.1);
  color: #e74c3c;
}

.message-warning {
  background-color: rgba(245, 158, 11, 0.1);
  color: #f59e0b;
}

.settings-card {
  background-color: var(--bg-card);
  border-radius: 12px;
  border: 1px solid var(--border);
  padding: 1.5rem;
  width: 100%;
  box-sizing: border-box;
}

.setting-row {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  padding: 1rem 0;
  border-bottom: 1px solid var(--border);
}

.setting-row:last-child {
  border-bottom: none;
  padding-bottom: 0;
}

.setting-row:first-child {
  padding-top: 0;
}

.setting-label {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.setting-label strong {
  font-size: 0.875rem;
  color: var(--text-primary);
}

.setting-desc {
  font-size: 0.75rem;
  color: var(--text-secondary);
}

.setting-input-group {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  width: 100%;
}

.text-input {
  flex: 1;
  min-width: 0;
  width: 100%;
  padding: 0.5rem 0.75rem;
  border-radius: 6px;
  border: 1px solid var(--border);
  background-color: var(--bg-primary);
  color: var(--text-primary);
  font-size: 0.875rem;
  font-family: 'SF Mono', 'Consolas', 'Monaco', monospace;
  transition: border-color 0.15s;
  overflow: hidden;
  text-overflow: ellipsis;
}

.text-input:focus {
  border-color: var(--accent);
  outline: none;
}

.text-input::placeholder {
  color: var(--text-muted);
}

.btn {
  padding: 0.5rem 1rem;
  border-radius: 6px;
  border: 1px solid var(--border);
  cursor: pointer;
  font-size: 0.8125rem;
  transition: all 0.15s;
  white-space: nowrap;
}

.btn-primary {
  background: var(--accent);
  color: white;
  border-color: var(--accent);
}

.btn-primary:hover:not(:disabled) {
  opacity: 0.9;
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-secondary {
  background: var(--bg-secondary);
  color: var(--text-primary);
}

.btn-secondary:hover:not(:disabled) {
  background: var(--bg-card);
}

.btn-secondary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-outline {
  background: none;
  border: 1px solid var(--border);
  color: var(--text-secondary);
}

.btn-outline:hover {
  border-color: var(--accent);
  color: var(--accent);
}

.btn-sm {
  padding: 0.375rem 0.625rem;
  font-size: 0.75rem;
}

.setting-actions {
  display: flex;
  gap: 0.75rem;
  margin-top: 1.5rem;
  padding-top: 1.5rem;
  border-top: 1px solid var(--border);
}

/* 响应式设计 - 平板及以下 */
@media (max-width: 768px) {
  .settings-card {
    padding: 1rem;
  }

  .setting-row {
    gap: 0.5rem;
  }

  .setting-input-group {
    flex-direction: column;
    align-items: stretch;
  }

  .text-input {
    width: 100%;
  }

  .btn-sm {
    align-self: flex-start;
  }

  .setting-actions {
    flex-direction: column;
  }

  .setting-actions .btn {
    width: 100%;
  }
}

/* 响应式设计 - 小屏幕手机 */
@media (max-width: 480px) {
  .page-title {
    font-size: 1.125rem;
  }

  .settings-card {
    padding: 0.75rem;
    border-radius: 8px;
  }

  .setting-row {
    padding: 0.75rem 0;
  }

  .text-input {
    font-size: 0.8125rem;
    padding: 0.375rem 0.5rem;
  }
}
</style>
