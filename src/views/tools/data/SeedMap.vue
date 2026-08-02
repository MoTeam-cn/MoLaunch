<script setup lang="ts">
/**
 * 种子地图工具（OpenLayers 渲染引擎版）
 *
 * 逻辑见 useSeedMap.ts，样式/调色板见 @/utils/seedmap/constants.ts
 * 交互：拖拽平移 · 滚轮缩放 · 点击结构看 popup · 点击空白标记坐标 · 输入坐标前往
 * 渲染引擎：OpenLayers（tile 缓存 + 内置交互），不再手写 Canvas
 *
 * 拆分（保持主文件 ≤ 300 行约束）：
 * - SeedMapControls：地图叠加控件（悬浮提示/坐标面板/缩放/加载遮罩）
 * - SeedMapSidebar：结构筛选栏
 */
import {
  MapIcon, ArrowPathIcon, FolderOpenIcon,
} from '@heroicons/vue/24/outline'
import Button from '@/components/common/Button.vue'
import Input from '@/components/common/Input.vue'
import Select from '@/components/common/Select.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import AlertV2 from '@/components/common/AlertV2.vue'
import StructPopup from './StructPopup.vue'
import SeedMapIntro from './SeedMapIntro.vue'
import LoadSaveModal from './LoadSaveModal.vue'
import SeedMapControls from './SeedMapControls.vue'
import SeedMapSidebar from './SeedMapSidebar.vue'
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
  toggleStructureType, isStructureSelected,
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

        <!-- 地图叠加控件（悬浮提示 / 坐标面板 / 缩放 / 加载遮罩） -->
        <SeedMapControls
          v-model:userX="userX"
          v-model:userZ="userZ"
          v-model:yCoord="yCoord"
          v-model:doContour="doContour"
          v-model:ymaxLimit="ymaxLimit"
          v-model:largeBiomes="largeBiomes"
          v-model:showCoordPanel="showCoordPanel"
          :mouse-block="mouseBlock"
          :mouse-biome-name="mouseBiomeName"
          :hover-struct="hoverStruct"
          :hover-marker="hoverMarker"
          :last-click-block="lastClickBlock"
          :dimension="dimension"
          :loading="loading"
          @go-to-user-coord="goToUserCoord"
          @copy-coord="copyCoord"
          @zoom-in="zoomIn"
          @zoom-out="zoomOut"
          @reset-view="resetView"
        />
      </div>

      <!-- 结构筛选栏 -->
      <SeedMapSidebar
        v-model:showSpawn="showSpawn"
        v-model:showStronghold="showStronghold"
        v-model:showNonViable="showNonViable"
        :dimension="dimension"
        :structure-list-for-version="structureListForVersion"
        :is-structure-selected="isStructureSelected"
        :toggle-structure-type="toggleStructureType"
      />

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
