<script setup lang="ts">
/**
 * 鸣谢子页签：特别鸣谢 + 法律信息 + 许可与版权声明
 */
import { ref, computed } from 'vue'
import Card from '@/components/common/Card.vue'
import CollapsibleCard from '@/components/common/CollapsibleCard.vue'
import Button from '@/components/common/Button.vue'
import { resolveLogo, openLink } from '@/utils/aboutLogos'
import type { AboutData } from '@/utils/api/about'
import {
  HeartIcon,
  ArrowTopRightOnSquareIcon,
  ShieldCheckIcon,
  ScaleIcon,
  ChevronDownIcon,
} from '@heroicons/vue/24/outline'

const props = defineProps<{
  aboutData: AboutData | null
  loading: boolean
  loadError: string
}>()

const acknowledgements = computed(() => props.aboutData?.acknowledgements ?? [])
const licenses = computed(() => props.aboutData?.licenses ?? [])

// ── 特别鸣谢：作者展开/收起 ──
const expandedItems = ref<Set<string>>(new Set())

function toggleItemExpanded(name: string) {
  if (expandedItems.value.has(name)) {
    expandedItems.value.delete(name)
  } else {
    expandedItems.value.add(name)
  }
  expandedItems.value = new Set(expandedItems.value)
}

function isItemExpanded(name: string): boolean {
  return expandedItems.value.has(name)
}
</script>

<template>
  <!-- 特别鸣谢 -->
  <Card>
    <template #title>
      <div class="flex items-center gap-2">
        <HeartIcon class="h-4 w-4 text-red-400" />
        <span class="text-sm font-semibold text-gray-800">特别鸣谢</span>
      </div>
    </template>

    <!-- 加载状态 -->
    <div v-if="loading" class="py-8 text-center text-[12px] text-gray-400">加载中...</div>
    <div v-else-if="loadError" class="py-8 text-center text-[12px] text-red-500">
      加载失败：{{ loadError }}
    </div>
    <div v-else class="space-y-3">
      <div
        v-for="item in acknowledgements"
        :key="item.name"
        class="rounded-lg border border-gray-100 transition-colors hover:border-gray-200 hover:bg-gray-50"
      >
        <div class="flex items-start gap-4 p-4">
          <!-- Logo（圆形，放大到 14x14 让方形 logo 白边不突兀） -->
          <div class="flex h-14 w-14 flex-none items-center justify-center rounded-full bg-white overflow-hidden ring-1 ring-gray-100">
            <img v-if="resolveLogo(item.logo)" :src="resolveLogo(item.logo)" :alt="item.name" class="h-full w-full object-cover" />
          </div>

          <div class="min-w-0 flex-1">
            <div class="flex items-center justify-between gap-2">
              <Button type="text" size="small" class="!px-0 !py-0" @click="openLink(item.home)">
                <span class="text-[13px] font-semibold text-gray-800">{{ item.name }}</span>
                <template #icon><ArrowTopRightOnSquareIcon class="h-3 w-3 text-gray-400" /></template>
              </Button>
              <!-- 展开作者按钮 -->
              <button
                class="flex items-center gap-1 rounded px-2 py-1 text-[11px] text-gray-500 transition-colors hover:bg-gray-100 hover:text-gray-700"
                :aria-expanded="isItemExpanded(item.name)"
                @click="toggleItemExpanded(item.name)"
              >
                <span>作者</span>
                <ChevronDownIcon
                  class="h-3 w-3 transition-transform duration-200"
                  :class="isItemExpanded(item.name) ? 'rotate-180' : ''"
                />
              </button>
            </div>
            <p class="mt-1 text-[12px] leading-relaxed text-gray-500">{{ item.desc }}</p>
          </div>
        </div>

        <!-- 展开区：作者列表 -->
        <transition
          enter-active-class="transition-all duration-200 ease-out"
          leave-active-class="transition-all duration-200 ease-in"
          enter-from-class="opacity-0 max-h-0"
          enter-to-class="opacity-100 max-h-96"
          leave-from-class="opacity-100 max-h-96"
          leave-to-class="opacity-0 max-h-0"
        >
          <div v-if="isItemExpanded(item.name)" class="border-t border-gray-100 bg-gray-50/50 px-4 py-3">
            <div class="text-[11px] font-semibold uppercase tracking-wide text-gray-400">作者列表</div>
            <div v-if="item.authors.length === 0" class="mt-1.5 text-[12px] text-gray-400">
              暂未提供作者信息
            </div>
            <div v-else class="mt-2 flex flex-wrap gap-2">
              <div
                v-for="author in item.authors"
                :key="author.name"
                class="flex items-center gap-2 rounded-full bg-white px-2 py-1 ring-1 ring-gray-200"
              >
                <!-- 作者头像（圆形） -->
                <img
                  v-if="author.avatar && resolveLogo(author.avatar)"
                  :src="resolveLogo(author.avatar)"
                  :alt="author.name"
                  class="h-5 w-5 rounded-full object-cover"
                />
                <!-- 无头像时显示姓名首字占位 -->
                <span
                  v-else
                  class="flex h-5 w-5 items-center justify-center rounded-full bg-primary-100 text-[10px] font-medium text-primary-700"
                >
                  {{ author.name.charAt(0) }}
                </span>
                <span class="pr-1 text-[11px] font-medium text-gray-700">{{ author.name }}</span>
              </div>
            </div>
          </div>
        </transition>
      </div>
    </div>
  </Card>

  <!-- 法律信息（默认折叠） -->
  <CollapsibleCard title="法律信息">
    <div class="space-y-4">
      <!-- 隐私声明 -->
      <div>
        <div class="flex items-center gap-1.5 text-[13px] font-semibold text-gray-700">
          <ShieldCheckIcon class="h-4 w-4 text-gray-400" />
          隐私声明与个人信息保护政策
        </div>
        <p class="mt-1 text-[12px] leading-relaxed text-gray-500">
          MoLaunch 不会收集用户的任何个人信息。所有账号凭据仅存储在本地，不会上传至任何服务器。
          联机功能中的 FRP 隧道连接由第三方 FRP 服务商处理，MoLaunch 仅负责创建隧道配置，不中转或存储任何游戏数据。
        </p>
      </div>

      <!-- 版权声明 -->
      <div>
        <div class="flex items-center gap-1.5 text-[13px] font-semibold text-gray-700">
          <ScaleIcon class="h-4 w-4 text-gray-400" />
          版权声明
        </div>
        <p class="mt-1 text-[12px] leading-relaxed text-gray-500">
          Copyright &copy; 2026 MoTeam. All Rights Reserved.<br>
          本软件为开源软件，遵循 MIT 许可协议发布，源代码托管于 GitHub。
        </p>
      </div>

      <!-- Mojang 免责声明 -->
      <div>
        <div class="text-[13px] font-semibold text-gray-700">Mojang 免责声明</div>
        <p class="mt-1 text-[12px] leading-relaxed text-gray-500">
          MoLaunch 不是 Minecraft 官方产品，未经 MOJANG 或 MICROSOFT 批准，也不与 MOJANG 或 MICROSOFT 关联。
          Minecraft 是 Mojang Synergies AB 的商标。
        </p>
      </div>

      <!-- 特别说明 -->
      <div>
        <div class="text-[13px] font-semibold text-gray-700">特别说明</div>
        <div class="mt-1 space-y-2 text-[12px] leading-relaxed text-gray-500">
          <p>
            <span class="font-medium text-gray-600">关于 PCL2：</span>
            本启动器基于 Tauri v2 框架开发，前端采用 Vue3，后端采用 Rust。
            开发过程中参考了 Plain Craft Launcher 2 (PCL2) 的部分设计逻辑与交互理念，
            后端逻辑均为 Rust 原创实现，不存在直接复制 PCL2 源代码的情况，但在功能逻辑与交互设计上可能存在相似之处，在此特别声明并致谢。
            PCL2 采用<a href="#" class="text-primary-500 hover:text-primary-600" @click.prevent="openLink('https://shimo.im/docs/rGrd8pY8xWkt6ryW')"> 《PCL 分发有限许可》</a>，详情参阅其许可文档。
          </p>
          <p>
            <span class="font-medium text-gray-600">关于 Arco Design：</span>
            本启动器前端早期使用 Tailwind CSS 自行编写组件，但视觉效果与交互细节不尽人意。
            后续引入 Arco Design Vue 开源组件库，提取其组件源码并复刻改写为 Vue SFC + Tailwind 形式，
            以获得更一致的视觉体验与交互质量。所有涉及复刻的代码文件顶部均已添加 Arco Design MIT 许可证要求的版权声明注释，可自行查看源代码确认。
            在此特别声明并致谢。
          </p>
          <p>
            <span class="font-medium text-gray-600">关于 Element Plus Icons：</span>
            本启动器前端默认使用 Heroicons Vue 作为主图标库，但其图标集相对精简，部分场景（如日志级别提示、状态徽标、对话框图标等）缺少合适样式或表达不够直观。
            为保证视觉一致性与信息传达准确性，开发者从 Element Plus Icons 开源图标库中检索所需图标，仅提取其 SVG path 数据并集中写入项目内的 <code class="rounded bg-gray-100 px-1 py-0.5 text-[11px] text-gray-700">src/utils/element-icons.ts</code> 文件复用，未引入 Element Plus 运行时依赖。
            该文件顶部已添加 Element Plus Icons MIT 许可证要求的版权声明与完整许可文本，可自行查看源代码确认。在此特别声明并致谢。
          </p>
        </div>
      </div>

      <!-- 跳转按钮 -->
      <div class="flex gap-3 pt-1">
        <Button type="primary" size="small" @click="openLink('https://github.com/MoTeam-cn/MoLaunch')">
          <template #icon><ArrowTopRightOnSquareIcon class="h-3.5 w-3.5" /></template>
          查看源代码
        </Button>
        <Button type="outline" size="small" @click="openLink('https://github.com/MoTeam-cn/MoLaunch/blob/main/LICENSE')">
          <template #icon><ScaleIcon class="h-3.5 w-3.5" /></template>
          查看许可协议
        </Button>
      </div>
    </div>
  </CollapsibleCard>

  <!-- 许可与版权声明（默认折叠） -->
  <CollapsibleCard title="许可与版权声明">
    <!-- 加载状态 -->
    <div v-if="loading" class="py-6 text-center text-[12px] text-gray-400">加载中...</div>
    <div v-else-if="loadError" class="py-6 text-center text-[12px] text-red-500">
      加载失败：{{ loadError }}
    </div>
    <div v-else class="space-y-1">
      <div
        v-for="lib in licenses"
        :key="lib.name"
        class="border-b border-gray-50 py-2.5 last:border-0"
      >
        <div class="flex items-start justify-between gap-4">
          <div class="min-w-0 flex-1">
            <div class="text-[12px] font-semibold text-gray-700">{{ lib.name }}</div>
            <div class="mt-0.5 text-[11px] leading-relaxed text-gray-500">
              {{ lib.copyright }}<br>
              Licensed under {{ lib.license }}
            </div>
          </div>
        </div>
        <div class="mt-1.5 flex gap-2">
          <Button type="text" size="small" @click="openLink(lib.sourceUrl)">查看来源网站</Button>
          <Button type="text" size="small" @click="openLink(lib.licenseUrl)">查看许可文档</Button>
        </div>
      </div>
    </div>
  </CollapsibleCard>
</template>
