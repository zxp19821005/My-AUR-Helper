<!--
  SettingsProxySection.vue - 代理管理设置组件

  功能：
  - 配置代理文件下载 URL 与四类代理测试 URL
  - 采用草稿模型：编辑仅修改本地 draft，点击「保存设置」才写入数据库
  - 「重置设置」撤销未保存修改，恢复到上次保存值（未保存过时回退到默认值）

  依赖组件：
  - SettingsCard: 通用设置卡片组件
  - SettingRow: 通用设置行组件
  - SettingsActionBar: 右下角保存/重置操作栏
-->
<script setup lang="ts">
import { ref, onMounted, nextTick } from "vue";
import { useSettingsStore } from "../../stores/settings";
import { useSettingsDraft } from "../../composables/useSettingsDraft";
import SettingsCard from "./SettingsCard.vue";
import SettingRow from "./SettingRow.vue";
import SettingsActionBar from "./SettingsActionBar.vue";

const settingsStore = useSettingsStore();

type ProxySettings = Record<string, string>;

/** 默认值（数据库为空时作为基线） */
const defaults: ProxySettings = {
  proxy_download_url: "https://update.greasyfork.org/scripts/412245/Github%20%E5%A2%9E%E5%BC%BA%20-%20%E9%AB%98%E9%80%9F%E4%B8%8B%E8%BD%BD.user.js",
  proxy_test_download_url: "https://github.com/zxp19821005/My_AUR_Files/releases/latest/download/README.md",
  proxy_test_clone_url: "https://github.com/zxp19821005/My_AUR_Files.git",
  proxy_test_raw_url: "https://raw.githubusercontent.com/zxp19821005/My_AUR_Files/main/README.md",
  proxy_test_ssh_url: "ssh://git@ssh.github.com:443/zxp19821005/My_AUR_Files",
};

/** 设置项字段配置（label/description/placeholder 驱动模板渲染） */
const fields = [
  { key: "proxy_download_url", label: "代理文件下载地址", description: "用于下载代理规则 JS 文件", placeholder: "输入代理文件下载 URL" },
  { key: "proxy_test_download_url", label: "下载代理测试地址", description: "用于测试下载代理的连通性", placeholder: "输入下载代理测试 URL" },
  { key: "proxy_test_clone_url", label: "克隆代理测试地址", description: "用于测试克隆代理的连通性", placeholder: "输入克隆代理测试 URL" },
  { key: "proxy_test_raw_url", label: "RAW 代理测试地址", description: "用于测试 RAW 代理的连通性", placeholder: "输入 RAW 代理测试 URL" },
  { key: "proxy_test_ssh_url", label: "SSH 代理测试地址", description: "用于测试 SSH 代理的连通性", placeholder: "输入 SSH 代理测试 URL" },
] as const;

/** 草稿模型：初始为空，挂载后从数据库/默认值加载 */
const { draft, dirty, saving, reset, commit } = useSettingsDraft<ProxySettings>({
  proxy_download_url: "",
  proxy_test_download_url: "",
  proxy_test_clone_url: "",
  proxy_test_raw_url: "",
  proxy_test_ssh_url: "",
});

const loading = ref(false);
const message = ref("");

onMounted(async () => {
  await loadSettings();
  initTextareas();
});

/** 自动调整 textarea 高度 */
function autoResize(event: Event) {
  const el = event.target as HTMLTextAreaElement;
  el.style.height = "auto";
  el.style.height = el.scrollHeight + "px";
}

function initTextareas() {
  nextTick(() => {
    document
      .querySelectorAll(".proxy-section-root .text-input")
      .forEach((el) => {
        const ta = el as HTMLTextAreaElement;
        ta.style.height = "auto";
        ta.style.height = ta.scrollHeight + "px";
      });
  });
}

/** 加载设置（数据库值优先，缺失则用默认值作为基线） */
async function loadSettings() {
  loading.value = true;
  try {
    const loaded: ProxySettings = { ...defaults };
    for (const { key } of fields) {
      loaded[key] = await settingsStore.getSetting(key, defaults[key]);
    }
    draft.value = loaded;
    commit();
  } catch (e) {
    showMessage("加载设置失败: " + String(e));
  } finally {
    loading.value = false;
  }
}

/** 验证 URL 格式（SSH 地址非标准 URL，单独放行） */
function isValidUrl(url: string, key: string): boolean {
  if (!url) return false;
  if (key === "proxy_test_ssh_url") return url.startsWith("ssh://");
  try {
    new URL(url);
    return true;
  } catch {
    return false;
  }
}

/** 保存到数据库 */
async function handleSave() {
  saving.value = true;
  try {
    for (const { key } of fields) {
      if (!isValidUrl(draft.value[key], key)) {
        showMessage(`请输入有效的 ${key} 地址`);
        return;
      }
    }
    for (const { key } of fields) {
      await settingsStore.setSetting(key, draft.value[key]);
    }
    commit();
    showMessage("已保存");
  } catch (e) {
    showMessage("保存失败: " + String(e));
  } finally {
    saving.value = false;
  }
}

function showMessage(text: string) {
  message.value = text;
  setTimeout(() => {
    if (message.value === text) message.value = "";
  }, 3000);
}
</script>

<template>
  <div class="proxy-section-root">
    <div v-if="message" class="message">{{ message }}</div>

    <SettingsCard title="代理管理设置" description="配置代理文件下载地址和各类代理的测试地址。修改后点击右下角「保存设置」才会写入。">
      <SettingRow
        v-for="f in fields"
        :key="f.key"
        :label="f.label"
        :description="f.description"
      >
        <div class="input-row">
          <textarea
            v-model="draft[f.key]"
            class="text-input"
            :placeholder="f.placeholder"
            rows="1"
            @input="autoResize($event)"
          ></textarea>
        </div>
      </SettingRow>
    </SettingsCard>

    <SettingsActionBar
      :dirty="dirty"
      :saving="saving"
      @save="handleSave"
      @reset="reset"
    />
  </div>
</template>

<style scoped>
.proxy-section-root {
  width: 100%;
}

.message {
  padding: 0.5rem 1rem;
  margin-bottom: 1rem;
  border-radius: 6px;
  font-size: 0.875rem;
  background-color: rgba(76, 175, 125, 0.1);
  color: var(--success);
}

.input-row {
  display: flex;
  align-items: flex-start;
  gap: 0.5rem;
  flex: 1 1 auto;
  min-width: 0;
}

.text-input {
  padding: 0.375rem 0.5rem;
  border-radius: 6px;
  border: 1px solid var(--border);
  background-color: var(--bg-primary);
  color: var(--text-primary);
  font-size: 0.875rem;
  flex: 1 1 auto;
  min-width: 0;
  resize: none;
  overflow: hidden;
  font-family: inherit;
  line-height: 1.5;
  word-break: break-all;
}

.text-input:focus {
  border-color: var(--accent);
  outline: none;
}

.text-input::placeholder {
  color: var(--text-muted);
}

/* 响应式设计 - 平板及以下 */
@media (max-width: 768px) {
  .input-row {
    flex-direction: column;
  }
}
</style>
