<script setup lang="ts">
/**
 * 许可协议子页签：展示项目许可协议全文备份
 *
 * 数据源：项目根目录 LICENSE（build.rs 每次构建自动同步到 resources/LICENSE.txt，
 * resources.rs 以 include_str! 编译期嵌入二进制），经 get_project_license IPC 返回。
 *
 * 排版：不引入 markdown，正文用极简渲染器输出——先 HTML 转义防注入，再对短引号内容
 * （产品名 / 法律术语，≤20 字符）加粗 + 字距，段落按空行拆分、段内换行保留，提升可读性。
 */
import { ref, computed, onMounted, defineAsyncComponent } from 'vue'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
import { getProjectLicense } from '@/utils/api/about'
import { openLink } from '@/utils/aboutLogos'
import {
  ScaleIcon,
  DocumentTextIcon,
  ArrowTopRightOnSquareIcon,
} from '@heroicons/vue/24/outline'

const licenseText = ref('')
const loading = ref(true)
const loadError = ref('')

/** HTML 转义，避免正文中的 < > & 等字符被当作标签渲染 */
function escapeHtml(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;')
}

/** 短引号内容（≤20 字符，多为产品名 / 法律术语）加粗，长句保留原文 */
function boldQuotes(text: string): string {
  return text.replace(/“([^”]{1,20})”/g, '<strong>“$1”</strong>')
}

const renderedLicense = computed(() => {
  const raw = licenseText.value
  if (!raw) return ''
  return raw
    .split(/\n{2,}/)
    .map((para) => {
      const line = escapeHtml(para).replace(/\n/g, '<br />')
      return `<p>${boldQuotes(line)}</p>`
    })
    .join('')
})

onMounted(async () => {
  try {
    licenseText.value = await getProjectLicense()
  } catch (err) {
    loadError.value = err instanceof Error ? err.message : String(err)
  } finally {
    loading.value = false
  }
})
</script>

<template>
  <div class="rounded-lg border border-gray-200 bg-white shadow-sm">
    <!-- 标题栏：不吸顶常驻，随外层容器统一滚动 -->
    <div class="flex items-center justify-between gap-3 border-b border-gray-200 px-4 py-3">
      <div class="flex min-w-0 items-center gap-2">
        <ScaleIcon class="h-4 w-4 shrink-0 text-primary-500" />
        <span class="truncate text-sm font-semibold text-gray-800">MoLaunch 分发有限许可证</span>
      </div>
      <Button
        type="outline"
        size="small"
        class="shrink-0"
        @click="openLink('https://github.com/MoTeam-cn/MoLaunch/blob/main/LICENSE')"
      >
        <template #icon><ArrowTopRightOnSquareIcon class="h-3.5 w-3.5" /></template>
        查看 GitHub 原文
      </Button>
    </div>

    <!-- 加载中 -->
    <div v-if="loading" class="flex items-center justify-center px-6 py-14 text-[12px] text-gray-400">
      正在加载许可协议...
    </div>

    <!-- 加载失败 -->
    <div
      v-else-if="loadError"
      class="flex flex-col items-center justify-center px-6 py-14 text-gray-400"
    >
      <DocumentTextIcon class="mb-2 h-8 w-8 text-gray-300" />
      <span class="text-[12px]">许可协议加载失败：{{ loadError }}</span>
    </div>

    <!-- 协议正文 -->
    <div v-else class="license-markdown px-4 py-3 text-[12px] leading-relaxed text-gray-600">
      <!-- eslint-disable-next-line vue/no-v-html -- 正文已先 HTML 转义再做引号加粗，无注入风险 -->
      <div v-html="renderedLicense" />
    </div>

    <!-- 底部说明 -->
    <div class="rounded-b-lg border-t border-gray-100 bg-gray-50 px-4 py-2.5">
      <p class="text-[11px] text-gray-400">
        本协议由构建流程自动嵌入二进制；本许可证只携带此版本构建时的许可证，不排除后续版本更迭许可证更新的情况，具体以仓库许可证版本为主。
      </p>
    </div>
  </div>
</template>

<style scoped>
/* 引号加粗内容：加粗 + 字距，让「产品名 / 术语」与正文区分开（v-html 内容需用 :deep 命中） */
:deep(.license-markdown strong) {
  font-weight: 700;
  letter-spacing: 0.03em;
  color: #374151;
}

/* 段落间距：空行分段拆出的 <p> 留白，避免正文挤成一片 */
:deep(.license-markdown p) {
  margin-bottom: 0.75rem;
}

:deep(.license-markdown p:last-child) {
  margin-bottom: 0;
}
</style>
