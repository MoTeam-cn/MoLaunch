<script setup lang="ts">
/**
 * 更新日志弹窗（启动时展示）
 *
 * 对齐 PCL2 的做法：应用升级到新版本后，启动时自动展示一次本次更新日志。
 * 触发与去重逻辑见 utils/updateLog.ts；内容为 vite 构建时从 CHANGELOG.md
 * 提取的当前版本段落（虚拟模块 virtual:update-log），时间线渲染复用 ReleaseTimeline。
 */
import { computed } from 'vue'
import { ArrowTopRightOnSquareIcon, SparklesIcon } from '@heroicons/vue/24/outline'
import Button from '@/components/common/Button.vue'
import Drawer from '@/components/common/Drawer.vue'
import ReleaseTimeline from '@/components/about/ReleaseTimeline.vue'
import {
  UPDATE_LOG_GITHUB_URL,
  closeUpdateLog,
  getChangelogContent,
  getChangelogVersion,
  updateLogVisible,
} from '@/utils/updateLog'
import { openLink } from '@/utils/aboutLogos'

const visible = computed(() => updateLogVisible.value)
const version = getChangelogVersion()

function onVisibleChange(v: boolean) {
  if (!v) closeUpdateLog()
}

/** 打开 GitHub Releases 查看完整更新日志 */
function onOpenFullLog() {
  openLink(UPDATE_LOG_GITHUB_URL)
}
</script>

<template>
  <Drawer
    :visible="visible"
    placement="right"
    :width="560"
    render-in-place
    popup-container="#app-content"
    closable
    mask-closable
    esc-to-close
    @update:visible="onVisibleChange"
  >
    <template #title>
      <div class="flex items-center gap-2">
        <SparklesIcon class="h-5 w-5 text-primary-500" />
        <span>MoLaunch 已更新</span>
        <span class="font-semibold text-primary-600">v{{ version }}</span>
      </div>
    </template>

    <div class="flex h-full min-h-0 flex-col">
      <p class="py-1 text-xs text-gray-500">感谢您更新到新版本，以下是本次版本更新内容。</p>
      <div class="min-h-0 flex-1 overflow-y-auto rounded-md bg-gray-50 p-3">
        <ReleaseTimeline :notes="getChangelogContent()" />
      </div>
    </div>

    <template #footer>
      <div class="flex justify-end gap-2">
        <Button type="ghost" size="small" @click="onOpenFullLog">
          <template #icon>
            <ArrowTopRightOnSquareIcon class="h-3.5 w-3.5" />
          </template>
          完整更新日志
        </Button>
        <Button type="primary" size="small" @click="closeUpdateLog">知道了</Button>
      </div>
    </template>
  </Drawer>
</template>