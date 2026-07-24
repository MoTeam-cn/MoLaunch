<script setup lang="ts">
/**
 * 种子地图工具（OpenLayers 渲染引擎版）
 *
 * 逻辑见 useSeedMap.ts，样式/调色板见 @/utils/seedmap/constants.ts
 * 交互：拖拽平移 · 滚轮缩放 · 点击结构看 popup · 点击空白标记坐标 · 输入坐标前往
 * 渲染引擎：OpenLayers（tile 缓存 + 内置交互），不再手写 Canvas
 */
import {
  MapIcon, ArrowPathIcon, HomeIcon, KeyIcon,
  Squares2X2Icon, MapPinIcon, PlusIcon, MinusIcon, AdjustmentsHorizontalIcon,
} from '@heroicons/vue/24/outline'
import Button from '@/components/common/Button.vue'
import Input from '@/components/common/Input.vue'
import Select from '@/components/common/Select.vue'
import StructPopup from './StructPopup.vue'
import { formatCoord } from '@/utils/seedmap/format'
import { useSeedMap } from './useSeedMap'

const {
  seedInput, mcVersion, dimension, largeBiomes, userX, userZ,
  versionOptions, dimensionOptions,
  loading, hoverStruct, mouseBlock, lastClickBlock,
  popupData, mouseBiomeName,
  mapContainer, popupContainer,
  showSpawn, showStronghold, showCoordPanel,
  yCoord, doContour, ymaxLimit,
  structureListForVersion,
  loadSeed, goToUserCoord, zoomIn, zoomOut, resetView,
  copyCoord, goToStruct, closePopup,
  getStructIcon, getStructIconUrl, toggleStructureType, isStructureSelected,
} = useSeedMap()
</script>

<template>
  <section class="rounded-lg border border-gray-300 bg-white">
    <div class="flex items-center gap-2 px-5 pt-5 pb-3">
      <MapIcon class="h-5 w-5 text-gray-700" />
      <h3 class="text-sm font-semibold text-gray-900">种子地图</h3>
      <span class="ml-auto text-xs text-gray-400">基于 cubiomes + OpenLayers</span>
    </div>

    <!-- 控制栏：种子 + 版本 + 维度 + 加载 -->
    <div class="px-5 pb-3 flex flex-wrap items-center gap-3">
      <div class="flex items-center gap-2">
        <span class="text-xs text-gray-600">种子</span>
        <Input v-model="seedInput" placeholder="输入种子" width="200px" @keydown.enter="loadSeed()" />
      </div>
      <div class="flex items-center gap-2">
        <span class="text-xs text-gray-600">版本</span>
        <Select v-model="mcVersion" :options="versionOptions" class="w-24" />
      </div>
      <div class="flex items-center gap-2">
        <span class="text-xs text-gray-600">维度</span>
        <Select v-model="dimension" :options="dimensionOptions" class="w-24" />
      </div>
      <Button type="primary" size="small" :loading="loading" class="ml-auto" @click="loadSeed()">
        <template #icon><ArrowPathIcon class="h-4 w-4" /></template>
        加载
      </Button>
    </div>

    <!-- OpenLayers 地图容器 -->
    <div class="px-5 pb-5">
      <div class="relative w-full border border-gray-200 rounded overflow-hidden bg-gray-900">
        <div ref="mapContainer" class="w-full" style="height: 480px;">
          <!-- OL Overlay popup 容器：始终挂载（OL Overlay 通过 element 引用），
               内部 StructPopup 用 v-if 控制（避免空 popup 渲染） -->
          <div ref="popupContainer" class="ol-popup-host">
            <StructPopup
              v-if="popupData"
              :struct="popupData.struct"
              @goto="goToStruct(popupData!.struct.x, popupData!.struct.z)"
              @close="closePopup"
            />
          </div>
        </div>

        <!-- 悬浮坐标 + 群系名 -->
        <div v-if="mouseBlock" class="absolute top-2 left-2 px-2 py-1 bg-black/70 text-white text-xs rounded font-mono pointer-events-none z-10">
          {{ formatCoord(mouseBlock.x, mouseBlock.z) }}
          <span v-if="mouseBiomeName" class="ml-1 text-gray-300">{{ mouseBiomeName }}</span>
        </div>
        <!-- 悬浮结构提示 -->
        <div v-if="hoverStruct" class="absolute top-2 right-2 px-2 py-1 bg-black/80 text-white text-xs rounded pointer-events-none z-10">
          {{ getStructIcon(hoverStruct.stype).label }}
          {{ formatCoord(hoverStruct.x, hoverStruct.z) }}
        </div>

        <!-- 左下角：大型群系 + 坐标输入 + 点击坐标 -->
        <div class="absolute bottom-2 left-2 flex flex-col gap-1 z-10">
          <!-- 坐标输入面板（展开时显示） -->
          <div v-if="showCoordPanel" class="mb-1 p-2 bg-white/95 rounded shadow-lg flex flex-col gap-1">
            <div class="flex items-center gap-1">
              <Input v-model="userX" placeholder="X" width="60px" @keydown.enter="goToUserCoord" />
              <Input v-model="userZ" placeholder="Z" width="60px" @keydown.enter="goToUserCoord" />
              <Button type="primary" size="mini" @click="goToUserCoord">前往</Button>
            </div>
            <div class="flex items-center gap-1">
              <span class="text-xs text-gray-600 w-6">Y</span>
              <Input v-model.number="yCoord" type="number" width="60px" />
              <Button
                :type="doContour ? 'primary' : 'secondary'"
                size="mini"
                class="!h-7"
                @click="doContour = !doContour"
              >等高线</Button>
              <span class="text-xs text-gray-600">限高</span>
              <Input v-model.number="ymaxLimit" type="number" width="50px" placeholder="0" />
            </div>
          </div>
          <!-- 点击坐标显示（含复制按钮，复用项目 Button.vue） -->
          <div v-if="lastClickBlock" class="flex items-center gap-1">
            <span class="px-2 py-0.5 bg-black/70 text-white text-xs rounded font-mono pointer-events-none">
              {{ formatCoord(lastClickBlock.x, lastClickBlock.z) }}
            </span>
            <Button
              type="ghost"
              size="mini"
              class="!h-5 !px-1.5 !text-xs"
              title="复制坐标"
              @click="copyCoord(lastClickBlock!.x, lastClickBlock!.z)"
            >复制</Button>
          </div>
          <!-- 按钮组 -->
          <div class="flex gap-1">
            <Button
              :type="largeBiomes ? 'primary' : 'secondary'"
              size="mini"
              :disabled="dimension !== 0"
              class="!w-7 !h-7 !p-0"
              title="大型群系"
              @click="largeBiomes = !largeBiomes"
            >
              <Squares2X2Icon class="h-4 w-4" />
            </Button>
            <Button
              :type="showCoordPanel ? 'primary' : 'secondary'"
              size="mini"
              class="!w-7 !h-7 !p-0"
              title="前往坐标"
              @click="showCoordPanel = !showCoordPanel"
            >
              <MapPinIcon class="h-4 w-4" />
            </Button>
          </div>
        </div>

        <!-- 右下角：缩放控件 -->
        <div class="absolute bottom-2 right-2 flex flex-col gap-1 z-10">
          <Button type="secondary" size="mini" class="!w-7 !h-7 !p-0" @click="zoomIn"><PlusIcon class="h-4 w-4" /></Button>
          <Button type="secondary" size="mini" class="!w-7 !h-7 !p-0" @click="zoomOut"><MinusIcon class="h-4 w-4" /></Button>
          <Button type="secondary" size="mini" class="!w-7 !h-7 !p-0" @click="resetView"><AdjustmentsHorizontalIcon class="h-4 w-4" /></Button>
        </div>

        <!-- 加载遮罩 -->
        <div v-if="loading" class="absolute inset-0 bg-black/30 flex items-center justify-center pointer-events-none z-20">
          <div class="text-white text-sm">加载中...</div>
        </div>
      </div>

      <!-- 结构筛选栏 -->
      <div class="mt-3 flex flex-wrap items-center gap-2">
        <span class="text-xs text-gray-600 font-medium mr-1">显示：</span>
        <Button
          :type="showSpawn ? 'primary' : 'outline'"
          size="mini"
          @click="showSpawn = !showSpawn"
        >
          <template #icon><HomeIcon class="h-3.5 w-3.5" /></template>
          出生点
        </Button>
        <Button
          v-if="dimension === 0"
          :type="showStronghold ? 'primary' : 'outline'"
          size="mini"
          @click="showStronghold = !showStronghold"
        >
          <template #icon><KeyIcon class="h-3.5 w-3.5" /></template>
          要塞
        </Button>
        <div class="w-px h-4 bg-gray-200 mx-1" />
        <Button
          v-for="s in structureListForVersion"
          :key="s.name"
          :type="isStructureSelected(s.name) ? 'primary' : 'outline'"
          size="mini"
          @click="toggleStructureType(s.name)"
        >
          <img
            v-if="getStructIconUrl(s.name)"
            :src="getStructIconUrl(s.name)"
            class="w-4 h-4 mr-1 align-middle inline-block"
            :style="{ opacity: isStructureSelected(s.name) ? 1 : 0.5 }"
          />
          <span
            v-else
            class="w-2.5 h-2.5 mr-1 inline-block rounded-full align-middle"
            :style="{ backgroundColor: getStructIcon(s.name).color, opacity: isStructureSelected(s.name) ? 1 : 0.5 }"
          />
          {{ getStructIcon(s.name).label }}
        </Button>
      </div>

      <p class="mt-2 text-xs text-gray-400">
        提示：拖拽平移 · 滚轮缩放 · 点击结构看 popup · 点击空白标记坐标 · 已加载区块自动缓存
      </p>
    </div>
  </section>
</template>

<style scoped>
/* OL canvas 渲染时保持像素清晰（nearest neighbor），避免群系边界模糊 */
:deep(.ol-viewport canvas) {
  image-rendering: pixelated;
  image-rendering: -moz-crisp-edges;
  image-rendering: crisp-edges;
}
</style>
