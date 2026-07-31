<script setup lang="ts">
/**
 * 游戏内调色板工具
 *
 * 功能：
 * - RGB / HEX / HSL 三种颜色格式互转
 * - 生成 Minecraft 可用颜色代码（§ 前缀的格式化代码）
 * - 预设 Minecraft 染料色板（16 种标准染料色）
 *
 * 纯前端计算，无后端调用
 */
import { ref, computed, watch } from 'vue'
import { SwatchIcon, ClipboardIcon } from '@heroicons/vue/24/outline'
import Input from '@/components/common/Input.vue'
import Button from '@/components/common/Button.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import { toastSuccess, toastError } from '@/utils/toast'
import { copyToClipboard } from '@/utils/seedmap/format'

// 当前 RGB 值（0-255）
const r = ref(22)
const g = ref(93)
const b = ref(255)

// HEX 输入
const hexInput = ref('')

// Minecraft 16 色染料预设（名称 + RGB + § 代码）
const mcDyes = [
  { name: '白色', code: '§f', rgb: [255, 255, 255] },
  { name: '橙色', code: '§6', rgb: [219, 125, 62] },
  { name: '品红色', code: '§d', rgb: [179, 80, 188] },
  { name: '淡蓝色', code: '§9', rgb: [107, 138, 201] },
  { name: '黄色', code: '§e', rgb: [177, 166, 39] },
  { name: '黄绿色', code: '§a', rgb: [65, 174, 56] },
  { name: '粉红色', code: '§d', rgb: [208, 132, 153] },
  { name: '灰色', code: '§7', rgb: [64, 64, 64] },
  { name: '淡灰色', code: '§7', rgb: [154, 161, 161] },
  { name: '青色', code: '§3', rgb: [46, 110, 137] },
  { name: '紫色', code: '§5', rgb: [126, 52, 191] },
  { name: '蓝色', code: '§1', rgb: [46, 56, 141] },
  { name: '棕色', code: '§6', rgb: [79, 50, 31] },
  { name: '绿色', code: '§2', rgb: [53, 70, 27] },
  { name: '红色', code: '§c', rgb: [150, 52, 48] },
  { name: '黑色', code: '§0', rgb: [25, 22, 22] },
]

function rgbToHex(r: number, g: number, b: number): string {
  return '#' + [r, g, b].map((v) => v.toString(16).padStart(2, '0')).join('').toUpperCase()
}

function rgbToHsl(r: number, g: number, b: number): [number, number, number] {
  r /= 255; g /= 255; b /= 255
  const max = Math.max(r, g, b), min = Math.min(r, g, b)
  let h = 0, s = 0
  const l = (max + min) / 2
  if (max !== min) {
    const d = max - min
    s = l > 0.5 ? d / (2 - max - min) : d / (max + min)
    switch (max) {
      case r: h = (g - b) / d + (g < b ? 6 : 0); break
      case g: h = (b - r) / d + 2; break
      case b: h = (r - g) / d + 4; break
    }
    h /= 6
  }
  return [Math.round(h * 360), Math.round(s * 100), Math.round(l * 100)]
}

const hexValue = computed(() => rgbToHex(r.value, g.value, b.value))
const hslValue = computed(() => {
  const [h, s, l] = rgbToHsl(r.value, g.value, b.value)
  return `HSL(${h}, ${s}%, ${l}%)`
})

// 同步 HEX 输入 → RGB
watch(hexInput, (val) => {
  const cleaned = val.trim().replace(/^#/, '')
  if (/^[0-9a-fA-F]{6}$/.test(cleaned)) {
    r.value = parseInt(cleaned.substring(0, 2), 16)
    g.value = parseInt(cleaned.substring(2, 4), 16)
    b.value = parseInt(cleaned.substring(4, 6), 16)
  }
})

// RGB 变化时同步 hexInput（避免循环：只在 hexInput 不匹配时更新）
watch([r, g, b], () => {
  const expected = rgbToHex(r.value, g.value, b.value).replace('#', '')
  if (hexInput.value.replace(/^#/, '').toUpperCase() !== expected) {
    hexInput.value = expected
  }
})

function selectDye(rgb: number[]) {
  r.value = rgb[0]
  g.value = rgb[1]
  b.value = rgb[2]
}

async function copyHex() {
  const ok = await copyToClipboard(hexValue.value)
  if (ok) toastSuccess('已复制 HEX: ' + hexValue.value)
  else toastError('复制失败')
}

async function copyCode(code: string) {
  const ok = await copyToClipboard(code)
  if (ok) toastSuccess('已复制: ' + code)
  else toastError('复制失败')
}

const colorPreviewStyle = computed(() => `background-color: ${hexValue.value}`)

/** Minecraft 16 色格式化代码（含色值 + 是否深色背景需浅色文字） */
const formatCodes = [
  { code: '§0', label: '黑', color: '#000', dark: true },
  { code: '§1', label: '深蓝', color: '#0000AA', dark: true },
  { code: '§2', label: '深绿', color: '#00AA00', dark: true },
  { code: '§3', label: '青', color: '#00AAAA', dark: true },
  { code: '§4', label: '深红', color: '#AA0000', dark: true },
  { code: '§5', label: '紫', color: '#AA00AA', dark: true },
  { code: '§6', label: '金', color: '#FFAA00', dark: true },
  { code: '§7', label: '灰', color: '#AAAAAA', dark: false },
  { code: '§8', label: '深灰', color: '#555555', dark: true },
  { code: '§9', label: '蓝', color: '#5555FF', dark: true },
  { code: '§a', label: '绿', color: '#55FF55', dark: false },
  { code: '§b', label: '青绿', color: '#55FFFF', dark: false },
  { code: '§c', label: '红', color: '#FF5555', dark: false },
  { code: '§d', label: '粉', color: '#FF55FF', dark: false },
  { code: '§e', label: '黄', color: '#FFFF55', dark: false },
  { code: '§f', label: '白', color: '#FFFFFF', dark: false },
]
</script>

<template>
  <section class="rounded-lg border border-gray-300 bg-white">
    <div class="flex items-center gap-2 px-5 pt-5 pb-3">
      <SwatchIcon class="h-5 w-5 text-gray-700" />
      <h3 class="text-sm font-semibold text-gray-900">游戏内调色板</h3>
    </div>

    <div class="px-5 pb-5 space-y-4">
      <!-- 颜色预览 + HEX 输入 -->
      <div class="flex items-center gap-3">
        <div
          class="h-12 w-12 flex-none rounded-lg border border-gray-200 shadow-sm"
          :style="colorPreviewStyle"
        ></div>
        <div class="flex-1 space-y-1.5">
          <div class="flex items-center gap-2">
            <span class="w-10 text-xs text-gray-500">HEX</span>
            <Input v-model="hexInput" placeholder="#165DFF" size="small" width="120px" />
            <Tooltip text="复制 HEX" position="top">
              <Button type="ghost" size="small" @click="copyHex">
                <template #icon>
                  <ClipboardIcon class="h-3.5 w-3.5" />
                </template>
              </Button>
            </Tooltip>
          </div>
          <div class="text-xs text-gray-500">{{ hslValue }}</div>
        </div>
      </div>

      <!-- RGB 滑块（range 类型无自研组件，保留原生 input range；数字输入用自研 Input） -->
      <div class="space-y-2">
        <div class="flex items-center gap-2">
          <span class="w-4 text-xs font-medium text-gray-600">R</span>
          <input v-model.number="r" type="range" min="0" max="255" class="flex-1 h-1.5 cursor-pointer" />
          <Input v-model.number="r" type="number" size="small" width="64px" />
        </div>
        <div class="flex items-center gap-2">
          <span class="w-4 text-xs font-medium text-gray-600">G</span>
          <input v-model.number="g" type="range" min="0" max="255" class="flex-1 h-1.5 cursor-pointer" />
          <Input v-model.number="g" type="number" size="small" width="64px" />
        </div>
        <div class="flex items-center gap-2">
          <span class="w-4 text-xs font-medium text-gray-600">B</span>
          <input v-model.number="b" type="range" min="0" max="255" class="flex-1 h-1.5 cursor-pointer" />
          <Input v-model.number="b" type="number" size="small" width="64px" />
        </div>
      </div>

      <!-- Minecraft 染料预设（与 ColorPicker.vue 预设色块一致的 div role=button 模式） -->
      <div class="border-t border-gray-100 pt-4">
        <div class="text-xs font-medium text-gray-500 mb-2">Minecraft 染料色</div>
        <div class="grid grid-cols-8 gap-1.5">
          <Tooltip
            v-for="dye in mcDyes"
            :key="dye.name"
            :text="`${dye.name} ${dye.code} (${dye.rgb.join(', ')})`"
            position="top"
          >
            <div
              role="button"
              tabindex="0"
              class="h-8 w-full rounded border border-gray-200 transition-transform hover:scale-110 cursor-pointer"
              :style="{ backgroundColor: `rgb(${dye.rgb.join(',')})` }"
              @click="selectDye(dye.rgb)"
              @keydown.enter="selectDye(dye.rgb)"
            ></div>
          </Tooltip>
        </div>
      </div>

      <!-- Minecraft 格式化代码 -->
      <div class="border-t border-gray-100 pt-4">
        <div class="text-xs font-medium text-gray-500 mb-2">Minecraft 格式化代码</div>
        <div class="flex flex-wrap gap-1.5">
          <Button
            v-for="code in formatCodes"
            :key="code.code"
            type="ghost"
            size="small"
            class="font-mono"
            :style="{ backgroundColor: code.color, color: code.dark ? '#fff' : '#000' }"
            @click="copyCode(code.code)"
          >
            {{ code.code }}
          </Button>
        </div>
      </div>
    </div>
  </section>
</template>
