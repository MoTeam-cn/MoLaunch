<script setup lang="ts">
import { defineAsyncComponent } from 'vue'
/**
 * 种子地图结构筛选栏：出生点/要塞/校验开关 + 按版本可选结构图标按钮。
 * 从 SeedMap.vue 拆出，避免 Vue 组件超 300 行。
 */
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Tooltip = defineAsyncComponent(() => import('@/components/common/Tooltip.vue'))
import { getStructIcon, getStructIconUrl } from '@/utils/seedmap/constants'
import type { StructureTypeConfig } from '@/utils/seedmap/structures'
import { HomeIcon, KeyIcon, ShieldCheckIcon } from '@heroicons/vue/24/outline'

const showSpawn = defineModel<boolean>('showSpawn', { required: true })
const showStronghold = defineModel<boolean>('showStronghold', { required: true })
const showNonViable = defineModel<boolean>('showNonViable', { required: true })

defineProps<{
  dimension: number
  structureListForVersion: StructureTypeConfig[]
  isStructureSelected: (name: string) => boolean
  toggleStructureType: (name: string) => void
}>()
</script>

<template>
  <!-- 结构筛选栏：图标-only + Tooltip 悬停显示文字，节省空间单行排列 -->
  <div class="mt-3 flex flex-wrap items-center gap-1">
    <span class="text-xs text-gray-600 font-medium mr-1">显示：</span>
    <Tooltip text="出生点" position="top" :delay="200">
      <Button
        :type="showSpawn ? 'primary' : 'outline'"
        size="mini"
        class="!w-7 !h-7 !p-0 !flex !justify-center !items-center"
        @click="showSpawn = !showSpawn"
      >
        <HomeIcon class="h-4 w-4" />
      </Button>
    </Tooltip>
    <Tooltip v-if="dimension === 0" text="要塞" position="top" :delay="200">
      <Button
        :type="showStronghold ? 'primary' : 'outline'"
        size="mini"
        class="!w-7 !h-7 !p-0 !flex !justify-center !items-center"
        @click="showStronghold = !showStronghold"
      >
        <KeyIcon class="h-4 w-4" />
      </Button>
    </Tooltip>
    <Tooltip
      :text="showNonViable ? '显示全部候选（含未校验）' : '仅显示已校验'"
      position="top"
      :delay="200"
    >
      <Button
        :type="showNonViable ? 'primary' : 'outline'"
        size="mini"
        class="!w-7 !h-7 !p-0 !flex !justify-center !items-center"
        @click="showNonViable = !showNonViable"
      >
        <ShieldCheckIcon class="h-4 w-4" />
      </Button>
    </Tooltip>
    <div class="w-px h-4 bg-gray-200 mx-1" />
    <Tooltip
      v-for="s in structureListForVersion"
      :key="s.name"
      :text="getStructIcon(s.name).label"
      position="top"
      :delay="200"
    >
      <Button
        :type="isStructureSelected(s.name) ? 'primary' : 'outline'"
        size="mini"
        class="!w-7 !h-7 !p-0 !flex !justify-center !items-center"
        @click="toggleStructureType(s.name)"
      >
        <img
          v-if="getStructIconUrl(s.name)"
          :src="getStructIconUrl(s.name)"
          class="w-4 h-4"
          :style="{ opacity: isStructureSelected(s.name) ? 1 : 0.5 }"
        />
        <span
          v-else
          class="w-3 h-3 inline-block rounded-full"
          :style="{ backgroundColor: getStructIcon(s.name).color, opacity: isStructureSelected(s.name) ? 1 : 0.5 }"
        />
      </Button>
    </Tooltip>
  </div>
</template>
