<script setup lang="ts">
/**
 * 插件系统运行逻辑流程图（5 步等宽方框 + 箭头）
 */
const flowSteps = [
  {
    step: 1,
    title: '扫描目录',
    desc: '启动器启动时扫描 <base_dir>/plugins/ 下的子目录',
  },
  {
    step: 2,
    title: '解析清单',
    desc: '读取每个目录的 manifest.json，校验 ID 合法性与入口文件',
  },
  {
    step: 3,
    title: '加载插件',
    desc: '内置插件直接编译进 Vue；外部插件通过 iframe sandbox 隔离加载',
  },
  {
    step: 4,
    title: '权限校验',
    desc: '外部插件调用 SDK 时，根据 manifest.permissions 白名单逐次校验',
  },
  {
    step: 5,
    title: '事件桥接',
    desc: '游戏启动/退出等事件通过 postMessage 桥接到沙箱内',
  },
]
</script>

<template>
  <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
    <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">插件系统运行逻辑</h3>
    <div class="px-5 pb-5">
      <!-- 流程图：5 个等宽等高方框 + 4 个箭头分隔 -->
      <div class="overflow-x-auto pb-2">
        <div class="flex items-stretch min-w-[760px]">
          <template v-for="(s, idx) in flowSteps" :key="s.step">
            <!-- 步骤卡片（flex-1 等宽，items-stretch 等高） -->
            <div class="flex flex-1 flex-col rounded-md border border-gray-200 bg-gray-50/50 p-3">
              <div class="mb-2 flex items-center gap-1.5">
                <span class="flex h-5 w-5 flex-none items-center justify-center rounded-full bg-primary-500 text-[10px] font-bold text-white">
                  {{ s.step }}
                </span>
                <span class="text-xs font-medium text-gray-900">{{ s.title }}</span>
              </div>
              <p class="text-[11px] leading-relaxed text-gray-500">{{ s.desc }}</p>
            </div>
            <!-- 箭头（独立元素，不参与伸缩） -->
            <div v-if="idx < flowSteps.length - 1" class="flex flex-none items-center px-1.5 text-gray-300">
              <svg
                class="h-4 w-4"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
                stroke-width="2"
              >
                <path stroke-linecap="round" stroke-linejoin="round" d="M8.25 4.5l7.5 7.5-7.5 7.5" />
              </svg>
            </div>
          </template>
        </div>
      </div>
    </div>
  </div>
</template>
