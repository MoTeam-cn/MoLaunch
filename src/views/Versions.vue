<script setup lang="ts">
/**
 * 版本管理页面
 */

import { ref, computed, onMounted } from 'vue'
import { useVersionStore } from '@/stores/version'

const versionStore = useVersionStore()

const searchQuery = ref('')
const filterType = ref<'all' | 'release' | 'snapshot'>('all')
const loading = ref(false)

onMounted(async () => {
  if (versionStore.versions.length === 0) {
    loading.value = true
    await versionStore.fetchVersions()
    loading.value = false
  }
})

const filteredVersions = computed(() => {
  let versions = versionStore.versions

  // 按类型过滤
  if (filterType.value === 'release') {
    versions = versions.filter(v => v.version_type === 'release')
  } else if (filterType.value === 'snapshot') {
    versions = versions.filter(v => v.version_type === 'snapshot')
  }

  // 按搜索词过滤
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase()
    versions = versions.filter(v => v.id.toLowerCase().includes(query))
  }

  return versions
})

function formatDate(timestamp: number): string {
  return new Date(timestamp * 1000).toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
  })
}

function getVersionTypeBadge(versionType: string) {
  switch (versionType) {
    case 'release':
      return {
        text: '正式版',
        class: 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200',
      }
    case 'snapshot':
      return {
        text: '快照版',
        class: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200',
      }
    default:
      return {
        text: '旧版本',
        class: 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-200',
      }
  }
}
</script>

<template>
  <div class="max-w-4xl mx-auto">
    <!-- 标题 -->
    <div class="mb-6">
      <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">
        版本管理
      </h1>
      <p class="text-gray-600 dark:text-gray-400 mt-1">
        浏览和下载 Minecraft 版本
      </p>
    </div>

    <!-- 搜索和过滤 -->
    <div class="card mb-6">
      <div class="flex flex-col md:flex-row gap-4">
        <!-- 搜索框 -->
        <div class="flex-1">
          <input
            v-model="searchQuery"
            type="text"
            placeholder="搜索版本..."
            class="input"
          />
        </div>

        <!-- 过滤按钮 -->
        <div class="flex gap-2">
          <button
            @click="filterType = 'all'"
            class="px-4 py-2 rounded-lg text-sm font-medium transition-colors"
            :class="filterType === 'all'
              ? 'bg-primary-600 text-white'
              : 'bg-gray-200 text-gray-700 hover:bg-gray-300 dark:bg-gray-700 dark:text-gray-300 dark:hover:bg-gray-600'
            "
          >
            全部
          </button>
          <button
            @click="filterType = 'release'"
            class="px-4 py-2 rounded-lg text-sm font-medium transition-colors"
            :class="filterType === 'release'
              ? 'bg-green-600 text-white'
              : 'bg-gray-200 text-gray-700 hover:bg-gray-300 dark:bg-gray-700 dark:text-gray-300 dark:hover:bg-gray-600'
            "
          >
            正式版
          </button>
          <button
            @click="filterType = 'snapshot'"
            class="px-4 py-2 rounded-lg text-sm font-medium transition-colors"
            :class="filterType === 'snapshot'
              ? 'bg-yellow-600 text-white'
              : 'bg-gray-200 text-gray-700 hover:bg-gray-300 dark:bg-gray-700 dark:text-gray-300 dark:hover:bg-gray-600'
            "
          >
            快照版
          </button>
        </div>
      </div>
    </div>

    <!-- 加载状态 -->
    <div v-if="loading" class="card text-center py-12">
      <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600 mx-auto"></div>
      <p class="text-gray-600 dark:text-gray-400 mt-4">加载版本列表...</p>
    </div>

    <!-- 版本列表 -->
    <div v-else-if="filteredVersions.length > 0" class="space-y-2">
      <div
        v-for="version in filteredVersions"
        :key="version.id"
        class="card flex items-center justify-between hover:shadow-md transition-shadow"
      >
        <div class="flex items-center">
          <div>
            <div class="flex items-center">
              <span class="font-semibold text-gray-900 dark:text-gray-100">
                {{ version.id }}
              </span>
              <span
                class="ml-2 text-xs px-2 py-0.5 rounded-full"
                :class="getVersionTypeBadge(version.version_type).class"
              >
                {{ getVersionTypeBadge(version.version_type).text }}
              </span>
              <span
                v-if="version.id === versionStore.latestRelease"
                class="ml-2 text-xs px-2 py-0.5 rounded-full bg-primary-100 text-primary-800 dark:bg-primary-900 dark:text-primary-200"
              >
                最新
              </span>
            </div>
            <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">
              发布于 {{ formatDate(version.release_time) }}
            </p>
          </div>
        </div>
        <button class="btn-primary text-sm">
          <svg class="w-4 h-4 mr-1 inline" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
          </svg>
          下载
        </button>
      </div>
    </div>

    <!-- 空状态 -->
    <div v-else class="card text-center py-12">
      <svg class="w-16 h-16 text-gray-400 mx-auto" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9.172 16.172a4 4 0 015.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
      </svg>
      <p class="text-gray-600 dark:text-gray-400 mt-4">
        {{ searchQuery ? '未找到匹配的版本' : '暂无版本数据' }}
      </p>
    </div>
  </div>
</template>
