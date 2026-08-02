/**
 * 创建房间表单 composable
 *
 * 从 components/online/CreateRoomForm.vue 抽离（保持主文件 ≤ 300 行约束）：
 * - 表单状态（基础字段 / 整合包元数据 / 白名单）
 * - 已安装版本列表加载 + 选中后异步解析 MC 版本 / 加载器信息
 * - 提交校验（版本 / 端口 / 人数）与创建房间流程（stun → create 两步）
 * - 高级设置状态徽章（整合包关联 / 白名单启用联动）
 *
 * 返回全部模板所需状态与动作，主文件仅保留模板 + 组装。
 */
import { ref, computed, onMounted } from 'vue'
import { useOnlineStore } from '@/stores/online'
import {
  listInstalledVersionsWithType,
  getVersionLoaderInfo,
  type InstalledVersionInfo,
} from '@/utils/api/version'
import { getVersionGameVersion } from '@/utils/api/personalization'
import type { ModpackMeta } from '@/types/online'
import { toastError, toastWarning } from '@/utils/toast'

/** 创建房间表单 composable（表单状态 / 校验 / 提交 / 进阶配置） */
export function useCreateRoomForm() {
  const store = useOnlineStore()

  /** 创建房间表单 */
  const createForm = ref({
    maxPlayers: 4,
    password: '',
    mcVersion: '',         // 纯 MC 版本号（如 1.20.1），由 getVersionGameVersion 解析
    mcPort: 25565,
    selectedVersionId: '', // 选中的 version_id（Select 回显用）
    hostLoader: '',        // forge/fabric/neoforge/.../release
    hostLoaderVersion: '', // 如 47.3.0
    roomType: 'private' as 'private' | 'lobby', // 联机大厅阶段 2：private 仅房间码 / lobby 加入大厅
  })

  /** 整合包元数据（联机大厅阶段 3，undefined=纯原版房间） */
  const modpackMeta = ref<ModpackMeta | undefined>()
  /** 整合包勾选状态（联机大厅阶段 3，即使版本无元数据也反映用户勾选意图，用于徽章联动） */
  const modpackEnabled = ref(false)

  const publicRoomHint = computed(() =>
    createForm.value.roomType === 'lobby'
      ? '房间将加入联机大厅，其他玩家可在「联机大厅」中检索并加入'
      : '仅凭房间码加入，不会出现在大厅列表中',
  )

  /** 已安装版本列表（用于 MC 版本下拉选择） */
  const installedVersions = ref<InstalledVersionInfo[]>([])
  const versionOptions = computed(() =>
    installedVersions.value.map((v) => ({ label: v.id, value: v.id })),
  )
  const versionsLoading = ref(false)
  /** 版本信息解析中（避免重复点击/提交） */
  const versionResolving = ref(false)

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
   * 选择已安装版本后异步解析三字段（联机大厅阶段 1）
   *
   * - `mcVersion` ← getVersionGameVersion（inheritsFrom / --fml.mcVersion / URL 正则 / jar / id）
   * - `hostLoader` + `hostLoaderVersion` ← getVersionLoaderInfo（setup.ini 的 Type + XxxVersion）
   *
   * 任一调用失败时兜底：mcVersion = version_id，hostLoader = 'release'，hostLoaderVersion = ''
   */
  async function onVersionSelect(value: string | number) {
    const versionId = String(value)
    // 清空操作（allow-clear 触发 emit ''）：重置所有版本相关字段，不再调用解析接口
    if (!versionId) {
      createForm.value.selectedVersionId = ''
      createForm.value.mcVersion = ''
      createForm.value.hostLoader = ''
      createForm.value.hostLoaderVersion = ''
      return
    }
    createForm.value.selectedVersionId = versionId
    // 立即清空旧值，避免异步返回前显示脏数据
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

  /** 最大人数提示：mesh 拓扑 5+ 人带宽压力陡增，超限显示 error */
  const maxPlayersHint = computed(() => {
    const v = createForm.value.maxPlayers
    if (v < 2) return '至少需要 2 人（房主 + 1 参与者）'
    if (v > 5) return 'mesh 模式不建议超过 5 人，请使用专业服务器'
    return 'mesh 模式建议 2-5 人，超过请使用专业服务器'
  })
  const maxPlayersHintType = computed<'default' | 'error' | 'success'>(() => {
    const v = createForm.value.maxPlayers
    if (v < 2 || v > 5) return 'error'
    return 'default'
  })

  /** 白名单表单状态 */
  const whitelistForm = ref({ enabled: false, deviceIds: [] as string[] })

  /** 高级设置状态徽章：同时反映整合包勾选与白名单启用状态 */
  const advancedBadge = computed(() => {
    const parts: string[] = []
    if (modpackEnabled.value) parts.push('已关联整合包')
    if (whitelistForm.value.enabled) parts.push('白名单已启用')
    return parts.length > 0 ? parts.join(' · ') : '未启用'
  })
  const advancedBadgeActive = computed(() =>
    modpackEnabled.value || whitelistForm.value.enabled,
  )

  /** 整合包勾选状态变化回调（联机大厅阶段 3） */
  function onModpackEnabledChange(enabled: boolean) {
    modpackEnabled.value = enabled
  }

  /** 创建房间步骤指示器（mesh 拓扑两步：stun → create） */
  const createSteps = [{ key: 'stun' as const, label: '获取 STUN 服务器' }, { key: 'create' as const, label: '创建房间' }]
  const stepOrder = ['stun', 'create'] as const
  const currentStepIndex = computed(() => store.roomCreateStep ? stepOrder.indexOf(store.roomCreateStep) : -1)

  /** 房主创建房间（mesh 拓扑：不生成本地 Offer，参与者加入后再 per-participant 生成） */
  async function handleCreateRoom() {
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
    if (createForm.value.maxPlayers < 2 || createForm.value.maxPlayers > 5) {
      toastError('人数无效：mesh 模式最大人数范围为 2-5')
      return
    }

    try {
      store.roomCreateStep = 'stun'
      const stun = await store.fetchStunServers()

      store.roomCreateStep = 'create'
      await store.hostCreateRoom(
        '',
        [],
        createForm.value.maxPlayers,
        createForm.value.password,
        createForm.value.mcVersion,
        createForm.value.mcPort,
        stun,
        whitelistForm.value.enabled,
        whitelistForm.value.deviceIds,
        createForm.value.hostLoader,
        createForm.value.hostLoaderVersion,
        createForm.value.roomType,
        // 公开房间携带大厅 ID（当前固定 global，阶段 5 做大厅选择器后扩展）
        createForm.value.roomType === 'lobby' ? 'global' : undefined,
        // 联机大厅阶段 3：整合包元数据（undefined=纯原版房间）
        modpackMeta.value,
      )
    } catch (e) {
      toastError(`创建房间失败：${e instanceof Error ? e.message : String(e)}`)
    } finally {
      store.roomCreateStep = null
    }
  }

  return {
    store,
    createForm,
    modpackMeta,
    onModpackEnabledChange,
    publicRoomHint,
    versionOptions,
    versionsLoading,
    onVersionSelect,
    maxPlayersHint,
    maxPlayersHintType,
    whitelistForm,
    advancedBadge,
    advancedBadgeActive,
    createSteps,
    currentStepIndex,
    handleCreateRoom,
  }
}
