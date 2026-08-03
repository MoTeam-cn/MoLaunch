/**
 * 水印数据 composable：设备 ID / 版本号 / 构建指纹 / 屏印哈希 / 时间标签
 *
 * 屏印哈希按小时分桶（同设备同小时一致便于追溯，跨小时变化防剥离），不直接包含设备 ID。
 */

import { computed, ref, onMounted } from 'vue'
import { useSdkStore } from '@/stores/sdk'
import { stripMcsdkPrefix } from '@/utils/online/device-id'
import { getVersionInfo, getBuildFingerprint } from '@/utils/version'

/**
 * 简易字符串哈希（djb2 变体）
 *
 * 不引入第三方库，避免 vendor chunk 膨胀。返回 8 位十六进制字符串。
 * 同样的输入永远得到同样的输出，便于服务端反查。
 */
function hashString(input: string): string {
  let hash = 5381
  for (let i = 0; i < input.length; i++) {
    hash = ((hash << 5) + hash + input.charCodeAt(i)) >>> 0
  }
  return hash.toString(16).padStart(8, '0').toUpperCase()
}

export interface WatermarkData {
  /** 设备 ID（已去除 mcsdk- 前缀） */
  deviceId: string
  /** 应用版本号（如 0.1.0-beta.1） */
  version: string
  /** 发布通道（beta/alpha/rc/canary/stable） */
  channel: string
  /** 构建指纹（版本号字符串） */
  buildFingerprint: string
  /** 屏印哈希（设备ID + 版本号 + 当前小时，便于服务端反查） */
  screenHash: string
  /** 当前时间字符串（用于水印文字展示） */
  timeLabel: string
  /** 是否已就绪（设备ID已获取且为测试版构建） */
  ready: boolean
}

export function useWatermarkData() {
  const sdkStore = useSdkStore()
  const screenHash = ref('')
  const timeLabel = ref('')

  /** 计算屏印哈希（按当前小时分桶，同一小时稳定） */
  function refreshScreenHash() {
    const now = new Date()
    // 屏印哈希按小时分桶：同一设备同一小时的截图哈希一致，便于追溯
    const hourBucket = `${now.getFullYear()}-${now.getMonth() + 1}-${now.getDate()}-${now.getHours()}`
    const devicePart = sdkStore.deviceId || 'unknown'
    const versionPart = getVersionInfo().raw
    screenHash.value = hashString(`${devicePart}|${versionPart}|${hourBucket}`)
    // 时间标签精确到分钟（用于水印文字展示，攻击者无法通过分钟定位设备）
    timeLabel.value = `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())} ${pad(now.getHours())}:${pad(now.getMinutes())}`
  }

  function pad(n: number): string {
    return n.toString().padStart(2, '0')
  }

  onMounted(async () => {
    // 设备 ID 由 App.vue 启动时获取，这里取 store 中已有值
    // 若未获取到则触发一次（兜底）
    if (!sdkStore.deviceId) {
      await sdkStore.fetchDeviceId()
    }
    refreshScreenHash()
    // 每 30 秒刷新一次屏印哈希和时间标签
    // 跨小时时哈希会变化，便于服务端定位截图时间窗口
    setInterval(refreshScreenHash, 30_000)
  })

  const data = computed<WatermarkData>(() => {
    const versionInfo = getVersionInfo()
    return {
      deviceId: sdkStore.deviceId ? stripMcsdkPrefix(sdkStore.deviceId) : '',
      version: versionInfo.raw,
      channel: versionInfo.channel,
      buildFingerprint: getBuildFingerprint(),
      screenHash: screenHash.value,
      timeLabel: timeLabel.value,
      ready: !!(sdkStore.deviceId && versionInfo.isPreRelease),
    }
  })

  return data
}
