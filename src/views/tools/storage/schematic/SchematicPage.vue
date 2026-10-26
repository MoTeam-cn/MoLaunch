<script setup lang="ts">
/**
 * 投影预览 / 编辑
 *
 * 打开 Sponge / Litematica / 原版结构 / 旧版 MCEdit 投影，按轴逐层浏览。
 * 数据常驻后端会话，前端只取当前视口切片，因此不受整体体积限制。
 * 支持画笔、矩形、球体、圆柱、条件替换，撤销重做，旋转镜像，三格式导出；
 * 方块实体（箱子内容、告示牌文字等）随编辑与导出保留。
 */

import SchematicCanvas from './SchematicCanvas.vue'
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
      @open="editor.openFile"
      @undo="editor.step('undo')"
      @redo="editor.step('redo')"
      @transform="editor.doTransform"
      @zoom="editor.zoom"
      @pan="editor.onPan"
      @export="editor.doExport"
    />

    <p v-if="!editor.session.value" class="text-xs text-gray-500">
      支持 Sponge（.schem / .schematic，v1 与 v2）、Litematica（.litematic）、原版结构（.nbt）
      与旧版 MCEdit 投影。打开后按视口分片加载，大投影也能流畅浏览；方块实体随编辑与导出保留。
    </p>

    <div v-else class="flex flex-wrap items-start gap-4">
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
  </section>
</template>
