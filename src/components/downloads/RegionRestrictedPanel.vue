<script setup lang="ts">
/**
 * 地区下载限制提示面板
 *
 * 命中官网下发的受限地区时替换下载页内容区：
 * 标题与正文取自 /download-policy.json 的 notice，策略未下发时回退静态文案；
 * 策略允许且处于开发者模式时，额外提供「临时关闭限制」入口（仅本次运行有效）。
 */
import { computed, defineAsyncComponent } from 'vue'
import {
  ExclamationTriangleIcon,
  ArrowTopRightOnSquareIcon,
  WrenchScrewdriverIcon,
} from '@heroicons/vue/24/outline'
import { openExternal } from '@/utils/openExternal'
import { useRegion } from '@/composables/useRegion'
import { showConfirmAsync } from '@/utils/modal'
import { toastError, toastInfo, toastSuccess } from '@/utils/toast'

const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))

const { notice, canBypass, setBypass } = useRegion()

/** 我的世界官方网站（中国版下载入口） */
const OFFICIAL_URL = 'https://mc.163.com/'
/** 策略未下发文案时的兜底正文 */
const FALLBACK_BODY =
  '受最近某事件影响，MoLaunch 启动器暂不向中国用户提供 MC 下载功能，恕我无能为力... qwq...'

/** 提示标题（官网下发优先） */
const title = computed(() => notice.value?.title.trim() || '当前地区暂不支持下载')
/** 提示正文（官网下发优先） */
const body = computed(() => notice.value?.text.trim() || FALLBACK_BODY)

/** 在系统浏览器打开官方网站 */
async function openOfficialSite() {
  toastInfo('正在打开我的世界官方网站…')
  try {
    await openExternal(OFFICIAL_URL)
  } catch (e) {
    console.error('[Region] 打开官方网站失败', e)
    toastError('打开失败，请手动访问 mc.163.com')
  }
}

/** 开发者模式：临时关闭下载限制（重启后按官网策略重新生效） */
async function handleBypass() {
  const agreed = await showConfirmAsync(
    '临时关闭下载限制',
    '仅对本次运行生效，重启启动器后按官网策略重新限制。\n该入口仅开发者模式可见。',
  )
  if (!agreed) return
  try {
    await setBypass(true)
    toastSuccess('已临时关闭下载限制')
  } catch (e) {
    toastError(String(e))
  }
}
</script>

<template>
  <div class="flex h-full items-center justify-center bg-gray-50 p-6">
    <div class="max-w-md text-center">
      <ExclamationTriangleIcon class="mx-auto h-12 w-12 text-amber-400" />
      <h3 class="mt-4 text-base font-semibold text-gray-900">{{ title }}</h3>
      <p class="mt-2 text-sm leading-relaxed text-gray-500">{{ body }}</p>
      <div class="mt-5 flex items-center justify-center gap-2">
        <Button type="primary" @click="openOfficialSite">
          <template #icon><ArrowTopRightOnSquareIcon class="h-4 w-4" /></template>
          前往官方网站
        </Button>
        <Button v-if="canBypass" type="outline" @click="handleBypass">
          <template #icon><WrenchScrewdriverIcon class="h-4 w-4" /></template>
          临时关闭限制
        </Button>
      </div>
      <p class="mt-4 text-xs text-gray-400">已安装的版本不受影响，仍可正常启动与游玩。</p>
    </div>
  </div>
</template>
