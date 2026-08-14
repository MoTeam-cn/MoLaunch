<script setup lang="ts">
/**
 * 创作指令 - 成就生成器
 *
 * Canvas 2D 绘制原版风格「获得成就」弹窗（320×65、2 倍率输出），
 * 支持物品图标 / 标题内容颜色 / 字体，导出 PNG 到指定目录；
 * 图片右下角固定叠加白色 MoLaunch 版权水印。
 */
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { ArrowDownTrayIcon, TrophyIcon } from '@heroicons/vue/24/outline'
import Input from '@/components/common/Input.vue'
import Button from '@/components/common/Button.vue'
import Select from '@/components/common/Select.vue'
import { toastError, toastSuccess } from '@/utils/toast'
import { pickSavePath } from '@/utils/fileDialog'
import { getAtlasLayout, getAtlasPngUrl, loadVersionItems } from '@/utils/recipe-generator/resources'
import type { AssetItem, AtlasLayout } from '@/utils/recipe-generator/resources'
import { systemManager, SYSTEM_ACTIONS } from '@/utils/api/system-manager'
import ColorSelect from '../command/ColorSelect.vue'
import { COLOR_OPTIONS } from '../command/generator'
import RecipeItemIcon from './recipe-generator/RecipeItemIcon.vue'
import { ACHIEVEMENT_SCALE, ACHIEVEMENT_SIZE, drawAchievement } from './achievement/draw'

const items = ref<AssetItem[]>([])
const keyword = ref('')
const selected = ref<AssetItem | null>(null)
const atlas = ref<AtlasLayout | null>(null)
const atlasImg = ref<HTMLImageElement | null>(null)
const canvasRef = ref<HTMLCanvasElement | null>(null)

const title = ref('获得成就！')
const titleColor = ref('gold')
const content = ref('达成一个成就')
const contentColor = ref('white')
const fontFamily = ref("Consolas, 'Courier New', monospace")
const exporting = ref(false)

const fontOptions = [
  { label: '等宽（像素风）', value: "Consolas, 'Courier New', monospace" },
  { label: '微软雅黑', value: "'Microsoft YaHei', 'PingFang SC', sans-serif" },
  { label: '宋体', value: "SimSun, 'Songti SC', serif" },
  { label: 'Arial', value: 'Arial, sans-serif' },
]

const colorMap = new Map(COLOR_OPTIONS.map((c) => [c.value, c.color]))
function colorHex(v: string) {
  return colorMap.get(v) ?? '#ffffff'
}

const filteredItems = computed(() => {
  const q = keyword.value.trim().toLowerCase()
  if (!q) return items.value.slice(0, 200)
  return items.value
    .filter(
      (i) =>
        i.id.toLowerCase().includes(q) ||
        i.name.toLowerCase().includes(q) ||
        i.zh.toLowerCase().includes(q),
    )
    .slice(0, 200)
})

function pickItem(item: AssetItem) {
  selected.value = item
  keyword.value = ''
}

function draw() {
  const canvas = canvasRef.value
  const img = atlasImg.value
  if (!canvas || !img) return
  canvas.width = ACHIEVEMENT_SIZE.width * ACHIEVEMENT_SCALE
  canvas.height = ACHIEVEMENT_SIZE.height * ACHIEVEMENT_SCALE
  const ctx = canvas.getContext('2d')
  if (!ctx) return
  const texture = selected.value?.texture
  const region = texture && atlas.value ? atlas.value.layout[texture] : undefined
  drawAchievement(ctx, {
    title: title.value,
    titleColor: colorHex(titleColor.value),
    content: content.value,
    contentColor: colorHex(contentColor.value),
    fontFamily: fontFamily.value,
    icon: region ? { img, region } : null,
  })
}

watch([title, titleColor, content, contentColor, fontFamily, selected], () => nextTick(draw))

onMounted(async () => {
  const [itemList, layout] = await Promise.all([loadVersionItems('26.2'), getAtlasLayout()])
  items.value = itemList
  atlas.value = layout
  const img = new Image()
  img.src = getAtlasPngUrl()
  await img.decode()
  atlasImg.value = img
  selected.value = itemList.find((i) => i.id === 'diamond') ?? null
  await nextTick()
  draw()
})

async function exportPng() {
  const canvas = canvasRef.value
  if (!canvas) return
  exporting.value = true
  try {
    const path = await pickSavePath({
      title: '导出成就图片',
      defaultPath: 'achievement.png',
      filters: [{ name: 'PNG 图片', extensions: ['png'] }],
    })
    if (!path) return
    const base64 = canvas.toDataURL('image/png').replace(/^data:image\/png;base64,/, '')
    await systemManager(SYSTEM_ACTIONS.WRITE_BINARY_FILE, { path, base64 })
    toastSuccess('已导出到指定目录')
  } catch (err) {
    toastError(`导出失败：${err}`)
  } finally {
    exporting.value = false
  }
}
</script>

<template>
  <section class="rounded-lg border border-gray-300 bg-white">
    <div class="flex items-center gap-2 px-5 pt-5 pb-3">
      <TrophyIcon class="h-5 w-5 text-gray-700" />
      <h3 class="text-sm font-semibold text-gray-900">成就生成器</h3>
    </div>

    <div class="px-5 pb-5 space-y-4">
      <!-- 成就图标 -->
      <div>
        <div class="text-xs font-medium text-gray-500 mb-2">成就图标</div>
        <Input v-model="keyword" placeholder="搜索物品（ID / 名称 / 中文）…" size="small" clearable />
        <div
          v-if="atlas && filteredItems.length && !selected"
          class="mt-2 max-h-48 overflow-y-auto rounded border border-gray-200"
        >
          <button
            v-for="item in filteredItems"
            :key="item.id"
            type="button"
            class="flex w-full items-center gap-2 px-3 py-1.5 text-left text-xs hover:bg-primary-50"
            @click="pickItem(item)"
          >
            <RecipeItemIcon :texture="item.texture" :atlas-url="getAtlasPngUrl()" :atlas="atlas" :size="20" />
            <span class="truncate font-mono text-gray-700">{{ item.id }}</span>
            <span class="truncate text-gray-400">{{ item.zh || item.name }}</span>
          </button>
        </div>
        <div
          v-if="selected && atlas"
          class="mt-2 flex items-center justify-between rounded border border-primary-200 bg-primary-50 px-3 py-2"
        >
          <span class="flex items-center gap-2 text-xs font-medium text-primary-700">
            <RecipeItemIcon :texture="selected.texture" :atlas-url="getAtlasPngUrl()" :atlas="atlas" :size="20" />
            {{ selected.zh || selected.name }}（{{ selected.id }}）
          </span>
          <Button type="text" size="mini" @click="selected = null">重新选择</Button>
        </div>
      </div>

      <!-- 标题 / 内容 / 颜色 / 字体 -->
      <div class="grid grid-cols-2 gap-4">
        <div>
          <div class="text-xs font-medium text-gray-500 mb-2">成就标题</div>
          <Input v-model="title" placeholder="如：获得成就！" size="small" />
        </div>
        <div>
          <div class="text-xs font-medium text-gray-500 mb-2">标题颜色</div>
          <ColorSelect v-model="titleColor" />
        </div>
        <div>
          <div class="text-xs font-medium text-gray-500 mb-2">成就内容</div>
          <Input v-model="content" placeholder="如：达成一个成就" size="small" />
        </div>
        <div>
          <div class="text-xs font-medium text-gray-500 mb-2">内容颜色</div>
          <ColorSelect v-model="contentColor" />
        </div>
        <div>
          <div class="text-xs font-medium text-gray-500 mb-2">字体</div>
          <Select v-model="fontFamily" :options="fontOptions" />
        </div>
      </div>

      <!-- 预览与导出 -->
      <div class="flex flex-col items-center gap-4 rounded-lg bg-gray-100 p-4">
        <canvas
          ref="canvasRef"
          class="rounded shadow-sm"
          :style="{ width: ACHIEVEMENT_SIZE.width + 'px', height: ACHIEVEMENT_SIZE.height + 'px' }"
        />
        <div class="flex items-center gap-2">
          <Button size="small" :disabled="exporting" @click="exportPng">
            <template #icon><ArrowDownTrayIcon class="h-4 w-4" /></template>
            {{ exporting ? '导出中…' : '导出 PNG 到指定目录' }}
          </Button>
          <span class="text-xs text-gray-400">图片右下角含 MoLaunch 版权水印</span>
        </div>
      </div>
    </div>
  </section>
</template>
