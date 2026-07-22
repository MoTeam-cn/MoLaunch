<script setup lang="ts">
/**
 * 资源包转换
 *
 * 列举 {game_dir}/resourcepacks/ 下的资源包（zip 文件 / 文件夹），
 * 支持在 zip ↔ folder 两种格式间转换。
 * - folder → zip：把目录内容打包为同名 .zip
 * - zip → folder：解压到同名目录
 * 转换走 showConfirm 回调式，目标已存在时后端返回失败提示。
 */
import { ref, onMounted } from 'vue'
import {
  Square3Stack3DIcon,
  ArrowPathIcon,
  ArrowPathRoundedSquareIcon,
  CheckCircleIcon,
} from '@heroicons/vue/24/outline'
import Button from '@/components/common/Button.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import { toastSuccess, toastError, toastInfo } from '@/utils/toast'
import { showConfirm } from '@/utils/modal'
import { resourcepackList, resourcepackConvert } from '@/utils/api/tools'
import type { ResourcePackItem } from '@/utils/api/tools'
import { formatBytes } from '@/utils/format'

const items = ref<ResourcePackItem[]>([])
const loading = ref(false)
const converting = ref<string | null>(null)
const loaded = ref(false)

async function loadList() {
  loading.value = true
  try {
    const res = await resourcepackList()
    items.value = res.items
    loaded.value = true
  } catch (e) {
    toastError(`加载资源包列表失败: ${e instanceof Error ? e.message : String(e)}`)
  } finally {
    loading.value = false
  }
}

function requestConvert(item: ResourcePackItem) {
  const target = item.format === 'zip' ? 'folder' : 'zip'
  const targetLabel = target === 'zip' ? 'ZIP 压缩包' : '文件夹'
  showConfirm(
    '确认转换资源包格式',
    `将把「${item.name}」从 ${item.format.toUpperCase()} 转换为 ${targetLabel}。若目标已存在将取消转换。`,
    () => doConvert(item, target),
  )
}

async function doConvert(item: ResourcePackItem, target: 'zip' | 'folder') {
  converting.value = item.path
  try {
    const res = await resourcepackConvert(item.path, target)
    if (res.success) {
      toastSuccess(`转换成功：${res.output_path}`)
      await loadList()
    } else {
      toastInfo(res.message || '转换未完成')
    }
  } catch (e) {
    toastError(`转换失败: ${e instanceof Error ? e.message : String(e)}`)
  } finally {
    converting.value = null
  }
}
</script>

<template>
  <section class="rounded-lg border border-gray-300 bg-white">
    <div class="flex items-center gap-2 px-5 pt-5 pb-3">
      <Square3Stack3DIcon class="h-5 w-5 text-gray-700" />
      <h3 class="text-sm font-semibold text-gray-900">资源包转换</h3>
      <span class="ml-auto text-xs text-gray-400">{{ items.length }} 个资源包</span>
      <Button type="outline" size="small" :loading="loading" @click="loadList">
        <template #icon><ArrowPathIcon class="h-4 w-4" /></template>
        刷新
      </Button>
    </div>
    <div class="px-5 pb-5 space-y-3">
      <p class="text-xs text-gray-500">
        在 ZIP 压缩包与文件夹两种资源包格式间互转，便于编辑或分发。目标同名条目已存在时将跳过转换。
      </p>

      <!-- 资源包列表 -->
      <div v-if="items.length > 0" class="max-h-[400px] overflow-y-auto rounded-lg border border-gray-200 divide-y divide-gray-100">
        <div
          v-for="item in items"
          :key="item.path"
          class="flex items-center gap-3 px-3 py-2.5"
        >
          <Square3Stack3DIcon class="h-5 w-5 flex-none text-gray-400" />
          <Tooltip :text="item.path" position="top" :delay="200" block>
            <div class="flex-1 min-w-0">
              <div class="truncate text-sm font-medium text-gray-900">{{ item.name }}</div>
              <div class="text-xs text-gray-400">{{ formatBytes(item.size) }}</div>
            </div>
          </Tooltip>
          <span
            class="flex-none rounded-full px-2 py-0.5 text-xs font-medium"
            :class="item.format === 'zip' ? 'bg-blue-100 text-blue-700' : 'bg-purple-100 text-purple-700'"
          >
            {{ item.format === 'zip' ? 'ZIP' : '文件夹' }}
          </span>
          <Button
            type="outline"
            size="small"
            :loading="converting === item.path"
            @click="requestConvert(item)"
          >
            <template #icon><ArrowPathRoundedSquareIcon class="h-4 w-4" /></template>
            转为{{ item.format === 'zip' ? '文件夹' : 'ZIP' }}
          </Button>
        </div>
      </div>

      <!-- 空状态 -->
      <div
        v-else-if="loaded"
        class="flex flex-col items-center justify-center py-8 text-gray-400"
      >
        <CheckCircleIcon class="h-8 w-8 mb-2 text-green-400" />
        <span class="text-xs">资源包目录为空</span>
      </div>
    </div>
  </section>
</template>
