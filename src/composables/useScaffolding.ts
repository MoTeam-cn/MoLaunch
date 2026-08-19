/**
 * Scaffolding 联机中心 composable
 *
 * 封装 scaffolding_host_start / scaffolding_host_stop / scaffolding_client_probe IPC：
 * - 房主：hostStart 一站式启动（探测 MC 端口 → 联机中心 → easytier）
 * - 房客：probe 加入网络后探测房主 MC 服务地址
 * 探测结果写入 online store 的 easytier 切片（mcIp/mcPort），供面板统一消费。
 */
import { ref } from 'vue'
import {
  scaffoldingClientPoll,
  scaffoldingClientProbe,
  scaffoldingHostSetMcPort,
  scaffoldingHostStart,
  scaffoldingHostStop,
} from '@/utils/api/online-manager/easytier'
import { useOnlineStore } from '@/stores/online'
import { useEasyTierInstall } from './useEasyTierInstall'
import { EASYTIER_HOST_VIRTUAL_IP } from '@/types/online'

/** Scaffolding 联机中心 composable */
export function useScaffolding() {
  const store = useOnlineStore()
  const install = useEasyTierInstall()
  const starting = ref(false)
  const stopping = ref(false)
  const probing = ref(false)
  const error = ref('')

  /** 房主一站式启动联机中心 */
  async function hostStart(
    roomCode: string,
    port?: number,
  ): Promise<{ ok: boolean; error?: string }> {
    starting.value = true
    error.value = ''
    try {
      // 前置依赖：easytier 内核未安装时自动下载（进度弹窗由 EasyTierInstallModal 展示）
      const ready = await install.ensureInstalled()
      if (!ready.ok) {
        error.value = ready.error ?? 'easytier 内核安装失败'
        return { ok: false, error: error.value }
      }
      const res = await scaffoldingHostStart({ roomCode, mcPort: port })
      store.setEasyTierRuntime({
        mcIp: EASYTIER_HOST_VIRTUAL_IP,
        centerPort: res.centerPort,
        // 未探测到端口时（先开房后开局域网）暂不写入，由后台监视循环自动补回
        ...(res.mcPort != null ? { mcPort: res.mcPort } : {}),
      })
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
      store.resetEasyTierRuntime()
    } finally {
      stopping.value = false
    }
  }

  /** 房主手动指定 MC 端口（最高权重，自动探测不再覆盖；null 恢复自动模式） */
  async function setMcPort(port: number | null): Promise<void> {
    await scaffoldingHostSetMcPort({ mcPort: port })
    if (port != null) {
      store.setEasyTierRuntime({ mcPort: port })
    }
  }

  /** 房客解析房间码 → 加入网络 → 探测房主 MC 服务 */
  async function probe(
    roomCode: string,
  ): Promise<{ ok: boolean; mcIp?: string; mcPort?: number; error?: string }> {
    probing.value = true
    error.value = ''
    try {
      // 前置依赖：easytier 内核未安装时自动下载（进度弹窗由 EasyTierInstallModal 展示）
      const ready = await install.ensureInstalled()
      if (!ready.ok) {
        error.value = ready.error ?? 'easytier 内核安装失败'
        return { ok: false, error: error.value }
      }
      const res = await scaffoldingClientProbe({ roomCode })
      store.setEasyTierRuntime({ mcIp: res.mcIp, mcPort: res.mcPort })
      return { ok: true, mcIp: res.mcIp, mcPort: res.mcPort }
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      return { ok: false, error: error.value }
    } finally {
      probing.value = false
    }
  }

  /** 房客周期轮询房主 MC 端口（轻量，需已加入网络；不写入 store，由调用方决定） */
  async function poll(
    roomCode: string,
  ): Promise<{ ok: boolean; mcIp?: string; mcPort?: number; error?: string }> {
    try {
      const res = await scaffoldingClientPoll({ roomCode })
      return { ok: true, mcIp: res.mcIp, mcPort: res.mcPort }
    } catch (e) {
      return { ok: false, error: e instanceof Error ? e.message : String(e) }
    }
  }

  return {
    starting,
    stopping,
    probing,
    error,
    hostStart,
    hostStop,
    setMcPort,
    probe,
    poll,
  }
}
