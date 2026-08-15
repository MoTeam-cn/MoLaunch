/**
 * 创建房间表单 composable（Scaffolding 收敛版）
 *
 * 表单状态、版本列表加载与解析、本地生成 U/xxx 码、登记 + 拉起联机中心一站式流程；
 * 返回全部模板所需状态与动作，主文件仅保留模板与组装。
 */
import { ref, computed, onMounted } from 'vue'
import { useOnlineStore } from '@/stores/online'
import { useRoomHost } from './useRoomHost'
import {
  listInstalledVersionsWithType,
  getVersionLoaderInfo,
  type InstalledVersionInfo,
} from '@/utils/api/version'
import { getVersionGameVersion } from '@/utils/api/personalization'
import { generateScaffoldingCode } from '@/types/online'
import type { ModpackMeta } from '@/types/online'
import { toastError, toastWarning } from '@/utils/toast'

/** 创建流程阶段（UI 步骤指示） */
export type CreateStep = 'idle' | 'code' | 'register' | 'start'

/** 创建房间表单 composable（表单状态 / 校验 / 提交 / 整合包） */
export function useCreateRoomForm() {
  const store = useOnlineStore()
  const roomHost = useRoomHost()

  /** 创建房间表单 */
  const createForm = ref({
    remark: '',
    isPublic: true,
    password: '',
    mcVersion: '',         // 纯 MC 版本号（如 1.20.1），由 getVersionGameVersion 解析
    mcPort: 25565,
    selectedVersionId: '', // 选中的 version_id（Select 回显用）
    hostLoader: '',        // forge/fabric/neoforge/.../release
    hostLoaderVersion: '', // 如 47.3.0
  })

  /** 整合包元数据（undefined=纯原版房间） */
  const modpackMeta = ref<ModpackMeta | undefined>()
  /** 整合包勾选状态（即使版本无元数据也反映用户勾选意图） */
  const modpackEnabled = ref(false)

  const publicRoomHint = computed(() =>
    createForm.value.isPublic
      ? '公开房间将按整合包聚类进入「联机大厅」，其他玩家可检索并加入'
      : '私密房间仅凭房间码加入，不会出现在大厅列表中',
  )

  /** 已安装版本列表（用于 MC 版本下拉选择） */
  const installedVersions = ref<InstalledVersionInfo[]>([])
  const versionOptions = computed(() =>
    installedVersions.value.map((v) => ({ label: v.id, value: v.id })),
  )
  const versionsLoading = ref(false)
  /** 版本信息解析中（避免重复点击/提交） */
  const versionResolving = ref(false)
  /** 创建流程进行中（避免重复提交） */
  const creating = ref(false)
  /** 当前创建阶段（UI 步骤指示） */
  const createStep = ref<CreateStep>('idle')
  const createSteps = [
    { key: 'code', label: '生成房间码' },
    { key: 'register', label: '登记房间' },
    { key: 'start', label: '拉起联机中心' },
  ] as const

  onMounted(async () => {
    versionsLoading.value = true
    try {
      installedVersions.value = await listInstalledVersionsWithType()
    } catch (e) {
      console.error('Failed to load installed versions:', e)
      toastError('加载已安装版本列表失败，请重试')
    } finally {
      versionsLoading.value = false
    }
  })

  /**
   * 选择已安装版本后异步解析三字段
   *
   * - `mcVersion` ← getVersionGameVersion（inheritsFrom / --fml.mcVersion / URL 正则 / jar / id）
   * - `hostLoader` + `hostLoaderVersion` ← getVersionLoaderInfo（setup.ini 的 Type + XxxVersion）
   *
   * 任一调用失败时兜底：mcVersion = version_id，hostLoader = 'release'，hostLoaderVersion = ''
   */
  async function onVersionSelect(value: string | number) {
    const versionId = String(value)
    if (!versionId) {
      createForm.value.selectedVersionId = ''
      createForm.value.mcVersion = ''
      createForm.value.hostLoader = ''
      createForm.value.hostLoaderVersion = ''
      return
    }
    createForm.value.selectedVersionId = versionId
    createForm.value.mcVersion = ''
    createForm.value.hostLoader = ''
    createForm.value.hostLoaderVersion = ''
    versionResolving.value = true
    try {
      const [gameVersion, loaderInfo] = await Promise.all([
        getVersionGameVersion(versionId),
        getVersionLoaderInfo(versionId),
      ])
      createForm.value.mcVersion = gameVersion ?? versionId
      createForm.value.hostLoader = loaderInfo.loaderType
      createForm.value.hostLoaderVersion = loaderInfo.loaderVersion
    } catch (e) {
      console.error('Failed to resolve version info:', e)
      toastWarning('版本信息解析失败，已使用版本 ID 作为兜底，请核对加载器类型')
      createForm.value.mcVersion = versionId
      createForm.value.hostLoader = 'release'
      createForm.value.hostLoaderVersion = ''
    } finally {
      versionResolving.value = false
    }
  }

  /** 整合包勾选状态变化回调 */
  function onModpackEnabledChange(enabled: boolean) {
    modpackEnabled.value = enabled
  }

  /**
   * 房主创建房间（本地生成完整码 → 登记 → 拉起联机中心）
   *
   * 拉起失败时回滚登记并清空本地房间状态。
   */
  async function handleCreateRoom() {
    if (creating.value) return
    if (!createForm.value.selectedVersionId) {
      toastError('请选择 MC 版本：创建房间前需指明房主的 Minecraft 版本')
      return
    }
    if (versionResolving.value) {
      toastError('版本信息解析中，请稍候再试')
      return
    }
    if (!createForm.value.mcVersion) {
      toastError('版本信息解析失败，请重新选择 MC 版本')
      return
    }
    if (createForm.value.mcPort <= 0 || createForm.value.mcPort > 65535) {
      toastError('MC 端口无效：端口范围 1-65535')
      return
    }

    creating.value = true
    try {
      createStep.value = 'code'
      const roomCode = generateScaffoldingCode()

      createStep.value = 'register'
      await store.hostCreateRoom({
        roomCode,
        remark: createForm.value.remark,
        isPublic: createForm.value.isPublic,
        password: createForm.value.password,
        hostMcVersion: createForm.value.mcVersion,
        hostMcPort: createForm.value.mcPort,
        hostLoader: createForm.value.hostLoader,
        hostLoaderVersion: createForm.value.hostLoaderVersion,
        modpack: modpackMeta.value,
      })

      createStep.value = 'start'
      const started = await roomHost.hostStart()
      if (!started.ok) {
        await store.hostCloseRoom()
        throw new Error(started.error || '拉起联机中心失败')
      }
    } catch (e) {
      toastError(`创建房间失败：${e instanceof Error ? e.message : String(e)}`)
    } finally {
      createStep.value = 'idle'
      creating.value = false
    }
  }

  return {
    store,
    createForm,
    creating,
    createSteps,
    createStep,
    modpackMeta,
    modpackEnabled,
    onModpackEnabledChange,
    publicRoomHint,
    versionOptions,
    versionsLoading,
    onVersionSelect,
    handleCreateRoom,
  }
}
