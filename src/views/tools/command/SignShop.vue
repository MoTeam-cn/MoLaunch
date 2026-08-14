<script setup lang="ts">
/**
 * 指令生成 - 告示牌商店
 *
 * 配置告示牌四行文字（含颜色代码）、放置坐标与朝向，
 * 生成 /setblock 指令放置带文本的告示牌，用于搭建告示牌商店。
 */
import { computed, ref, defineAsyncComponent } from 'vue'
import { DocumentTextIcon } from '@heroicons/vue/24/outline'
const Input = defineAsyncComponent(() => import('@/components/common/Input.vue'))
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Select = defineAsyncComponent(() => import('@/components/common/Select.vue'))
import { copyToClipboard } from '@/utils/clipboard'
import { SIGN_FACINGS, SIGN_IDS } from './data'
import { buildSignShopCommand } from './generator'
const ColorSelect = defineAsyncComponent(() => import('./ColorSelect.vue'))

const signId = ref(SIGN_IDS[0])
const facing = ref(SIGN_FACINGS[0].id)
const x = ref('~')
const y = ref('~')
const z = ref('~')
const lines = ref<string[]>(['', '', '', ''])
const textColor = ref('white')

const signOptions = computed(() => SIGN_IDS.map((s) => ({ label: s, value: s })))
const facingOptions = computed(() => SIGN_FACINGS.map((f) => ({ label: f.name, value: f.id })))

const command = computed(() =>
  buildSignShopCommand({
    signId: signId.value,
    facing: facing.value,
    x: x.value,
    y: y.value,
    z: z.value,
    lines: lines.value,
    textColor: textColor.value,
  }),
)

async function copyCommand() {
  await copyToClipboard(command.value, { toast: true })
}
</script>

<template>
  <section class="rounded-lg border border-gray-300 bg-white">
    <div class="flex items-center gap-2 px-5 pt-5 pb-3">
      <DocumentTextIcon class="h-5 w-5 text-gray-700" />
      <h3 class="text-sm font-semibold text-gray-900">告示牌商店（/setblock）</h3>
    </div>

    <div class="px-5 pb-5 space-y-4">
      <!-- 告示牌类型 + 朝向 -->
      <div class="grid grid-cols-2 gap-4">
        <div>
          <div class="text-xs font-medium text-gray-500 mb-2">告示牌类型</div>
          <Select v-model="signId" :options="signOptions" />
        </div>
        <div>
          <div class="text-xs font-medium text-gray-500 mb-2">朝向</div>
          <Select v-model="facing" :options="facingOptions" />
        </div>
      </div>

      <!-- 坐标 -->
      <div>
        <div class="text-xs font-medium text-gray-500 mb-2">放置坐标（支持 ~ 相对坐标）</div>
        <div class="grid grid-cols-3 gap-2">
          <Input v-model="x" placeholder="X" size="small" />
          <Input v-model="y" placeholder="Y" size="small" />
          <Input v-model="z" placeholder="Z" size="small" />
        </div>
      </div>

      <!-- 四行文字 -->
      <div>
        <div class="flex items-center justify-between mb-2">
          <span class="text-xs font-medium text-gray-500">文字内容（最多 4 行）</span>
          <div class="flex items-center gap-1.5">
            <span class="text-xs text-gray-400">颜色</span>
            <ColorSelect v-model="textColor" />
          </div>
        </div>
        <div class="space-y-2">
          <Input v-for="idx in 4" :key="idx" v-model="lines[idx - 1]" :placeholder="`第 ${idx} 行${idx === 2 ? '（如：价格 5 绿宝石）' : ''}`" size="small" />
        </div>
      </div>

      <!-- 指令结果 -->
      <div class="border-t border-gray-100 pt-4">
        <div class="text-xs font-medium text-gray-500 mb-2">生成指令</div>
        <div class="rounded-lg bg-gray-50 px-3 py-2.5 font-mono text-xs break-all text-gray-700 min-h-[2.5rem]">
          {{ command }}
        </div>
        <div class="mt-2 flex justify-end">
          <Button size="small" @click="copyCommand">
            复制指令
          </Button>
        </div>
      </div>
    </div>
  </section>
</template>
