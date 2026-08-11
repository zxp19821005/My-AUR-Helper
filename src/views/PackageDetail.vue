<!--
  PackageDetail.vue - 软件包详情页面

  功能：
  - 显示软件包详细信息（基本信息、AUR信息、上游信息）
  - 支持编辑、删除、同步AUR、同步PKGBUILD、检查上游
  - 支持上一个/下一个导航

  使用组件：
  - PackageBasicInfoCard: 基本信息卡片
  - PackageAurInfoCard: AUR 信息卡片
  - PackageUpstreamInfoCard: 上游信息卡片
  - StandardizedButton: 操作按钮
  - StandardizedBadge: 状态徽章
  - StandardizedMessage: 消息提示
-->
<script setup lang="ts">
import { ref, onMounted, inject } from "vue";
import { useRoute, useRouter } from "vue-router";
import { invoke } from "@tauri-apps/api/core";
import { FOOTER_KEY } from "../composables/footer";
import { useSoftwareForm } from "../composables/useSoftwareForm";
import SoftwareFormModal from "../components/package/SoftwareFormModal.vue";
import PackageBasicInfoCard from "../components/package/PackageBasicInfoCard.vue";
import PackageAurInfoCard from "../components/package/PackageAurInfoCard.vue";
import PackageUpstreamInfoCard from "../components/package/PackageUpstreamInfoCard.vue";
import PackageDetailFooter from "../components/package/PackageDetailFooter.vue";
import NavPager from "../components/common/NavPager.vue";
import StandardizedBadge from "../components/base/StandardizedBadge.vue";
import StandardizedMessage from "../components/base/StandardizedMessage.vue";

const route = useRoute();
const router = useRouter();
const footer = inject(FOOTER_KEY)!;

const { error, detail, save, init } = useSoftwareForm();

const showEditModal = ref(false);
const deleting = ref(false);
const updatingAur = ref(false);
const updatingPkgbuild = ref(false);
const checking = ref(false);
const successMsg = ref("");
const prevPkgname = ref<string | null>(null);
const nextPkgname = ref<string | null>(null);

function syncFooter() {
  if (detail.value) {
    footer.infoText = `${detail.value.pkgname}  |  ${detail.value.is_outdated ? "需更新" : "已最新"}`;
  }
}

async function loadNav(pkgname: string) {
  try {
    const [prev, next] = await invoke<[string | null, string | null]>("get_prev_next_software", { pkgname });
    prevPkgname.value = prev;
    nextPkgname.value = next;
  } catch {
    // ignore
  }
}

onMounted(async () => {
  const pkgname = route.params.pkgname as string;
  await init("edit", pkgname);
  await loadNav(pkgname);
  syncFooter();
});

async function handleEditSave() {
  await save("edit");
  if (!error.value && detail.value) {
    await init("edit", detail.value.pkgname);
    syncFooter();
  }
}

function navigate(direction: "prev" | "next") {
  const pkgname = direction === "prev" ? prevPkgname.value : nextPkgname.value;
  if (pkgname) {
    router.push(`/packages/${pkgname}`);
  }
}
</script>

<template>
  <div>
    <!-- 顶部导航栏 -->
    <div class="detail-header">
      <div class="header-left">
        <StandardizedButton
          variant="outline"
          size="sm"
          @click="router.push('/packages')"
        >
          ← 返回
        </StandardizedButton>
      </div>
      <div class="header-center">
        <h1 class="modal-title">
          {{ detail?.pkgname || "加载中..." }}
          <StandardizedBadge
            v-if="detail"
            :type="detail.is_outdated ? 'warning' : 'success'"
            :text="detail.is_outdated ? '需更新' : '已最新'"
            size="sm"
          />
        </h1>
      </div>
      <div class="header-right">
        <NavPager variant="inline" show-labels :prev="prevPkgname" :next="nextPkgname" @navigate="navigate" />
      </div>
    </div>

    <!-- 消息提示 -->
    <div class="message-container">
      <StandardizedMessage
        v-if="error"
        type="error"
        :message="error"
        :duration="0"
        closable
        @close="error = ''"
      />
      <StandardizedMessage
        v-if="successMsg"
        type="success"
        :message="successMsg"
        :duration="3000"
        @close="successMsg = ''"
      />
    </div>

    <!-- 详情卡片区域 -->
    <div v-if="detail" class="detail-content">
      <div class="detail-cards">
        <PackageBasicInfoCard :detail="detail" />
        <PackageAurInfoCard :detail="detail" />
        <PackageUpstreamInfoCard :detail="detail" />
      </div>
    </div>

    <!-- 加载状态 -->
    <div v-else-if="!error" class="detail-content">
      <div class="card empty-card">加载中...</div>
    </div>

    <!-- 底部操作栏 -->
    <PackageDetailFooter
      v-if="detail"
      :detail="detail"
      v-model:deleting="deleting"
      v-model:updating-aur="updatingAur"
      v-model:updating-pkgbuild="updatingPkgbuild"
      v-model:checking="checking"
      v-model:success-msg="successMsg"
      v-model:error="error"
      @edit="showEditModal = true"
      @update-aur="init('edit', detail.pkgname).then(syncFooter)"
      @update-pkgbuild="init('edit', detail.pkgname).then(syncFooter)"
      @check-update="init('edit', detail.pkgname).then(syncFooter)"
      @deleted="router.push('/packages')"
    />

    <!-- 编辑弹窗 -->
    <SoftwareFormModal
      :show="showEditModal"
      mode="edit"
      :pkgname="detail?.pkgname"
      @close="showEditModal = false"
      @saved="handleEditSave"
    />
  </div>
</template>

<style scoped>
.detail-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.75rem 1.25rem;
  border-bottom: 1px solid var(--border);
  background-color: var(--bg-primary);
}

.header-left, .header-right { display: flex; align-items: center; gap: 0.5rem; }
.header-center { flex: 1; text-align: center; }

.modal-title {
  display: inline-flex;
  align-items: center;
  gap: 0.75rem;
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
  white-space: normal;
  overflow: visible;
  text-overflow: clip;
}

.message-container {
  padding: 0.75rem 1.25rem;
  max-width: 900px;
  margin: 0 auto;
}

.detail-content {
  padding: 1.25rem;
  max-width: 900px;
  margin: 0 auto;
}

.detail-cards {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
  gap: 1rem;
  margin-top: 0.5rem;
}

.empty-card { padding: 2rem; text-align: center; color: var(--text-secondary); }

@media (max-width: 768px) {
  .detail-header {
    flex-direction: column;
    gap: 0.75rem;
    padding: 0.75rem;
  }

  .header-center {
    order: -1;
  }

  .modal-title {
    font-size: 1rem;
  }

  .detail-cards {
    grid-template-columns: 1fr;
  }
}
</style>