<template>
  <Drawer
    :visible="visible"
    placement="right"
    :width="560"
    render-in-place
    popup-container="#app-content"
    @update:visible="visible = $event"
  >
    <template #title>
      <!-- 双提示都触发时显示分段切换器，否则只显示当前提示标题 -->
      <SegmentedButtons
        v-if="pageVisible.buy && pageVisible.star"
        :model-value="activePage"
        :options="TAB_OPTIONS"
        button-class="flex-1"
        @update:model-value="onTabSwitch"
      />
      <div v-else class="flex items-center gap-1.5">
        <ShoppingBagIcon
          v-if="activePage === 'buy'"
          class="h-4 w-4 text-primary-500"
        />
        <StarIcon v-else class="h-4 w-4 text-primary-500" />
        <span>{{ headerTitle }}</span>
      </div>
    </template>

    <!-- 双页面横向滑动容器 -->
    <div class="hint-pages-wrapper">
      <div class="hint-pages" :style="pagesTransform">
        <!-- 正版购买建议页 -->
        <div v-if="pageVisible.buy" class="hint-page">
          <div
            v-if="buyCount !== undefined"
            class="flex items-center gap-2.5 rounded-md border border-primary-100 bg-primary-50 px-3 py-2.5"
          >
            <InformationCircleIcon class="h-4 w-4 shrink-0 text-primary-500" />
            <p class="text-sm leading-relaxed text-gray-700">
              你已通过 MoLaunch 启动游戏 {{ buyCount }} 次。
            </p>
          </div>

          <div class="mt-5">
            <p class="mb-1.5 text-xs font-medium text-gray-500">为什么建议购买正版？</p>
            <div class="rounded-md border border-gray-200 bg-gray-50 px-3 py-2.5 text-sm leading-relaxed text-gray-700">
              <p>购买正版可解锁完整游戏体验与官方服务器，同时支持 Mojang 持续开发更新。</p>
              <p class="mt-2 text-xs text-gray-500">登录正版账号后，此提示将不再显示。</p>
            </div>
          </div>

          <div class="mt-5 space-y-2.5">
            <p class="mb-1.5 text-xs font-medium text-gray-500">购买后你将获得</p>
            <div
              v-for="item in buyBenefits"
              :key="item.title"
              class="flex items-start gap-2.5 rounded-md border border-gray-200 px-3 py-2.5"
            >
              <CheckIcon class="mt-0.5 h-4 w-4 shrink-0 text-primary-500" />
              <div class="min-w-0 flex-1">
                <p class="text-sm font-medium text-gray-900">{{ item.title }}</p>
                <p class="mt-0.5 text-xs leading-relaxed text-gray-500">{{ item.desc }}</p>
              </div>
            </div>
          </div>
        </div>

        <!-- 支持 MoLaunch（点 Star）页 -->
        <div v-if="pageVisible.star" class="hint-page">
          <div
            v-if="starCount !== undefined && starConfig"
            class="flex items-center gap-2.5 rounded-md border border-primary-100 bg-primary-50 px-3 py-2.5"
          >
            <InformationCircleIcon class="h-4 w-4 shrink-0 text-primary-500" />
            <p class="text-sm leading-relaxed text-gray-700">
              {{ starCountMessage }}
            </p>
          </div>

          <div class="mt-5">
            <div class="rounded-md border border-gray-200 bg-gray-50 px-3 py-2.5 text-sm leading-relaxed text-gray-700">
              <p>{{ starConfig?.subMessage }}</p>
            </div>
          </div>

          <div class="mt-5 space-y-2.5">
            <p class="mb-1.5 text-xs font-medium text-gray-500">点 Star 的意义</p>
            <div
              v-for="item in starBenefits"
              :key="item.title"
              class="flex items-start gap-2.5 rounded-md border border-gray-200 px-3 py-2.5"
            >
              <CheckIcon class="mt-0.5 h-4 w-4 shrink-0 text-primary-500" />
              <div class="min-w-0 flex-1">
                <p class="text-sm font-medium text-gray-900">{{ item.title }}</p>
                <p class="mt-0.5 text-xs leading-relaxed text-gray-500">{{ item.desc }}</p>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 底部按钮随当前页切换 -->
    <template #footer>
      <div v-if="activePage === 'buy'" class="flex justify-end gap-2">
        <Button type="ghost" size="small" @click="visible = false">暂不考虑</Button>
        <Button type="primary" size="small" @click="supportPurchase">前往购买</Button>
      </div>
      <div v-else class="flex justify-end gap-2">
        <Button type="ghost" size="small" @click="visible = false">{{ starConfig?.cancelText ?? '暂不考虑' }}</Button>
        <Button type="primary" size="small" @click="goStar">{{ starConfig?.confirmText ?? '去点 Star' }}</Button>
      </div>
    </template>
  </Drawer>
</template>

<script setup lang="ts">
import { computed, ref, watch, defineAsyncComponent } from 'vue'
import {
  CheckIcon,
  InformationCircleIcon,
  ShoppingBagIcon,
  StarIcon,
} from '@heroicons/vue/24/outline'
import { open } from '@tauri-apps/plugin-shell'
const Drawer = defineAsyncComponent(() => import('@/components/common/Drawer.vue'))
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
import SegmentedButtons, { type SegmentedOption } from '@/components/common/SegmentedButtons.vue'
import { applyConfig } from '@/utils/api/config'
import { resolveStarHintConfig, type StarHintRemoteConfig } from '@/utils/starHint'

/** 展示项（权益 / 意义） */
interface HintItem {
  title: string
  desc: string
}

/** 双提示切换页（buy 在前、star 在后） */
type HintPage = 'buy' | 'star'

/** 双提示同时触发时的分段切换选项 */
const TAB_OPTIONS: SegmentedOption<HintPage>[] = [
  { label: '正版购买建议', value: 'buy' },
  { label: '支持 MoLaunch', value: 'star' },
]

/** 购买权益展示项 */
const buyBenefits: HintItem[] = [
  { title: '官方服务器', desc: '在线游玩与跨平台进度同步，告别离线限制' },
  { title: '皮肤上传', desc: '自定义角色外观，在多人游戏中展示专属形象' },
  { title: '完整内容', desc: '含最新版本更新，持续获得 Mojang 的后续支持' },
]

/** 点 Star 意义展示项 */
const starBenefits: HintItem[] = [
  { title: '项目方向', desc: '让开发者了解 MoLaunch 有人在使用，坚定持续开发与迭代的信心' },
  { title: '社区回馈', desc: 'Star 越多越容易被更多人发现，帮助启动器触达更多玩家' },
  { title: '免费无广告', desc: '让 MoLaunch 能继续以免费、无任何广告的形式维护下去' },
]

const MC_OFFICIAL_URL = 'https://www.minecraft.net/zh-hans'

const visible = ref(false)
const activePage = ref<HintPage>('buy')
/** 已登记触发的提示页（同时触发两个时出现切换器） */
const pageVisible = ref({ buy: false, star: false })
const buyCount = ref<number | undefined>(undefined)
const starCount = ref<number | undefined>(undefined)
const starConfig = ref<StarHintRemoteConfig | null>(null)

/** 分段切换器回调：仅接受已知页名 */
function onTabSwitch(value: string | number | boolean) {
  activePage.value = value === 'star' ? 'star' : 'buy'
}

/** 标题：单页时显示当前页标题 */
const headerTitle = computed(() => {
  if (activePage.value === 'buy') return '正版购买建议'
  return starConfig.value?.title ?? '支持 MoLaunch'
})

/** 换页滑动偏移：仅双页同时登记时才按当前页偏移，单页场景不做位移避免滑出视口 */
const pagesTransform = computed(() => {
  const hasBuy = pageVisible.value.buy
  const hasStar = pageVisible.value.star
  const index = hasBuy && hasStar ? (activePage.value === 'buy' ? 0 : 1) : 0
  return { transform: `translateX(-${index * 100}%)` }
})

/** 合并远程配置后的 Star 计数字样（替换 {count} 占位） */
const starCountMessage = computed(() => {
  if (starCount.value === undefined || !starConfig.value) return ''
  return starConfig.value.message.replace('{count}', String(starCount.value))
})

/** 登记正版购买提示页并切到该页 */
function showBuy(currentCount?: number) {
  pageVisible.value.buy = true
  buyCount.value = currentCount
  activePage.value = 'buy'
  visible.value = true
}

/** 登记点 Star 提示页并切到该页（合并远程配置） */
async function showStar(currentCount?: number) {
  starConfig.value = await resolveStarHintConfig()
  pageVisible.value.star = true
  starCount.value = currentCount
  activePage.value = 'star'
  visible.value = true
}

/** 关闭后重置登记，确保下次打开只含本次触发内容 */
watch(visible, (v) => {
  if (!v) {
    pageVisible.value = { buy: false, star: false }
    buyCount.value = undefined
    starCount.value = undefined
  }
})

/** 前往购买：打开官网并永久忽略提示 */
function supportPurchase() {
  void open(MC_OFFICIAL_URL)
  void applyConfig({ hintBuy: true })
  visible.value = false
}

/** 去点 Star：打开仓库并永久忽略提示 */
function goStar() {
  if (!starConfig.value) return
  void open(starConfig.value.githubUrl)
  void applyConfig({ hintStar: true })
  visible.value = false
}

defineExpose({ showBuy, showStar })
</script>

<style scoped>
.hint-pages-wrapper {
  overflow: hidden;
}

.hint-pages {
  display: flex;
  transition: transform 0.3s ease;
}

.hint-page {
  flex: 0 0 100%;
  min-width: 0;
}
</style>
