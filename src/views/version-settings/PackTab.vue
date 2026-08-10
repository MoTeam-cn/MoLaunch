<script setup lang="ts">
/**
 * 版本设置 - 资源包/光影管理子页（顶层容器）
 *
 * 通过 `kind` prop 区分资源包/光影，业务逻辑统一在 `@/composables/usePackOperations`。
 * 子组件位于 pack-tab/（PackToolbar / PackListItem / PackEmptyState）。
 */
import { computed } from 'vue'
import { useVersionSettings } from '@/composables/useVersionSettings'
import { usePackOperations } from '@/composables/usePackOperations'
import PackListItem from './pack-tab/PackListItem.vue'
import PackToolbar from './pack-tab/PackToolbar.vue'
import PackEmptyState from './pack-tab/PackEmptyState.vue'
import type { PackKind } from '@/utils/tauri'

const props = defineProps<{ kind: PackKind }>()

const { selectedId } = useVersionSettings()
const kindRef = computed(() => props.kind)

const {
  packs, packsLoading, packFilter, packSearch,
  available, checking,
  filteredPacks, filterOptions, loadPacks,
  handleToggle, handleDelete, handleInstall, handleOpenDir, handleOpenFile,
} = usePackOperations({ selectedId, kind: kindRef })
</script>

<template>
  <div class="flex h-full flex-col">
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

    <div class="flex-1 overflow-y-auto">
      <PackEmptyState
        v-if="(!available && !checking) || packsLoading"
        :variant="packsLoading && packs.length === 0 ? 'loading' : 'empty'"
        :count="0"
        :kind="kind"
        @install="handleInstall"
      />
      <PackEmptyState
        v-else-if="filteredPacks.length === 0"
        variant="empty"
        :count="packs.length"
        :kind="kind"
        @install="handleInstall"
      />
      <ul v-else class="divide-y divide-gray-100">
        <PackListItem
          v-for="pack in filteredPacks"
          :key="pack.file_name"
          :pack="pack"
          :selected-id="selectedId"
          :kind="kind"
          @toggle="handleToggle"
          @delete="handleDelete"
          @open-file="handleOpenFile"
        />
      </ul>
    </div>
  </div>
</template>
