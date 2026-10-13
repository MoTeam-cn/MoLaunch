<script setup lang="ts">
/**
 * 公告弹窗（启动时展示未读公告）
 *
 * 触发与已读去重见 utils/announcement.ts。正文为 Markdown，经 renderMarkdown
 * 消毒后渲染。dismissible=false 的公告不显示右上角关闭，只能点「我知道了」
 * 临时收起（下次启动仍会提示），避免重要通知被一次性关掉。
 */
import { computed, defineAsyncComponent } from 'vue'
import { ArrowTopRightOnSquareIcon, MegaphoneIcon } from '@heroicons/vue/24/outline'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Card = defineAsyncComponent(() => import('@/components/common/Card.vue'))
const Drawer = defineAsyncComponent(() => import('@/components/common/Drawer.vue'))
const Tag = defineAsyncComponent(() => import('@/components/common/Tag.vue'))
import { handleMarkdownLinkClick, renderMarkdown } from '@/utils/markdown'
import { isSafeExternalUrl, openExternal } from '@/utils/openExternal'
import {
  announcementVisible,
  closeAnnouncements,
  pendingAnnouncements,
} from '@/utils/announcement'
import type { AnnouncementLevel } from '@/types/announcement'

const visible = computed(() => announcementVisible.value)
const list = computed(() => pendingAnnouncements.value)

/** 全部公告都可关闭时才允许点遮罩 / 按 Esc 关闭 */
const closable = computed(() => list.value.every((a) => a.dismissible))

/** 级别 → Tag 预设色与文案（键需与 api-server 的 level 取值一致） */
const LEVEL: Record<AnnouncementLevel, { label: string; color: string }> = {
  info: { label: '通知', color: 'primary' },
  warning: { label: '注意', color: 'orange' },
  important: { label: '重要', color: 'red' },
}

function levelOf(level: string) {
  return LEVEL[level as AnnouncementLevel] ?? LEVEL.info
}

/** 外链来自服务端，先过安全校验再打开 */
function onOpenLink(url: string) {
  if (isSafeExternalUrl(url)) void openExternal(url)
}

function onVisibleChange(v: boolean) {
  if (!v) closeAnnouncements()
}
</script>

<template>
  <Drawer
    :visible="visible"
    placement="right"
    :width="520"
    render-in-place
    popup-container="#app-content"
    :closable="closable"
    :mask-closable="closable"
    :esc-to-close="closable"
    @update:visible="onVisibleChange"
  >
    <template #title>
      <div class="flex items-center gap-2">
        <MegaphoneIcon class="h-5 w-5 text-primary-500" />
        <span>公告</span>
      </div>
    </template>

    <div class="flex h-full min-h-0 flex-col gap-3 overflow-y-auto">
      <Card v-for="item in list" :key="item.id" size="small">
        <template #title>
          <div class="flex items-center gap-2">
            <Tag :color="levelOf(item.level).color" size="small">
              {{ levelOf(item.level).label }}
            </Tag>
            <span class="min-w-0 flex-1 truncate">{{ item.title }}</span>
          </div>
        </template>

        <!-- eslint-disable vue/no-v-html -- renderMarkdown 已用 DOMPurify 消毒 -->
        <div
          class="markdown-body text-xs leading-relaxed text-gray-700"
          @click="handleMarkdownLinkClick"
          v-html="renderMarkdown(item.content)"
        />
        <!-- eslint-enable vue/no-v-html -->

        <template v-if="item.link_url" #actions>
          <Button type="ghost" size="small" @click="onOpenLink(item.link_url)">
            <template #icon>
              <ArrowTopRightOnSquareIcon class="h-3.5 w-3.5" />
            </template>
            查看详情
          </Button>
        </template>
      </Card>
    </div>

    <template #footer>
      <div class="flex justify-end">
        <Button type="primary" size="small" @click="closeAnnouncements">我知道了</Button>
      </div>
    </template>
  </Drawer>
</template>
