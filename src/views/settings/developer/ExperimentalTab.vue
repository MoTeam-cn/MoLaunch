<script setup lang="ts">
/**
 * 开发者 - 实验性功能子页签
 *
 * 包含更新检测分支、检查更新源切换，以及下载侧实验开关
 * （受限 CurseForge 文件地址推导）。
 */
import { ref, onMounted, defineAsyncComponent } from 'vue'
import {
  getUpdateBranch,
  setUpdateBranch,
  getUpdateSource,
  setUpdateSource,
  type UpdateBranch,
  type UpdateSource,
} from '@/utils/api/developer'
import { applyConfig, getConfigMap } from '@/utils/api/config'
import { getCurrentChannel, getVersionInfo } from '@/utils/version'
import { toastError, toastInfo } from '@/utils/toast'
import { safeCall } from '@/utils/async'
const Select = defineAsyncComponent(() => import('@/components/common/Select.vue'))
const Checkbox = defineAsyncComponent(() => import('@/components/common/Checkbox.vue'))

// ==================== 更新检测分支 ====================
const versionInfo = getVersionInfo()

/** 与后端 channel_name 一致的版本后缀 → 更新分支推导 */
function derivedBranch(): string {
  const c = getCurrentChannel()
  if (c === 'rc' || c === 'beta') return 'beta'
  if (c === 'alpha' || c === 'canary') return 'alpha'
  return 'stable'
}

const branchLabels: Record<string, string> = {
  auto: '跟随版本',
  stable: 'Stable 正式版',
  beta: 'Beta 测试版',
  alpha: 'Alpha 内测版',
}

const branchOptions = Object.entries(branchLabels).map(([value, label]) => ({ label, value }))

const updateBranch = ref<UpdateBranch>('auto')

async function onUpdateBranchChange(v: string | number) {
  const branch = String(v) as UpdateBranch
  try {
    await setUpdateBranch(branch)
    updateBranch.value = branch
    toastInfo(
      branch === 'auto' ? '已恢复跟随版本推导更新分支' : `已切换到 ${branchLabels[branch]} 分支`,
    )
  } catch (e) {
    toastError('切换更新分支失败：' + e)
    updateBranch.value = await getUpdateBranch()
  }
}

// ==================== 检查更新源 ====================
const sourceLabels: Record<string, string> = {
  api: '云端（默认）',
  github: 'GitHub Releases',
  auto: '自动（云端优先）',
}

const sourceOptions = Object.entries(sourceLabels).map(([value, label]) => ({ label, value }))

const updateSource = ref<UpdateSource>('api')

async function onUpdateSourceChange(v: string | number) {
  const source = String(v) as UpdateSource
  try {
    await setUpdateSource(source)
    updateSource.value = source
    toastInfo(`检查更新源已切换为 ${sourceLabels[source]}`)
  } catch (e) {
    toastError('切换检查更新源失败：' + e)
    updateSource.value = await getUpdateSource()
  }
}

// ==================== 受限 CurseForge 文件地址推导 ====================
/** 是否允许推导被作者限制分发的 CurseForge 文件 CDN 地址（默认关闭） */
const cfDeriveUrl = ref(false)

async function onCfDeriveUrlChange(next: boolean) {
  try {
    await applyConfig({ cfDeriveUrl: next })
    cfDeriveUrl.value = next
    toastInfo(next ? '已开启受限 CurseForge 文件地址推导' : '已关闭受限 CurseForge 文件地址推导')
  } catch (e) {
    toastError('切换受限文件推导失败：' + e)
    const cfg = await getConfigMap(true)
    cfDeriveUrl.value = cfg.cfDeriveUrl
  }
}

onMounted(async () => {
  await safeCall(
    async () => {
      updateBranch.value = await getUpdateBranch()
      updateSource.value = await getUpdateSource()
      const cfg = await getConfigMap()
      cfDeriveUrl.value = cfg.cfDeriveUrl
    },
    'load developer config',
    () => toastError('加载开发者配置失败'),
  )
})
</script>

<template>
  <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
    <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">实验性功能</h3>
    <div class="divide-y divide-gray-200">
      <!-- 更新检测分支 -->
      <div class="px-5 py-4">
        <div class="flex items-center justify-between gap-4">
          <div class="min-w-0">
            <p class="text-sm font-medium text-gray-900">更新检测分支</p>
            <p class="text-xs text-gray-500 mt-0.5">
              切换检查更新时使用的发布分支（channel），不设置时跟随当前版本自动推导
            </p>
          </div>
          <div class="flex-none w-40">
            <Select
              :model-value="updateBranch"
              :options="branchOptions"
              @update:model-value="onUpdateBranchChange"
            />
          </div>
        </div>
        <p class="text-xs text-gray-400 mt-2">
          当前版本 {{ versionInfo.raw }}（自动推导 {{ derivedBranch() }} 分支）；
          {{ updateBranch === 'auto' ? '更新检测分支跟随版本自动推导' : `已切换到 ${branchLabels[updateBranch]} 分支` }}
        </p>
      </div>
      <!-- 检查更新源 -->
      <div class="px-5 py-4">
        <div class="flex items-center justify-between gap-4">
          <div class="min-w-0">
            <p class="text-sm font-medium text-gray-900">检查更新源</p>
            <p class="text-xs text-gray-500 mt-0.5">
              云端通道按设备灰度、需要设备凭证；GitHub 通道读取 Release 里的静态清单，不需要凭证
            </p>
          </div>
          <div class="flex-none w-40">
            <Select
              :model-value="updateSource"
              :options="sourceOptions"
              @update:model-value="onUpdateSourceChange"
            />
          </div>
        </div>
        <p class="text-xs text-gray-400 mt-2">
          {{
            updateSource === 'auto'
              ? '云端优先，失败时自动降级到 GitHub 清单'
              : `当前使用 ${sourceLabels[updateSource]} 检查更新`
          }}
        </p>
      </div>
    </div>
  </div>

  <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
    <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">下载实验开关</h3>
    <div class="divide-y divide-gray-200">
      <!-- 受限 CurseForge 文件地址推导 -->
      <div class="px-5 py-4">
        <div class="flex items-center justify-between gap-4">
          <div class="min-w-0">
            <p class="text-sm font-medium text-gray-900">自动推导受限 CurseForge 文件地址</p>
            <p class="text-xs text-gray-500 mt-0.5">
              作者禁止第三方分发时 CurseForge 不返回下载地址，开启后按 fileId 推导 CDN 地址直接下载
            </p>
          </div>
          <div class="flex-none">
            <Checkbox :model-value="cfDeriveUrl" @update:model-value="onCfDeriveUrlChange" />
          </div>
        </div>
        <p class="text-xs text-gray-400 mt-2">
          {{
            cfDeriveUrl
              ? '已开启：受限文件按 fileId 推导地址下载，安装整合包不再因缺文件中断'
              : '已关闭（默认）：受限文件不推导，安装时会列出缺失文件并提示手动下载'
          }}
        </p>
        <p class="text-xs text-gray-400 mt-1">
          推导依赖 CurseForge 未公开的 CDN 路径布局，可能与官方行为不一致；仅建议在明确需要时开启。
        </p>
      </div>
    </div>
  </div>
</template>
