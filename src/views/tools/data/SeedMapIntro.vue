<script setup lang="ts">
/**
 * 种子地图实现原理介绍组件
 *
 * 用于种子地图工具页面底部，介绍技术实现细节。
 * 默认折叠，用户点击标题栏可展开查看原理说明。
 * 风格与 MoLaunchIntro.vue 保持一致（grid-template-rows 0fr→1fr 平滑过渡）。
 *
 * 内容涵盖：
 * - 渲染引擎（OpenLayers + 自定义 MC 投影）
 * - 算法来源（cubiomes fork 分支，WASM 编译）
 * - Worker 架构（多 Worker 串行队列 + WASM 内存安全）
 * - 结构查找（region 遍历 + chunk finder 分块）
 * - 性能优化（tile 缓存、并发控制、低 zoom 跳过）
 */
import { ref } from 'vue'
import { ChevronDownIcon, BeakerIcon } from '@heroicons/vue/24/outline'

const isOpen = ref(false)

function toggle() {
  isOpen.value = !isOpen.value
}
</script>

<template>
  <div class="overflow-hidden rounded-lg border border-gray-200 bg-white">
    <!-- 标题栏（点击展开/折叠） -->
    <!-- 保留原生 button：折叠头（w-full justify-between + 图标旋转 + aria-expanded），
         Button.vue 的 scoped size 类与布局不适合折叠头 -->
    <button
      class="flex w-full items-center justify-between px-4 py-3 text-left transition-colors hover:bg-gray-50"
      :aria-expanded="isOpen"
      @click="toggle"
    >
      <div class="flex items-center gap-2">
        <BeakerIcon class="h-4 w-4 text-primary-500" />
        <span class="text-sm font-semibold text-gray-800">种子地图实现原理</span>
        <span class="rounded bg-gray-100 px-1.5 py-0.5 text-[10px] text-gray-500">点击展开</span>
      </div>
      <ChevronDownIcon
        class="h-4 w-4 flex-none text-gray-400 transition-transform duration-300 ease-in-out"
        :class="isOpen ? 'rotate-180' : ''"
      />
    </button>

    <!-- 内容区（grid-template-rows 0fr→1fr 平滑高度过渡） -->
    <div
      class="grid transition-all duration-300 ease-in-out"
      :class="isOpen ? 'grid-rows-[1fr]' : 'grid-rows-[0fr]'"
    >
      <div class="overflow-hidden">
        <div class="border-t border-gray-100 px-4 py-3">
          <p class="text-[12px] leading-relaxed text-gray-600">
            <span class="font-semibold text-gray-800">种子地图</span> 基于
            <span class="font-medium text-primary-600">OpenLayers</span> 渲染引擎构建，自定义
            <span class="font-medium text-primary-600">MC 投影</span>（1 单位 = 1 方块，extent ±3e7），
            通过 DataTile 按需加载瓦片并由 Worker 生成 ImageBitmap。算法核心是
            <span class="font-medium text-primary-600">cubiomes</span>（C 库，Emscripten 编译为 WASM），
            本项目使用 <span class="font-medium text-primary-600">MoTeam-cn/cubiomes</span> 分支，
            原生支持 MC 1.7~26.2 的群系、高度图与结构生成。WASM 加载采用 new Function +
            instantiateWasm 回调预实例化，避免 res:// 协议下 fetch 兼容性问题；每次调用后重新
            读取 HEAPU8 防止内存增长导致视图 detach。多 Worker 串行队列处理生成与查找任务，
            结构查找按 region 遍历（regionSize × 16 转 block），大范围自动分块调用
            callChunkFinder 合并结果。性能上启用 tile 缓存（cacheSize 4096）、preload 预加载
            相邻级别、低 zoom 跳过结构查找避免阻塞，并通过 pending 机制保证拖动新区域后
            自动补偿刷新。
          </p>
        </div>
      </div>
    </div>
  </div>
</template>
