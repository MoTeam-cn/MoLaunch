<script setup lang="ts">
/**
 * 版本设置 - 资源包/光影管理子页（顶层容器）
 *
 * 通过 `kind` prop 区分资源包/光影，业务逻辑统一在 `@/composables/usePackOperations`。
 * 子组件位于 pack-tab/（PackToolbar / PackListItem / PackEmptyState / PackUpdateDialog），
 * 详情弹窗复用社区 `@/components/community/ResourceDetail.vue`。
 */
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { useVersionSettings } from '@/composables/useVersionSettings'
import { usePackOperations } from '@/composables/usePackOperations'
import ResourceDetail from '@/components/community/ResourceDetail.vue'
import PackListItem from './pack-tab/PackListItem.vue'
import PackToolbar from './pack-tab/PackToolbar.vue'
import PackEmptyState from './pack-tab/PackEmptyState.vue'
import PackUpdateDialog from './pack-tab/PackUpdateDialog.vue'
import type { PackKind } from '@/utils/tauri'

const props = defineProps<{ kind: PackKind }>()

const router = useRouter()
const { selectedId } = useVersionSettings()
const kindRef = computed(() => props.kind)

const {
  packs, packsLoading, packFilter, packSearch,
  available, checking,
  versionGameVersion,
  filteredPacks, filterOptions, loadPacks,
  detailVisible, detailProject, detailLoadingFor,
  updatePackFor, updateVisible,
  handleToggle, handleDelete, handleInstall, handleOpenDir, handleOpenFile,
  openUpdateDialog, onShowInfo, onPackUpdated,
} = usePackOperations({ selectedId, kind: kindRef })
</script>

<template>
  <div class="flex h-full flex-col">
    <PackEmptyState
      v-if="!available && !checking"
      variant="not-modable"
      :count="0"
      :kind="kind"
      @install="handleInstall"
      @go-download="router.push('/apps/downloads')"
      @go-select="router.push('/apps/versions/select')"
    />

    <div v-else class="flex h-full flex-col">
      <PackToolbar
        v-model:pack-filter="packFilter"
        v-model:pack-search="packSearch"
        :packs-loading="packsLoading"
        :filter-options="filterOptions"
        :kind="kind"
        @install="handleInstall"
        @open-dir="handleOpenDir"
        @refresh="loadPacks"
      />

      <div class="flex-1 overflow-y-auto p-6">
        <PackEmptyState
          v-if="packsLoading && packs.length === 0"
          variant="loading"
          :count="0"
          :kind="kind"
          @install="handleInstall"
        />
        <PackEmptyState
          v-else-if="filteredPacks.length === 0"
          :variant="packs.length === 0 ? 'empty' : 'no-match'"
          :count="packs.length"
          :kind="kind"
          @install="handleInstall"
        />
        <div v-else class="overflow-hidden rounded-xl border border-gray-200 bg-white shadow-sm">
          <ul class="divide-y divide-gray-100">
            <PackListItem
              v-for="pack in filteredPacks"
              :key="pack.file_name"
              :pack="pack"
              :selected-id="selectedId"
              :kind="kind"
              :detail-loading-for="detailLoadingFor"
              @toggle="handleToggle"
              @delete="handleDelete"
              @open-file="handleOpenFile"
              @show-info="onShowInfo"
              @update="openUpdateDialog"
            />
          </ul>
        </div>
      </div>
    </div>

    <!-- 详情弹窗（复用社区资源详情组件，CF/MR 联动） -->
    <ResourceDetail
      :visible="detailVisible"
      :project="detailProject"
      :version-id="selectedId || undefined"
      :game-version="versionGameVersion || undefined"
      @close="detailVisible = false"
    />

    <!-- 更新/更改版本对话框 -->
    <PackUpdateDialog
      :visible="updateVisible"
      :pack="updatePackFor"
      :kind="kind"
      :mc-version="versionGameVersion || ''"
      :version-id="selectedId || ''"
      @update:visible="updateVisible = $event"
      @installed="onPackUpdated"
    />
  </div>
</template>
