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
  ShieldCheckIcon, FolderOpenIcon,
} from '@heroicons/vue/24/outline'
import Button from '@/components/common/Button.vue'
import Input from '@/components/common/Input.vue'
import Select from '@/components/common/Select.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import AlertV2 from '@/components/common/AlertV2.vue'
import StructPopup from './StructPopup.vue'
import SeedMapIntro from './SeedMapIntro.vue'
import LoadSaveModal from './LoadSaveModal.vue'
import { formatCoord } from '@/utils/seedmap/format'
import { useSeedMap } from './useSeedMap'
import { ref } from 'vue'
import { toastSuccess } from '@/utils/toast'

const {
  seedInput, mcVersion, dimension, largeBiomes, userX, userZ,
  versionOptions, dimensionOptions,
  loading, hoverStruct, hoverMarker, mouseBlock, lastClickBlock,
  popupData, mouseBiomeName,
  mapContainer, popupContainer,
  showSpawn, showStronghold, showCoordPanel, showNonViable,
  yCoord, doContour, ymaxLimit,
  structureListForVersion,
  loadSeed, goToUserCoord, zoomIn, zoomOut, resetView,
  copyCoord, goToStruct, closePopup,
  getStructIcon, getStructIconUrl, toggleStructureType, isStructureSelected,
} = useSeedMap()

// 从存档加载弹窗
const showLoadSaveModal = ref(false)

function handleLoadFromSave(payload: { seed: string; mcVersion: number; worldName: string }) {
  seedInput.value = payload.seed
  mcVersion.value = payload.mcVersion
  showLoadSaveModal.value = false
  loadSeed()
  toastSuccess(`已从存档「${payload.worldName}」加载种子`)
}
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
      <div class="ml-auto flex items-center gap-2">
        <Tooltip text="从本地存档加载种子" position="top" :delay="200">
          <Button type="outline" size="small" @click="showLoadSaveModal = true">
            <template #icon><FolderOpenIcon class="h-4 w-4" /></template>
            从存档
          </Button>
        </Tooltip>
        <Button type="primary" size="small" :loading="loading" @click="loadSeed()">
          <template #icon><ArrowPathIcon class="h-4 w-4" /></template>
          加载
        </Button>
      </div>
    </div>

    <!-- 测试警告（顶部醒目位置） -->
    <div class="px-5 pb-3 space-y-2">
      <AlertV2
        type="error"
        message="此种子地图还在测试中，不保护地图准确率，还待进一步更新测试，同时感谢 cubiomes 项目提供算法支持，虽然我们是基于他魔改的分支版本 =_="
      />
      <AlertV2
        type="success"
        message="本项目仍为半成品，目前测试 地图准确率不高，进一步优化好了，当然如果你有更好的方法 欢迎提出来"
      />
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
              :y-coord="yCoord"
              :show-viable="showNonViable"
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
        <!-- 悬浮出生点/要塞提示 -->
        <div v-else-if="hoverMarker" class="absolute top-2 right-2 px-2 py-1 bg-black/80 text-white text-xs rounded pointer-events-none z-10">
          {{ hoverMarker.label }}
          {{ formatCoord(hoverMarker.x, hoverMarker.z) }}
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
            <Tooltip text="复制坐标" position="top" :delay="200">
              <Button
                type="ghost"
                size="mini"
                class="!h-5 !px-1.5 !text-xs"
                @click="copyCoord(lastClickBlock!.x, lastClickBlock!.z)"
              >复制</Button>
            </Tooltip>
          </div>
          <!-- 按钮组 -->
          <div class="flex gap-1">
            <Tooltip text="大型群系" position="top" :delay="200">
              <Button
                :type="largeBiomes ? 'primary' : 'secondary'"
                size="mini"
                :disabled="dimension !== 0"
                class="!w-7 !h-7 !p-0 !flex !justify-center !items-center"
                @click="largeBiomes = !largeBiomes"
              >
                <Squares2X2Icon class="h-4 w-4" />
              </Button>
            </Tooltip>
            <Tooltip text="前往坐标" position="top" :delay="200">
              <Button
                :type="showCoordPanel ? 'primary' : 'secondary'"
                size="mini"
                class="!w-7 !h-7 !p-0 !flex !justify-center !items-center"
                @click="showCoordPanel = !showCoordPanel"
              >
                <MapPinIcon class="h-4 w-4" />
              </Button>
            </Tooltip>
          </div>
        </div>

        <!-- 右下角：缩放控件 -->
        <div class="absolute bottom-2 right-2 flex flex-col gap-1 z-10">
          <Tooltip text="放大" position="left" :delay="200">
            <Button type="secondary" size="mini" class="!w-7 !h-7 !p-0 !flex !justify-center !items-center" @click="zoomIn"><PlusIcon class="h-4 w-4" /></Button>
          </Tooltip>
          <Tooltip text="缩小" position="left" :delay="200">
            <Button type="secondary" size="mini" class="!w-7 !h-7 !p-0 !flex !justify-center !items-center" @click="zoomOut"><MinusIcon class="h-4 w-4" /></Button>
          </Tooltip>
          <Tooltip text="重置视图" position="left" :delay="200">
            <Button type="secondary" size="mini" class="!w-7 !h-7 !p-0 !flex !justify-center !items-center" @click="resetView"><AdjustmentsHorizontalIcon class="h-4 w-4" /></Button>
          </Tooltip>
        </div>

        <!-- 加载遮罩 -->
        <div v-if="loading" class="absolute inset-0 bg-black/30 flex items-center justify-center pointer-events-none z-20">
          <div class="text-white text-sm">加载中...</div>
        </div>
      </div>

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

      <p class="mt-2 text-xs text-gray-400">
        提示：拖拽平移 · 滚轮缩放 · 点击结构看 popup · 点击空白标记坐标 · 已加载区块自动缓存
      </p>
    </div>

    <!-- 底部：实现原理收缩框 -->
    <div class="px-5 pb-5">
      <SeedMapIntro />
    </div>

    <!-- 从存档加载弹窗 -->
    <LoadSaveModal
      :visible="showLoadSaveModal"
      @close="showLoadSaveModal = false"
      @load="handleLoadFromSave"
    />
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
