<script setup lang="ts">
/**
 * 便捷工具子组件（由 Tools.vue 作为侧边栏内容区承载）
 *
 * 本文件为编排层，各工具子组件已拆分到 ./quick-tools/ 目录：
 * - 清理游戏垃圾 → CleanupTool.vue
 * - 内存优化 → MemoryOptimizer.vue
 * - 更多工具敬请期待 → 本文件内保留（占位）
 */
import { SparklesIcon } from '@heroicons/vue/24/outline'
import Tooltip from '@/components/common/Tooltip.vue'
import { toastInfo } from '@/utils/toast'
import CleanupTool from './quick-tools/CleanupTool.vue'
import MemoryOptimizer from './quick-tools/MemoryOptimizer.vue'

// ==================== 敬请期待 ====================
// 15 个规划中的便捷工具，参考 PCL2/HMCL 常见工具集，避免与已实现功能重复
const upcomingTools = [
  { name: '存档备份恢复', desc: '备份和恢复游戏世界存档，支持多版本快照管理' },
  { name: 'Mod 依赖检测', desc: '扫描已安装 Mod，检测依赖冲突和缺失项' },
  { name: '崩溃日志分析', desc: '智能分析崩溃报告，定位问题 Mod 或配置' },
  { name: 'Java 版本管理', desc: '管理多个 Java 运行时，一键切换默认版本' },
  { name: '服务器状态检测', desc: '输入地址检测 Minecraft 服务器在线状态与延迟' },
  { name: '世界存档管理', desc: '管理、导出和分享世界存档，支持 zip 打包' },
  { name: '资源包转换', desc: '在 zip 与文件夹格式间转换资源包/数据包' },
  { name: '版本 JSON 编辑', desc: '可视化编辑版本 JSON，调整继承关系与参数' },
  { name: 'NBT 数据查看', desc: '解析玩家/方块/物品 NBT 数据为可读树形结构' },
  { name: '坐标距离计算', desc: '计算两地坐标距离，辅助地狱门连通与导航' },
  { name: '游戏内调色板', desc: '生成 Minecraft 可用的颜色代码，支持 RGB/HEX 互转' },
  { name: '截图批量管理', desc: '批量浏览、重命名、压缩和导出游戏截图' },
  { name: 'Mod 文件去重', desc: '扫描 mods 目录，找出重复或旧版本 Mod 文件' },
  { name: '启动器数据导出', desc: '导出配置/版本/账号数据，便于迁移或备份' },
  { name: '网络延迟测试', desc: '测试官方/镜像源连接速度，推荐最快下载源' },
]

function onUpcomingClick(name: string) {
  toastInfo(`「${name}」功能敬请期待`)
}
</script>

<template>
  <div class="mx-auto max-w-3xl space-y-6">
    <!-- 清理游戏垃圾 -->
    <CleanupTool />

    <!-- 内存优化 -->
    <MemoryOptimizer />

    <!-- 更多工具敬请期待 -->
    <section class="rounded-lg border border-gray-300 bg-white">
      <div class="flex items-center gap-2 px-5 pt-5 pb-3">
        <SparklesIcon class="h-5 w-5 text-gray-700" />
        <h3 class="text-sm font-semibold text-gray-900">更多工具</h3>
      </div>
      <div class="px-5 pb-5">
        <div class="grid grid-cols-2 gap-3">
          <Tooltip
            v-for="tool in upcomingTools"
            :key="tool.name"
            :text="tool.desc"
          >
            <button
              class="flex w-full items-center gap-2 rounded-lg border border-gray-200 px-4 py-3 text-left transition-colors hover:border-primary-300 hover:bg-primary-50"
              @click="onUpcomingClick(tool.name)"
            >
              <div class="min-w-0 flex-1">
                <div class="truncate text-sm font-medium text-gray-700">{{ tool.name }}</div>
                <div class="truncate text-xs text-gray-400">敬请期待</div>
              </div>
              <span class="flex-none rounded-full bg-gray-100 px-2 py-0.5 text-xs font-medium text-gray-400">
                即将推出
              </span>
            </button>
          </Tooltip>
        </div>
      </div>
    </section>
  </div>
</template>
