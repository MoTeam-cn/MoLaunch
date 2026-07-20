<script setup lang="ts">
/**
 * 社区资源内容区
 * 搜索栏 + 单列结果列表 + 分页 + 详情弹窗
 */

import { ref, watch } from 'vue'
import type { ResourceType, ResourceProject, SearchResult } from '@/types/community'
import { searchResources } from '@/utils/api/community'
import { showError } from '@/utils/toast'
import { useVersionStore } from '@/stores/version'
import { useSearchProgress } from '@/composables/useSearchProgress'
import SearchBar from '@/components/community/SearchBar.vue'
import Pagination from '@/components/community/Pagination.vue'
import ResourceCard from '@/components/community/ResourceCard.vue'
import ResourceDetail from '@/components/community/ResourceDetail.vue'

const props = defineProps<{ resourceType: ResourceType }>()
const versionStore = useVersionStore()

const query = ref('')
const gameVersion = ref('')
const modLoader = ref(0)
const source = ref(0)
const category = ref('')

const projects = ref<ResourceProject[]>([])
const total = ref(0)
const page = ref(0)
const pageSize = ref(40)
const loading = ref(false)

const detailVisible = ref(false)
const detailProject = ref<ResourceProject | null>(null)

const { stage: searchStage, percent: searchPercent, slowMerging, stageText, start, finish, fail } = useSearchProgress()

/** 执行搜索 */
async function doSearch() {
  loading.value = true
  start(source.value)
  try {
    const result: SearchResult = await searchResources({
      query: query.value,
      resourceType: props.resourceType,
      gameVersion: gameVersion.value || undefined,
      modLoader: modLoader.value,
      source: source.value,
      category: category.value || undefined,
      page: page.value,
    })
    projects.value = result.projects
    total.value = result.total_count
    pageSize.value = result.page_size
    finish()
  } catch (e: any) {
    showError('搜索失败: ' + (e?.message || String(e)))
    projects.value = []
    total.value = 0
    fail()
  } finally {
    loading.value = false
    setTimeout(() => { searchStage.value = 'idle' }, 600)
  }
}

function onPageChange(p: number) {
  page.value = p
  doSearch()
}

function openDetail(p: ResourceProject) {
  detailProject.value = p
  detailVisible.value = true
}

/** 重置筛选条件 */
function onReset() {
  query.value = ''
  gameVersion.value = ''
  modLoader.value = 0
  source.value = 0
  category.value = ''
  page.value = 0
  doSearch()
}

watch(() => props.resourceType, () => {
  page.value = 0
  category.value = ''
  doSearch()
}, { immediate: true })
</script>

<template>
  <div class="h-full flex flex-col">
    <!-- 搜索区 -->
    <div class="px-4 py-3 bg-white border-b border-gray-100 shrink-0">
      <SearchBar
        v-model:query="query"
        :resource-type="resourceType"
        v-model:game-version="gameVersion"
        v-model:mod-loader="modLoader"
        v-model:source="source"
        v-model:category="category"
        @search="page = 0; doSearch()"
        @reset="onReset"
      />
    </div>

    <!-- 结果区域 -->
    <div class="flex-1 overflow-y-auto p-3" style="background-color: #f5f7fa">
      <!-- 加载中：进度条展示（参考 LaunchLog.vue） -->
      <div v-if="loading" class="flex flex-col items-center justify-center py-20">
        <svg class="mb-5 h-10 w-10 animate-spin text-primary-500" viewBox="0 0 24 24" fill="none">
          <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" class="opacity-25" />
          <path d="M12 2a10 10 0 0 1 10 10" stroke="currentColor" stroke-width="3" stroke-linecap="round" />
        </svg>
        <h2 class="mb-1 text-base font-medium text-gray-700">正在搜索资源</h2>
        <p class="mb-6 text-xs text-gray-400">{{ query || '全部 ' + resourceType }}</p>

        <div class="w-full max-w-md">
          <div class="h-1.5 overflow-hidden rounded-full bg-gray-100">
            <div
              class="h-full rounded-full bg-gradient-to-r from-primary-400 to-primary-600 transition-all duration-300 ease-out"
              :style="{ width: Math.min(100, searchPercent) + '%' }"
            />
          </div>
          <div class="mt-2 flex items-center justify-between text-xs">
            <span class="text-gray-500">{{ stageText }}</span>
            <span class="font-medium text-primary-600">{{ searchPercent.toFixed(1) }}%</span>
          </div>
          <!-- 超过 5s 且在合并阶段，显示灰色小字提示 -->
          <p v-if="slowMerging" class="mt-2 text-center text-[11px] text-gray-400">
            资源有点多，稍安勿躁，静候处理
          </p>
        </div>
      </div>

      <!-- 空结果 -->
      <div v-else-if="projects.length === 0" class="flex flex-col items-center justify-center py-20 text-gray-400">
        <span class="text-4xl mb-3">🔍</span>
        <span class="text-sm">未找到匹配的资源</span>
      </div>

      <!-- 结果列表：单列 -->
      <div v-else class="max-w-4xl mx-auto space-y-1">
        <ResourceCard
          v-for="p in projects"
          :key="p.platform + '-' + p.id"
          :project="p"
          @click="openDetail"
        />
      </div>

      <Pagination
        :page="page"
        :total="total"
        :page-size="pageSize"
        @change="onPageChange"
      />
    </div>

    <ResourceDetail
      :visible="detailVisible"
      :project="detailProject"
      :version-id="versionStore.selectedVersion || undefined"
      @close="detailVisible = false"
    />
  </div>
</template>
