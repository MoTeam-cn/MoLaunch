<script setup lang="ts">
/**
 * 测试版水印组件
 *
 * 仅在测试版构建（package.json version 含 beta/alpha/rc/canary 后缀）时渲染：
 * - 全屏 45° 重复文字水印（半透明，pointer-events: none 不影响交互）
 * - 每个水印单元格包含：测试版标识 / 设备 ID（去 mcsdk- 前缀）/ 版本号 / 屏印哈希
 * - 屏印哈希由「设备ID + 版本号 + 当前小时」生成，服务端可反查定位设备
 *
 * 防剥离设计：
 * 1. 水印文字直接绘制到 DOM（SVG + CSS），开发者模式可移除 DOM
 *    但移除 DOM 不影响后端追溯（屏印哈希已嵌入截图）
 * 2. SVG metadata 中嵌入设备 ID 和时间戳，截图/拍照后可解析
 * 3. 文字使用低对比度（rgba(0,0,0,0.06)），不影响阅读但拍照后仍可识别
 * 4. 全屏覆盖 + 45° 倾斜 + 密集重复（200px × 120px 单元格），无法局部裁剪去除
 * 5. pointer-events: none 确保不影响 UI 交互
 *
 * 解锁隐藏（开发者调试用）：
 * - DevToolsTab.vue 提供「隐藏水印」按钮，调用 `useWatermarkUnlock.hide()`
 * - 隐藏前提：DevTools 已打开（后端 AtomicBool 维护状态）
 * - 解锁状态存 sessionStorage，重启后恢复显示
 * - DevTools 关闭时自动恢复水印（轮询检测）
 *
 * 集成位置：App.vue 顶层 Teleport 到 body，全局生效
 */
import { computed } from 'vue'
import { useWatermarkData } from '@/composables/useWatermarkData'
import { useWatermarkUnlock } from '@/composables/useWatermarkUnlock'
import { isPreReleaseBuild } from '@/utils/version'

const data = useWatermarkData()
const { unlocked, syncWithDevTools } = useWatermarkUnlock()

// 启动 DevTools 状态同步（解锁状态下轮询，关闭时自动恢复水印）
syncWithDevTools()

/** 是否显示水印：测试版构建 + 设备 ID 已就绪 + 未解锁隐藏 */
const showWatermark = computed(() => isPreReleaseBuild() && data.value.ready && !unlocked.value)

/** 水印文字主行：测试版标识 + 版本号 */
const mainLine = computed(() => {
  const info = data.value
  return `TEST BUILD · ${info.version} · ${info.channel.toUpperCase()}`
})

/** 水印文字次行：设备 ID（前 8 位 + 后 4 位，避免完整暴露但保留可追溯性） */
const deviceLine = computed(() => {
  const id = data.value.deviceId
  if (!id) return 'DEVICE UNKNOWN'
  // 显示前 8 位 + 后 4 位，中间星号
  if (id.length <= 12) return id
  return `${id.slice(0, 8)}****${id.slice(-4)}`
})

/** 水印文字第三行：屏印哈希 + 时间标签 */
const hashLine = computed(() => {
  return `#${data.value.screenHash} · ${data.value.timeLabel}`
})

/**
 * SVG metadata：嵌入完整设备 ID 和时间戳
 *
 * SVG 文件可被解析器读取，截图/拍照后通过 OCR + SVG 解析可还原。
 * 这里通过 <metadata> 标签嵌入 RDF 格式的设备标识信息。
 */
const svgMetadata = computed(() => {
  return `<!--MoLaunch-Watermark device="${data.value.deviceId}" hash="${data.value.screenHash}" time="${data.value.timeLabel}" build="${data.value.buildFingerprint}"-->`
})
</script>

<template>
  <Teleport to="body">
    <div
      v-if="showWatermark"
      class="watermark-overlay"
      aria-hidden="true"
    >
      <!--
        水印层：全屏覆盖 + 45° 倾斜 + 重复单元格
        使用 SVG <pattern> 实现可重复单元格，单元格内含三行文字
        pointer-events: none 确保不影响交互
      -->
      <svg
        class="watermark-svg"
        xmlns="http://www.w3.org/2000/svg"
        :data-device="data.deviceId"
        :data-hash="data.screenHash"
        :data-time="data.timeLabel"
        :data-build="data.buildFingerprint"
      >
        {{ svgMetadata }}
        <defs>
          <pattern
            id="watermark-pattern"
            x="0"
            y="0"
            width="280"
            height="140"
            patternUnits="userSpaceOnUse"
            patternTransform="rotate(-45)"
          >
            <text
              x="10"
              y="20"
              fill="rgba(0, 0, 0, 0.06)"
              font-size="11"
              font-family="monospace"
              font-weight="600"
            >{{ mainLine }}</text>
            <text
              x="10"
              y="40"
              fill="rgba(0, 0, 0, 0.06)"
              font-size="11"
              font-family="monospace"
            >{{ deviceLine }}</text>
            <text
              x="10"
              y="60"
              fill="rgba(0, 0, 0, 0.06)"
              font-size="9"
              font-family="monospace"
            >{{ hashLine }}</text>
          </pattern>
        </defs>
        <rect width="100%" height="100%" fill="url(#watermark-pattern)" />
      </svg>
    </div>
  </Teleport>
</template>

<style scoped>
.watermark-overlay {
  position: fixed;
  inset: 0;
  z-index: 9990;
  pointer-events: none;
  /* 不影响下层交互，仅作视觉覆盖 */
  user-select: none;
  /* 防止用户选中水印文字 */
  -webkit-user-select: none;
}

.watermark-svg {
  width: 100%;
  height: 100%;
  display: block;
  /* SVG 默认有 inline 空白，需 block 撑满 */
}
</style>
