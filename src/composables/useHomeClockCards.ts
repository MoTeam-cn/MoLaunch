/**
 * 主页时钟卡片 composable（从 HomeClockCard.vue 抽离）
 *
 * 封装时钟状态（当前时间 + 每秒更新定时器）+ 轮播信息卡片加载（内存/版本/启动历史/缓存）与自动翻页。
 */
import { ref, computed, onMounted, onUnmounted } from 'vue'
import * as tauri from '@/utils/tauri'
import { formatBytes, formatDate } from '@/utils/format'
import {
  CpuChipIcon,
  CircleStackIcon,
  ClockIcon,
  ChartBarIcon,
} from '@heroicons/vue/24/outline'

export interface InfoCard {
  key: string
  icon: typeof CpuChipIcon
  label: string
  value: string
  sub?: string
  /** 进度条百分比（0-100），不传则不显示 */
  progress?: number
  /** 进度条颜色（tailwind class） */
  progressColor?: string
}

/** 轮播间隔（毫秒） */
const CAROUSEL_INTERVAL = 6000

export function useHomeClockCards() {
  // ==================== 时钟 ====================
  const now = ref(new Date())
  let clockTimer: ReturnType<typeof setInterval> | null = null

  const timeText = computed(() => {
    const h = String(now.value.getHours()).padStart(2, '0')
    const m = String(now.value.getMinutes()).padStart(2, '0')
    return `${h}:${m}`
  })

  const secondsText = computed(() => String(now.value.getSeconds()).padStart(2, '0'))

  const dateText = computed(() => {
    const weekdays = ['周日', '周一', '周二', '周三', '周四', '周五', '周六']
    const y = now.value.getFullYear()
    const m = String(now.value.getMonth() + 1).padStart(2, '0')
    const d = String(now.value.getDate()).padStart(2, '0')
    return `${y}-${m}-${d} ${weekdays[now.value.getDay()]}`
  })

  // ==================== 轮播信息卡片 ====================
  const cards = ref<InfoCard[]>([])
  const currentIndex = ref(0)
  let carouselTimer: ReturnType<typeof setInterval> | null = null

  const currentCard = computed(() => cards.value[currentIndex.value] ?? null)

  /** 内存使用 */
  async function loadMemoryCard(): Promise<InfoCard | null> {
    try {
      const mem = await tauri.getSystemMemory()
      const percent = Math.round(mem.usage_percent)
      const color = percent >= 80 ? 'bg-red-500'
        : percent >= 60 ? 'bg-yellow-500'
        : 'bg-green-500'
      return {
        key: 'memory',
        icon: CpuChipIcon,
        label: '内存使用',
        value: `${percent}%`,
        sub: `${formatBytes(mem.used)} / ${formatBytes(mem.total)}`,
        progress: percent,
        progressColor: color,
      }
    } catch {
      return null
    }
  }

  /** 已安装版本数 */
  async function loadVersionsCard(): Promise<InfoCard | null> {
    try {
      const list = await tauri.listInstalledVersionsWithType()
      return {
        key: 'versions',
        icon: ChartBarIcon,
        label: '已安装版本',
        value: `${list.length} 个`,
        sub: list.length > 0 ? `最近：${list[0].id}` : '暂无版本',
      }
    } catch {
      return null
    }
  }

  /** 最近一次启动 */
  async function loadLaunchHistoryCard(): Promise<InfoCard | null> {
    try {
      const history = await tauri.getLaunchHistory()
      if (history.length === 0) {
        return {
          key: 'history',
          icon: ClockIcon,
          label: '启动历史',
          value: '暂无记录',
          sub: '启动一次游戏后将显示',
        }
      }
      const latest = history[0]
      const exitInfo = latest.exit_code === null
        ? '运行中'
        : latest.exit_code === 0
          ? '正常退出'
          : `异常退出 (${latest.exit_code})`
      return {
        key: 'history',
        icon: ClockIcon,
        label: '最近启动',
        value: latest.version_id,
        sub: `${formatDate(latest.launch_time)} · ${exitInfo}`,
      }
    } catch {
      return null
    }
  }

  /** 缓存占用 */
  async function loadCacheCard(): Promise<InfoCard | null> {
    try {
      const stats = await tauri.getCacheStats()
      const all = [
        ...stats.cache,
        ...stats.cacheTemp,
        ...stats.cacheApp,
      ]
      const totalSize = all.reduce((s, e) => s + e.totalSize, 0)
      const totalFiles = all.reduce((s, e) => s + e.fileCount, 0)
      return {
        key: 'cache',
        icon: CircleStackIcon,
        label: '缓存占用',
        value: formatBytes(totalSize),
        sub: `${totalFiles} 个文件`,
      }
    } catch {
      return null
    }
  }

  /** 加载所有信息卡片（失败的跳过） */
  async function loadCards() {
    const results = await Promise.all([
      loadMemoryCard(),
      loadVersionsCard(),
      loadLaunchHistoryCard(),
      loadCacheCard(),
    ])
    const valid = results.filter((c): c is InfoCard => c !== null)
    cards.value = valid
    if (currentIndex.value >= valid.length) {
      currentIndex.value = 0
    }
  }

  function startCarousel() {
    stopCarousel()
    carouselTimer = setInterval(() => {
      if (cards.value.length === 0) return
      currentIndex.value = (currentIndex.value + 1) % cards.value.length
    }, CAROUSEL_INTERVAL)
  }

  function stopCarousel() {
    if (carouselTimer) {
      clearInterval(carouselTimer)
      carouselTimer = null
    }
  }

  onMounted(async () => {
    clockTimer = setInterval(() => {
      now.value = new Date()
    }, 1000)
    await loadCards()
    startCarousel()
  })

  onUnmounted(() => {
    if (clockTimer) clearInterval(clockTimer)
    stopCarousel()
  })

  return {
    timeText,
    secondsText,
    dateText,
    cards,
    currentIndex,
    currentCard,
    startCarousel,
  }
}
