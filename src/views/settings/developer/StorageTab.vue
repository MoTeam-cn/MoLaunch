<script setup lang="ts">
/**
 * 开发者 - 存储信息子页签
 *
 * 展示缓存目录路径和存储信息路径，每项带「打开」或「定位」按钮。
 * 数据由父组件 SettingsDeveloper.vue 统一加载后通过 props 下发。
 */
import { computed } from 'vue'
import type { StorageDirs } from '@/utils/api/developer'
import { openPath, revealInExplorer } from '@/utils/api/system'
import { toastError } from '@/utils/toast'
import Button from '@/components/common/Button.vue'
import {
  FolderOpenIcon,
  DocumentTextIcon,
} from '@heroicons/vue/24/outline'

const props = defineProps<{
  storageDirs: StorageDirs | null
}>()

/** 缓存卡片条目（运行路径缓存 / 临时目录 / 系统临时缓存 / Minecraft 运行缓存） */
const cacheEntries = computed<{ label: string; path: string }[]>(() => {
  if (!props.storageDirs) return []
  return [
    { label: '运行路径缓存', path: props.storageDirs.cache },
    { label: '运行路径临时', path: props.storageDirs.temp },
    { label: '系统临时缓存', path: props.storageDirs.cacheTemp },
    { label: 'Minecraft 运行缓存', path: props.storageDirs.cacheApp },
  ]
})

/** 存储信息卡片条目（数据根目录 / 配置文件 / 日志目录） */
const storageEntries = computed<{ label: string; path: string; locate?: boolean }[]>(() => {
  if (!props.storageDirs) return []
  return [
    { label: '数据根目录', path: props.storageDirs.base },
    { label: '配置文件', path: props.storageDirs.config, locate: true },
    { label: '日志目录', path: props.storageDirs.logs },
  ]
})

/** AppData 全局共享卡片条目（环境变量缺失时路径为空串，过滤跳过） */
const appdataEntries = computed<{ label: string; path: string; locate?: boolean }[]>(() => {
  if (!props.storageDirs) return []
  return [
    { label: '全局共享根目录', path: props.storageDirs.appdataRoot },
    { label: 'TLS 证书目录', path: props.storageDirs.appdataCerts },
    { label: 'frpc 厂商二进制', path: props.storageDirs.appdataProviders },
    { label: 'FRP 认证 token', path: props.storageDirs.appdataFrpAuth },
    { label: '联机数据', path: props.storageDirs.appdataOnline },
    { label: '账号认证文件', path: props.storageDirs.appdataAuthFile, locate: true },
  ].filter((entry) => entry.path.length > 0)
})

async function openDir(path: string) {
  try {
    await openPath(path)
  } catch (e) {
    toastError('打开目录失败：' + e)
  }
}

async function locateFile(path: string) {
  try {
    await revealInExplorer(path)
  } catch (e) {
    toastError('定位文件失败：' + e)
  }
}

async function handleClick(entry: { path: string; locate?: boolean }) {
  if (entry.locate) {
    await locateFile(entry.path)
  } else {
    await openDir(entry.path)
  }
}
</script>

<template>
  <div v-if="storageDirs" class="space-y-6">
    <!-- 缓存目录 -->
    <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
      <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">缓存目录</h3>
      <div class="divide-y divide-gray-200">
        <div
          v-for="entry in cacheEntries"
          :key="entry.label"
          class="px-5 py-3 flex items-center justify-between"
        >
          <div>
            <p class="text-sm text-gray-500">{{ entry.label }}</p>
            <p class="text-xs text-gray-900 font-mono mt-1 break-all">{{ entry.path }}</p>
          </div>
          <Button
            type="outline"
            size="small"
            class="shrink-0 ml-4"
            @click="openDir(entry.path)"
          >
            <template #icon><FolderOpenIcon class="w-3.5 h-3.5" /></template>
            打开
          </Button>
        </div>
      </div>
    </div>

    <!-- 存储信息 -->
    <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
      <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">存储信息</h3>
      <div class="divide-y divide-gray-200">
        <div
          v-for="entry in storageEntries"
          :key="entry.label"
          class="px-5 py-3 flex items-center justify-between"
        >
          <div>
            <p class="text-sm text-gray-500">{{ entry.label }}</p>
            <p class="text-xs text-gray-900 font-mono mt-1 break-all">{{ entry.path }}</p>
          </div>
          <Button
            type="outline"
            size="small"
            class="shrink-0 ml-4"
            @click="handleClick(entry)"
          >
            <template #icon>
              <component :is="entry.locate ? DocumentTextIcon : FolderOpenIcon" class="w-3.5 h-3.5" />
            </template>
            {{ entry.locate ? '定位' : '打开' }}
          </Button>
        </div>
      </div>
    </div>

    <!-- AppData 全局共享 -->
    <div v-if="appdataEntries.length > 0" class="bg-white rounded-lg border border-gray-300 overflow-hidden">
      <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">AppData 全局共享</h3>
      <div class="divide-y divide-gray-200">
        <div
          v-for="entry in appdataEntries"
          :key="entry.label"
          class="px-5 py-3 flex items-center justify-between"
        >
          <div>
            <p class="text-sm text-gray-500">{{ entry.label }}</p>
            <p class="text-xs text-gray-900 font-mono mt-1 break-all">{{ entry.path }}</p>
          </div>
          <Button
            type="outline"
            size="small"
            class="shrink-0 ml-4"
            @click="handleClick(entry)"
          >
            <template #icon>
              <component :is="entry.locate ? DocumentTextIcon : FolderOpenIcon" class="w-3.5 h-3.5" />
            </template>
            {{ entry.locate ? '定位' : '打开' }}
          </Button>
        </div>
      </div>
    </div>
  </div>
</template>
