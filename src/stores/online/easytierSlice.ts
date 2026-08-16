/**
 * 联机 store easytier 切片（easytier + Scaffolding 运行时状态，供全局访问/重启恢复）
 */
import { ref } from 'vue'

/** easytier + Scaffolding 运行时状态 */
export interface EasyTierRuntime {
  /** 是否已加入虚拟网络 */
  joined: boolean
  /** 虚拟网络名（scaffolding-mc-{N 段}） */
  networkName: string
  /** 虚拟网络密钥（房间码 S 段） */
  networkSecret: string
  /** 本机虚拟 IP（房主固定 10.144.144.1；房客 DHCP 未回显时为空） */
  virtualIp: string
  /** easytier rpc-portal 地址 */
  rpcPortal: string
  /** easytier-core 子进程 PID */
  pid?: number
  /** easytier-core 版本号（后端 --version 查询，失败时为空串） */
  version: string
  /** 房主 MC 进服地址（房客 probe 后写入；no-tun 下为本地 port-forward 地址 127.0.0.1:local_port） */
  mcIp: string
  /** 房主 MC 局域网端口 */
  mcPort: number
  /** 联机中心实际监听端口（房主） */
  centerPort: number
}

/** 创建 easytier 运行时切片 */
export function useOnlineEasyTierSlice() {
  const easytierRuntime = ref<EasyTierRuntime>(emptyRuntime())

  function setEasyTierRuntime(patch: Partial<EasyTierRuntime>): void {
    easytierRuntime.value = { ...easytierRuntime.value, ...patch }
  }

  function resetEasyTierRuntime(): void {
    easytierRuntime.value = emptyRuntime()
  }

  return { easytierRuntime, setEasyTierRuntime, resetEasyTierRuntime }
}

function emptyRuntime(): EasyTierRuntime {
  return {
    joined: false,
    networkName: '',
    networkSecret: '',
    virtualIp: '',
    rpcPortal: '',
    pid: undefined,
    version: '',
    mcIp: '',
    mcPort: 0,
    centerPort: 0,
  }
}
