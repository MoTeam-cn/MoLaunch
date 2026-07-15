/**
 * 搜索进度展示 composable
 * 参考 LaunchLog.vue 的进度条逻辑
 * 搜索是单次请求，没有真实分阶段进度，用伪进度 + 真实阶段切换
 */
import { ref, computed, onUnmounted } from 'vue'

export type SearchStage = 'idle' | 'requesting' | 'merging' | 'done' | 'failed'

export function useSearchProgress() {
  const stage = ref<SearchStage>('idle')
  const percent = ref(0)
  /** 是否超过 5s 且仍在合并阶段（用于显示"资源有点多"提示） */
  const slowMerging = ref(false)
  let progressTimer: number | null = null
  let stageTimer: number | null = null
  let slowTimer: number | null = null
  let startTime = 0

  /** 平台名（根据 source 动态生成提示文字） */
  let platformLabel = 'CurseForge 与 Modrinth'
  /** 是否单平台（单平台不需要"合并去重"文案） */
  let isSinglePlatform = false

  /** 启动伪进度（在真实等待中平滑递增到 85%，完成时跳 100%）
   * @param source 0=全部, 1=仅CF, 2=仅MR
   */
  function start(source?: number) {
    stop()
    isSinglePlatform = source === 1 || source === 2
    platformLabel = source === 1 ? 'CurseForge' : source === 2 ? 'Modrinth' : 'CurseForge 与 Modrinth'
    stage.value = 'requesting'
    percent.value = 5
    slowMerging.value = false
    startTime = Date.now()
    progressTimer = window.setInterval(() => {
      if (percent.value < 85) {
        const remain = 85 - percent.value
        percent.value += Math.max(0.3, remain * 0.08)
      }
    }, 200)
    // 1.5s 后切换到"合并结果"阶段标签（请求大概率还在进行）
    stageTimer = window.setTimeout(() => {
      if (stage.value === 'requesting') stage.value = 'merging'
    }, 1500)
    // 5s 后如果还在 merging 阶段，显示"资源有点多"提示
    slowTimer = window.setTimeout(() => {
      if (stage.value === 'merging' || stage.value === 'requesting') {
        slowMerging.value = true
      }
    }, 5000)
  }

  function finish() {
    stop()
    percent.value = 100
    stage.value = 'done'
  }

  function fail() {
    stop()
    stage.value = 'failed'
  }

  function reset() {
    stop()
    stage.value = 'idle'
    percent.value = 0
    slowMerging.value = false
  }

  function stop() {
    if (progressTimer) { clearInterval(progressTimer); progressTimer = null }
    if (stageTimer) { clearTimeout(stageTimer); stageTimer = null }
    if (slowTimer) { clearTimeout(slowTimer); slowTimer = null }
  }

  const stageText = computed(() => {
    switch (stage.value) {
      case 'requesting': return `正在请求 ${platformLabel}...`
      case 'merging': return isSinglePlatform ? '正在处理结果...' : '正在合并与去重结果...'
      case 'done': return '搜索完成'
      case 'failed': return '搜索失败'
      default: return ''
    }
  })

  onUnmounted(() => stop())

  return { stage, percent, slowMerging, stageText, start, finish, fail, reset }
}
