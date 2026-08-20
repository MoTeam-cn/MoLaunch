<script setup lang="ts">
/**
 * 设置-联机 - GitHub 镜像源编辑器（easytier 等外部下载竞速选源）
 */
import { onMounted, ref, defineAsyncComponent } from 'vue'
const Input = defineAsyncComponent(() => import('@/components/common/Input.vue'))
const Select = defineAsyncComponent(() => import('@/components/common/Select.vue'))
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Tooltip = defineAsyncComponent(() => import('@/components/common/Tooltip.vue'))
import { toastError, toastSuccess } from '@/utils/toast'
import { getConfigMap, refreshConfig } from '@/utils/tauri'
import { restoreDefaultProxies } from '@/utils/githubProxy'
import { setGithubProxies } from '@/utils/api/online-manager/easytier'
import { ArrowPathIcon, CheckIcon, CloudIcon, PlusIcon } from '@heroicons/vue/24/outline'

/** 编辑行（name 必填字符串，便于 v-model 绑定） */
interface ProxyRow {
  name: string
  type: 'path' | 'full'
  base: string
}

const githubProxies = ref<ProxyRow[]>([])
const proxiesLoaded = ref(false)
/** 重新测速 / 保存 独立加载态（互不干扰：点重新测速时保存按钮保持可用） */
const probeBusy = ref(false)
const saveBusy = ref(false)

/** type 选项（full: 镜像 + 完整 GitHub URL / path: 镜像前缀 + GitHub 路径），文案精简以适配窄列 */
const proxyTypeOptions = [
  { label: '追加路径', value: 'path' },
  { label: '完整 URL', value: 'full' },
]

async function loadGithubProxies() {
  try {
    const cfg = await getConfigMap(true)
    githubProxies.value = (cfg.onlineGithubProxies ?? []).map((p) => ({
      name: p.name ?? '',
      type: p.type,
      base: p.base,
    }))
  } catch (e) {
    console.error('加载 GitHub 镜像源失败', e)
  } finally {
    proxiesLoaded.value = true
  }
}

function addProxyRow() {
  githubProxies.value.push({ name: '', type: 'path', base: '' })
}

function removeProxyRow(index: number) {
  githubProxies.value.splice(index, 1)
}

/** 保存：过滤空 base 行，写运行时缓存 + 持久化配置 */
async function handleSaveProxies() {
  const list = githubProxies.value
    .filter((p) => p.base.trim().length > 0)
    .map((p) => ({ name: p.name?.trim() || undefined, type: p.type, base: p.base.trim() }))
  if (list.length === 0) {
    toastError('至少保留一个镜像源')
    return
  }
  saveBusy.value = true
  try {
    await setGithubProxies(list)
    refreshConfig()
    toastSuccess('镜像源已保存')
  } catch (e) {
    toastError(`保存失败: ${e}`)
  } finally {
    saveBusy.value = false
  }
}

/** 重新测速：对 githubProxy.json 全部源并发探测，筛选最快的前 10 个替换当前列表 */
async function handleRestoreDefaultProxies() {
  probeBusy.value = true
  try {
    const list = await restoreDefaultProxies()
    if (list.length === 0) {
      toastError('默认镜像源均不可用，请稍后重试')
    } else {
      githubProxies.value = list.map((p) => ({ name: p.name ?? '', type: p.type, base: p.base }))
      refreshConfig()
      toastSuccess(`已重新测速，筛选出 ${list.length} 个可用镜像源`)
    }
  } catch (e) {
    toastError(`重新测速失败: ${e}`)
  } finally {
    probeBusy.value = false
  }
}

onMounted(() => {
  void loadGithubProxies()
})
</script>

<template>
  <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
    <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">GitHub 镜像源</h3>
    <div class="px-5 pb-5">
      <p class="text-xs text-gray-500">
        easytier 内核等外部文件下载时按镜像优先、官方保底竞速选源。path 模式为「镜像前缀 + GitHub 路径」，full 模式为「镜像 + 完整 GitHub URL」。
      </p>
      <div v-if="!proxiesLoaded" class="mt-3 h-20 bg-gray-100 rounded animate-pulse" />
      <template v-else>
        <!-- 表头：名称/类型固定，镜像地址 minmax(0,1fr) 自适应填满剩余宽度（长地址不被截断），
             第 4 列 auto 仅占删除按钮内容宽、紧跟其后，无中间留白。
             列宽与数据行保持一致（名称 13rem），保证标签与 input 对齐 -->
        <div
          class="mt-3 grid grid-cols-[13rem_7rem_minmax(0,1fr)_auto] items-center gap-2 text-xs text-gray-400"
        >
          <span>名称</span>
          <span>类型</span>
          <span>镜像地址</span>
          <span />
        </div>
        <div
          v-for="(p, i) in githubProxies"
          :key="i"
          class="mt-2 grid grid-cols-[13rem_7rem_minmax(0,1fr)_auto] items-center gap-2"
        >
          <Input v-model="p.name" placeholder="可选" size="small" width="100%" />
          <Select v-model="p.type" :options="proxyTypeOptions" />
          <Input v-model="p.base" placeholder="https://mirror.example.com" size="small" width="100%" />
          <Button type="text" size="small" @click="removeProxyRow(i)">删除</Button>
        </div>
        <div
          v-if="githubProxies.length === 0"
          class="mt-3 py-8 flex flex-col items-center justify-center"
        >
          <CloudIcon class="w-8 h-8 text-gray-300" />
          <p class="mt-2 text-xs text-gray-400">暂无镜像源，可添加自定义或恢复默认</p>
        </div>
        <div class="mt-4 flex items-center justify-between gap-2">
          <Button type="outline" size="small" :disabled="probeBusy || saveBusy" @click="addProxyRow">
            <template #icon><PlusIcon class="w-4 h-4" /></template>
            添加镜像
          </Button>
          <div class="flex items-center gap-2">
            <Tooltip text="对内置镜像源清单全部并发测速，筛选响应最快的前 10 个替换当前列表" position="top">
              <Button type="outline" size="small" :loading="probeBusy" @click="handleRestoreDefaultProxies">
                <template #icon><ArrowPathIcon class="w-4 h-4" /></template>
                重新测速
              </Button>
            </Tooltip>
            <Button type="primary" size="small" :loading="saveBusy" @click="handleSaveProxies">
              <template #icon><CheckIcon class="w-4 h-4" /></template>
              保存
            </Button>
          </div>
        </div>
      </template>
    </div>
  </div>
</template>