/**
 * Scaffolding 联机中心 composable
 *
 * 封装 scaffolding_host_start / scaffolding_host_stop / scaffolding_client_probe IPC：
 * - 房主：hostStart 一站式启动（探测 MC 端口 → 联机中心 → easytier）
 * - 房客：probe 加入网络后探测房主 MC 服务地址
 */
import { ref } from 'vue'
import {
  scaffoldingClientProbe,
  scaffoldingHostStart,
  scaffoldingHostStop,
} from '@/utils/api/online-manager/easytier'
import { EASYTIER_HOST_VIRTUAL_IP } from '@/types/online'

/** Scaffolding 联机中心 composable */
export function useScaffolding() {
  const starting = ref(false)
  const stopping = ref(false)
  const probing = ref(false)
  const error = ref('')
  /** 房主虚拟 IP（MC 连接目标） */
  const mcIp = ref('')
  /** 房主 MC 局域网端口 */
  const mcPort = ref(0)
  /** 联机中心实际监听端口（房主） */
  const centerPort = ref(0)
  /** 联机中心 hostname（房主） */
  const hostname = ref('')

  /** 房主一站式启动联机中心 */
  async function hostStart(
    roomCode: string,
    port?: number,
  ): Promise<{ ok: boolean; error?: string }> {
    starting.value = true
    error.value = ''
    try {
      const res = await scaffoldingHostStart({ roomCode, mcPort: port })
      mcIp.value = EASYTIER_HOST_VIRTUAL_IP
      mcPort.value = res.mcPort
      centerPort.value = res.centerPort
      hostname.value = res.hostname
      return { ok: true }
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      return { ok: false, error: error.value }
    } finally {
      starting.value = false
    }
  }

  /** 房主停止联机中心与 easytier */
  async function hostStop(): Promise<void> {
    stopping.value = true
    try {
      await scaffoldingHostStop()
    } finally {
      stopping.value = false
      mcIp.value = ''
      mcPort.value = 0
      centerPort.value = 0
      hostname.value = ''
    }
  }

  /** 房客解析房间码 → 加入网络 → 探测房主 MC 服务 */
  async function probe(
    roomCode: string,
  ): Promise<{ ok: boolean; mcIp?: string; mcPort?: number; error?: string }> {
    probing.value = true
    error.value = ''
    try {
      const res = await scaffoldingClientProbe({ roomCode })
      mcIp.value = res.mcIp
      mcPort.value = res.mcPort
      return { ok: true, mcIp: res.mcIp, mcPort: res.mcPort }
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      return { ok: false, error: error.value }
    } finally {
      probing.value = false
    }
  }

  return {
    starting,
    stopping,
    probing,
    error,
    mcIp,
    mcPort,
    centerPort,
    hostname,
    hostStart,
    hostStop,
    probe,
  }
}
