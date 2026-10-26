<script setup lang="ts">
/**
 * 投影预览 / 编辑
 *
 * 打开 Sponge / Litematica / 原版结构 / 旧版 MCEdit 投影，按轴逐层浏览。
 * 数据常驻后端会话，前端只取当前视口切片，因此不受整体体积限制。
 * 支持画笔、矩形、球体、圆柱、条件替换，撤销重做，旋转镜像，三格式导出；
 * 方块实体（箱子内容、告示牌文字等）随编辑与导出保留。
 * 另有层密度条与等轴测立体预览，避免在空层上盲翻。
 */

import { ExclamationTriangleIcon } from '@heroicons/vue/24/outline'
import SchematicCanvas from './SchematicCanvas.vue'
import SchematicLayerBar from './SchematicLayerBar.vue'
import SchematicOverview from './SchematicOverview.vue'
import SchematicSidePanel from './SchematicSidePanel.vue'
import SchematicToolbar from './SchematicToolbar.vue'
import { useSchematicEditor } from '@/composables/useSchematicEditor'

const editor = useSchematicEditor()
</script>

<template>
  <section class="flex flex-col gap-4 rounded-lg border border-gray-300 bg-white p-4">
    <SchematicToolbar
      v-model:axis="editor.axis.value"
      v-model:layer="editor.layer.value"
      v-model:tool="editor.tool.value"
      v-model:export-format="editor.exportFormat.value"
      v-model:brush-size="editor.brushSize.value"
      v-model:radius="editor.radius.value"
      v-model:cylinder-height="editor.cylinderHeight.value"
      :loading="editor.loading.value"
      :busy="editor.busy.value"
      :session="editor.session.value"
      :plane-layers="editor.plane.value.layers"
      :can-undo="editor.canUndo.value"
      :can-redo="editor.canRedo.value"
      :non-air="editor.nonAir.value"
      :has-overview="editor.overview.value !== null"
      @open="editor.openFile"
      @undo="editor.step('undo')"
      @redo="editor.step('redo')"
      @transform="editor.doTransform"
      @jump-dense="editor.jumpToDensest"
      @zoom="editor.zoom"
      @pan="editor.onPan"
      @export="editor.doExport"
    />

    <p v-if="!editor.session.value" class="text-xs text-gray-500">
      支持 Sponge（.schem / .schematic，v1 与 v2）、Litematica（.litematic）、原版结构（.nbt）
      与旧版 MCEdit 投影。打开后按视口分片加载，大投影也能流畅浏览；方块实体随编辑与导出保留。
    </p>

    <template v-else>
      <div
        v-if="editor.layerEmpty.value"
        class="flex items-center gap-2 rounded border border-amber-300 bg-amber-50 px-3 py-2 text-xs text-amber-800"
      >
        <ExclamationTriangleIcon class="h-4 w-4 shrink-0" />
        第 {{ editor.layer.value + 1 }} 层没有任何方块
        <button
          type="button"
          class="ml-auto rounded border border-amber-400 px-2 py-0.5 hover:bg-amber-100"
          @click="editor.jumpToNearestOccupied"
        >
          跳到最近的非空层
        </button>
      </div>

      <div class="flex flex-wrap items-start gap-4">
        <div class="flex gap-3">
          <SchematicLayerBar
            v-if="editor.overview.value"
            :layer-counts="editor.overview.value.layerCounts"
            :current="editor.layer.value"
            @pick="editor.layer.value = $event"
          />
          <div class="flex flex-col gap-2">
            <SchematicCanvas
              v-if="editor.slice.value"
              :slice="editor.slice.value"
              :palette="editor.session.value.palette"
              :air-index="editor.airIndex.value"
              :cell-size="editor.cellSize.value"
              :interaction="editor.interaction.value"
              :brush-size="editor.brushSize.value"
              @hover="editor.hover.value = $event"
              @select="editor.onSelect"
              @paint="editor.onPaint"
              @point="editor.onPoint"
              @pan="editor.onPan"
            />
            <span class="h-4 text-xs text-gray-500">{{ editor.hoverLabel.value }}</span>
          </div>
        </div>

        <SchematicSidePanel
          v-model:selected-block="editor.selectedBlock.value"
          v-model:replace-from="editor.replaceFrom.value"
          :palette="editor.session.value.palette"
          :counts="editor.paletteCounts.value"
          :top-counts="editor.session.value.counts"
          :non-air="editor.nonAir.value"
          :total="editor.session.value.total"
          :show-picker="editor.needsBlock.value"
          :show-from="editor.tool.value === 'replace'"
        />
      </div>

      <SchematicOverview
        v-if="editor.overview.value"
        :overview="editor.overview.value"
        :palette="editor.session.value.palette"
        :empty-index="editor.overview.value.emptyIndex"
        :world-height="editor.session.value.size[1]"
      />
    </template>
  </section>
</template>
