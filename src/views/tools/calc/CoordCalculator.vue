<script setup lang="ts">
/**
 * 坐标距离计算工具
 *
 * 功能：
 * - 输入两组 XYZ 坐标，计算欧氏距离、曼哈顿距离、切比雪夫距离
 * - 地狱门连通性计算：主世界↔下界坐标换算（1:8 比例）
 *
 * 纯前端计算，无后端调用
 */
import { ref, computed, defineAsyncComponent } from 'vue'
import { MapPinIcon, ArrowsRightLeftIcon } from '@heroicons/vue/24/outline'
const Input = defineAsyncComponent(() => import('@/components/common/Input.vue'))
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Tooltip = defineAsyncComponent(() => import('@/components/common/Tooltip.vue'))

interface Coord {
  x: string
  y: string
  z: string
}

const pointA = ref<Coord>({ x: '', y: '', z: '' })
const pointB = ref<Coord>({ x: '', y: '', z: '' })

// 坐标方向换算模式：'toNether'（主世界→下界）/ 'toOverworld'（下界→主世界）
const convertMode = ref<'toNether' | 'toOverworld'>('toNether')
const convertInput = ref<string>('')

function parseNum(s: string): number | null {
  if (s.trim() === '') return null
  const n = Number(s)
  return Number.isNaN(n) ? null : n
}

const distances = computed(() => {
  const ax = parseNum(pointA.value.x)
  const ay = parseNum(pointA.value.y)
  const az = parseNum(pointA.value.z)
  const bx = parseNum(pointB.value.x)
  const by = parseNum(pointB.value.y)
  const bz = parseNum(pointB.value.z)
  if (ax === null || ay === null || az === null || bx === null || by === null || bz === null) {
    return null
  }
  const dx = Math.abs(ax - bx)
  const dy = Math.abs(ay - by)
  const dz = Math.abs(az - bz)
  return {
    euclidean: Math.sqrt(dx * dx + dy * dy + dz * dz),
    manhattan: dx + dy + dz,
    chebyshev: Math.max(dx, dy, dz),
    dx: ax - bx,
    dy: ay - by,
    dz: az - bz,
  }
})

const convertResult = computed(() => {
  const parts = convertInput.value.trim().split(/[\s,]+/).filter(Boolean)
  if (parts.length === 0) return null
  const nums = parts.map(parseNum)
  if (nums.some((n) => n === null)) return null
  const factor = convertMode.value === 'toNether' ? 1 / 8 : 8
  return nums.map((n) => (n! * factor).toFixed(2)).join(', ')
})

function swapPoints() {
  const tmp = { ...pointA.value }
  pointA.value = { ...pointB.value }
  pointB.value = tmp
}

function fmt(n: number): string {
  return n.toFixed(2)
}
</script>

<template>
  <section class="rounded-lg border border-gray-300 bg-white">
    <div class="flex items-center gap-2 px-5 pt-5 pb-3">
      <MapPinIcon class="h-5 w-5 text-gray-700" />
      <h3 class="text-sm font-semibold text-gray-900">坐标距离计算</h3>
    </div>

    <div class="px-5 pb-5 space-y-4">
      <!-- 坐标输入区 -->
      <div class="grid grid-cols-2 gap-4">
        <div class="space-y-2">
          <div class="text-xs font-medium text-gray-500">坐标 A</div>
          <div class="grid grid-cols-3 gap-2">
            <Input v-model="pointA.x" placeholder="X" type="number" size="small" />
            <Input v-model="pointA.y" placeholder="Y" type="number" size="small" />
            <Input v-model="pointA.z" placeholder="Z" type="number" size="small" />
          </div>
        </div>
        <div class="space-y-2">
          <div class="text-xs font-medium text-gray-500">坐标 B</div>
          <div class="grid grid-cols-3 gap-2">
            <Input v-model="pointB.x" placeholder="X" type="number" size="small" />
            <Input v-model="pointB.y" placeholder="Y" type="number" size="small" />
            <Input v-model="pointB.z" placeholder="Z" type="number" size="small" />
          </div>
        </div>
      </div>

      <!-- 交换按钮 -->
      <div class="flex justify-center">
        <Tooltip text="交换 A 和 B" position="top">
          <Button type="outline" size="small" @click="swapPoints">
            <template #icon>
              <ArrowsRightLeftIcon class="h-4 w-4" />
            </template>
            交换
          </Button>
        </Tooltip>
      </div>

      <!-- 距离结果 -->
      <div v-if="distances" class="grid grid-cols-3 gap-3">
        <div class="rounded-lg bg-primary-50 px-3 py-2.5 text-center">
          <div class="text-xs text-gray-500">欧氏距离</div>
          <div class="mt-0.5 text-sm font-semibold text-primary-700">{{ fmt(distances.euclidean) }}</div>
        </div>
        <div class="rounded-lg bg-gray-50 px-3 py-2.5 text-center">
          <div class="text-xs text-gray-500">曼哈顿距离</div>
          <div class="mt-0.5 text-sm font-semibold text-gray-700">{{ fmt(distances.manhattan) }}</div>
        </div>
        <div class="rounded-lg bg-gray-50 px-3 py-2.5 text-center">
          <div class="text-xs text-gray-500">切比雪夫距离</div>
          <div class="mt-0.5 text-sm font-semibold text-gray-700">{{ fmt(distances.chebyshev) }}</div>
        </div>
      </div>
      <div v-else class="rounded-lg bg-gray-50 px-3 py-3 text-center text-xs text-gray-400">
        请输入完整的两组坐标
      </div>

      <!-- 地狱门换算 -->
      <div class="border-t border-gray-100 pt-4">
        <div class="text-xs font-medium text-gray-500 mb-2">地狱门坐标换算（1:8 比例）</div>
        <div class="flex items-center gap-2 mb-2">
          <Button
            :type="convertMode === 'toNether' ? 'primary' : 'outline'"
            size="small"
            @click="convertMode = 'toNether'"
          >
            主世界 → 下界
          </Button>
          <Button
            :type="convertMode === 'toOverworld' ? 'primary' : 'outline'"
            size="small"
            @click="convertMode = 'toOverworld'"
          >
            下界 → 主世界
          </Button>
        </div>
        <Input v-model="convertInput" placeholder="输入坐标，如 128 64 -256 或 128,64,-256" size="small" />
        <div v-if="convertResult" class="mt-2 rounded-lg bg-primary-50 px-3 py-2">
          <span class="text-xs text-gray-500">换算结果：</span>
          <span class="text-sm font-medium text-primary-700">{{ convertResult }}</span>
        </div>
      </div>
    </div>
  </section>
</template>
