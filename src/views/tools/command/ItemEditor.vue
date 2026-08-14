<script setup lang="ts">
/**
 * 指令生成 - 物品编辑
 *
 * 配置物品信息生成 /give 指令：物品、数量、目标玩家、
 * 自定义名称（含颜色代码）、Lore、附魔列表。
 */
import { computed, onMounted, ref } from 'vue'
import { CubeIcon, PlusIcon, TrashIcon } from '@heroicons/vue/24/outline'
import Input from '@/components/common/Input.vue'
import Button from '@/components/common/Button.vue'
import { copyToClipboard } from '@/utils/clipboard'
import { loadVersionItems } from '@/utils/recipe-generator/resources'
import type { AssetItem } from '@/utils/recipe-generator/resources'
import { ENCHANTMENTS } from './data'
import { MC_COLORS, TARGETS, buildGiveCommand } from './generator'

const items = ref<AssetItem[]>([])
const keyword = ref('')
const selected = ref<AssetItem | null>(null)
const count = ref(1)
const target = ref('@p')
const itemName = ref('')
const nameColor = ref('gold')
const loreText = ref('')
const loreColor = ref('gray')
const enchants = ref<{ id: string; lvl: number }[]>([])

onMounted(async () => {
  items.value = await loadVersionItems('26.2')
})

const filteredItems = computed(() => {
  const q = keyword.value.trim().toLowerCase()
  if (!q) return items.value.slice(0, 200)
  return items.value.filter(
    (i) =>
      i.id.toLowerCase().includes(q) ||
      i.name.toLowerCase().includes(q) ||
      i.zh.toLowerCase().includes(q),
  ).slice(0, 200)
})

function pickItem(item: AssetItem) {
  selected.value = item
  keyword.value = ''
}

const command = computed(() => {
  if (!selected.value) return ''
  return buildGiveCommand({
    itemId: selected.value.id.replace(/^minecraft:/, ''),
    count: count.value,
    target: target.value,
    enchantments: enchants.value,
    name: itemName.value,
    nameColor: nameColor.value,
    lore: loreText.value.split('\n').filter((l) => l.trim()),
    loreColor: loreColor.value,
  })
})

function addEnchant() {
  const e = ENCHANTMENTS[0]
  enchants.value.push({ id: e.id, lvl: 1 })
}

function removeEnchant(index: number) {
  enchants.value.splice(index, 1)
}

async function copyCommand() {
  if (!command.value) return
  await copyToClipboard(command.value, { toast: true })
}
</script>

<template>
  <section class="rounded-lg border border-gray-300 bg-white">
    <div class="flex items-center gap-2 px-5 pt-5 pb-3">
      <CubeIcon class="h-5 w-5 text-gray-700" />
      <h3 class="text-sm font-semibold text-gray-900">物品编辑（/give）</h3>
    </div>

    <div class="px-5 pb-5 space-y-4">
      <!-- 物品选择 -->
      <div>
        <div class="text-xs font-medium text-gray-500 mb-2">选择物品</div>
        <Input v-model="keyword" placeholder="搜索物品（ID / 名称 / 中文）…" size="small" clearable />
        <div v-if="filteredItems.length && !selected" class="mt-2 max-h-48 overflow-y-auto rounded border border-gray-200">
          <button
            v-for="item in filteredItems"
            :key="item.id"
            type="button"
            class="flex w-full items-center gap-2 px-3 py-1.5 text-left text-xs hover:bg-primary-50"
            @click="pickItem(item)"
          >
            <span class="truncate font-mono text-gray-700">{{ item.id }}</span>
            <span class="truncate text-gray-400">{{ item.zh || item.name }}</span>
          </button>
        </div>
        <div v-if="selected" class="mt-2 flex items-center justify-between rounded border border-primary-200 bg-primary-50 px-3 py-2">
          <span class="text-xs font-medium text-primary-700">{{ selected.id }}</span>
          <Button type="text" size="mini" @click="selected = null">重新选择</Button>
        </div>
      </div>

      <!-- 数量 + 目标 -->
      <div class="grid grid-cols-2 gap-4">
        <div>
          <div class="text-xs font-medium text-gray-500 mb-2">数量</div>
          <Input v-model.number="count" type="number" min="1" size="small" />
        </div>
        <div>
          <div class="text-xs font-medium text-gray-500 mb-2">目标玩家</div>
          <select v-model="target" class="w-full rounded border border-gray-300 px-2 py-1.5 text-xs outline-none focus:border-primary-500">
            <option v-for="t in TARGETS" :key="t.id" :value="t.id">{{ t.label }}</option>
          </select>
        </div>
      </div>

      <!-- 自定义名称 -->
      <div class="grid grid-cols-[1fr_auto] gap-4 items-end">
        <div>
          <div class="text-xs font-medium text-gray-500 mb-2">自定义名称（可选）</div>
          <Input v-model="itemName" placeholder="如：神之剑" size="small" />
        </div>
        <div>
          <div class="text-xs font-medium text-gray-500 mb-2">颜色</div>
          <select v-model="nameColor" class="rounded border border-gray-300 px-2 py-1.5 text-xs outline-none focus:border-primary-500">
            <option v-for="c in MC_COLORS" :key="c" :value="c">{{ c }}</option>
          </select>
        </div>
      </div>

      <!-- Lore -->
      <div>
        <div class="text-xs font-medium text-gray-500 mb-2">Lore 描述（每行一条，可选）</div>
        <Input v-model="loreText" textarea :rows="3" placeholder="第一行&#10;第二行" size="small" />
      </div>

      <!-- 附魔 -->
      <div>
        <div class="flex items-center justify-between mb-2">
          <span class="text-xs font-medium text-gray-500">附魔</span>
          <Button type="outline" size="mini" @click="addEnchant">
            <template #icon><PlusIcon class="h-3.5 w-3.5" /></template>
            添加
          </Button>
        </div>
        <div v-if="enchants.length" class="space-y-2">
          <div v-for="(e, idx) in enchants" :key="idx" class="flex items-center gap-2">
            <select v-model="e.id" class="flex-1 rounded border border-gray-300 px-2 py-1.5 text-xs outline-none focus:border-primary-500">
              <option v-for="opt in ENCHANTMENTS" :key="opt.id" :value="opt.id">{{ opt.name }}（{{ opt.id }}）</option>
            </select>
            <Input v-model.number="e.lvl" type="number" min="1" max="255" size="small" width="64px" />
            <Button type="text" size="mini" @click="removeEnchant(idx)">
              <template #icon><TrashIcon class="h-4 w-4 text-red-500" /></template>
            </Button>
          </div>
        </div>
      </div>

      <!-- 指令结果 -->
      <div class="border-t border-gray-100 pt-4">
        <div class="text-xs font-medium text-gray-500 mb-2">生成指令</div>
        <div class="rounded-lg bg-gray-50 px-3 py-2.5 font-mono text-xs break-all text-gray-700 min-h-[2.5rem]">
          {{ command || '请先选择物品' }}
        </div>
        <div class="mt-2 flex justify-end">
          <Button size="small" :disabled="!command" @click="copyCommand">
            复制指令
          </Button>
        </div>
      </div>
    </div>
  </section>
</template>
