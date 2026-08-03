/**
 * 账号卡片列表 composable（从 AccountSelector.vue 抽离）
 *
 * 封装卡片列表构建（顺序稳定）、当前索引管理 + 边界检查、切换/删除/登出账号与首次加载拉取。
 * 调用方仅需解构返回值即可在模板中使用。
 */
import { ref, computed, watch, onMounted } from 'vue'
import { useAuthStore } from '@/stores/auth'
import { toastSuccess, toastError, toastWarning } from '@/utils/toast'
import type { AccountCardData } from '@/components/home/account-selector/types'

export function useAccountCards() {
  const authStore = useAuthStore()

  /** 当前显示的卡片索引 */
  const currentIndex = ref(0)

  /**
   * 账号卡片列表（微软账号 + 离线账号 + authlib 账号，顺序稳定）
   *
   * 关键：cards 的顺序不随 currentUser 变化而重排，
   * 当前账号通过 isActive=true 标记，这样切换账号时 currentIndex 指向稳定不变。
   */
  const cards = computed<AccountCardData[]>(() => {
    const list: AccountCardData[] = []
    const currentUuid = authStore.currentUser?.uuid

    // 微软账号
    for (const acc of authStore.msAccounts) {
      list.push({
        uuid: acc.uuid,
        username: acc.username,
        loginType: '正版',
        isExpired: acc.is_expired,
        refreshing: acc.refreshing,
        isActive: acc.uuid === currentUuid,
      })
    }
    // 离线账号
    for (const acc of authStore.offlineAccounts) {
      list.push({
        uuid: acc.uuid,
        username: acc.username,
        loginType: '离线',
        isActive: acc.uuid === currentUuid,
      })
    }
    // authlib 外置登录账号
    for (const acc of authStore.authlibAccounts) {
      list.push({
        uuid: acc.uuid,
        username: acc.player_name,
        loginType: '外置',
        isActive: acc.uuid === currentUuid,
        serverName: acc.server_name,
        serverUrl: acc.server_url,
      })
    }
    // 如果当前账号不在任何列表里（理论上不应发生），追加到末尾
    if (authStore.currentUser && !list.some(c => c.uuid === currentUuid)) {
      const loginType = authStore.currentUser.login_type
      list.push({
        uuid: authStore.currentUser.uuid,
        username: authStore.currentUser.name,
        loginType: loginType === 'Microsoft' ? '正版' : loginType === 'AuthlibInjector' ? '外置' : '离线',
        isActive: true,
      })
    }
    return list
  })

  /** 是否有"添加账号"卡片（末尾） */
  const hasAddCard = computed(() => cards.value.length > 0)
  /** 总卡片数（含添加卡片） */
  const totalCards = computed(() => cards.value.length + (hasAddCard.value ? 1 : 0))

  const isLoggedIn = computed(() => authStore.isLoggedIn)
  const currentUsername = computed(() => authStore.currentUser?.name ?? '')
  const currentLoginType = computed(() => {
    if (!authStore.currentUser) return ''
    const t = authStore.currentUser.login_type
    if (t === 'Microsoft') return '正版账号'
    if (t === 'AuthlibInjector') return '外置账号'
    return '离线账号'
  })

  /**
   * 确保 currentIndex 不越界，并定位到当前活跃账号。
   *
   * 首次加载或账号列表变化时，把 currentIndex 移到 active 卡片。
   * 切换账号时 cards 顺序稳定，currentIndex 不变。
   *
   * 关键：当 currentIndex 指向"添加账号"卡片（末尾）时，currentCard 是 undefined，
   * 不能因此误判为"未指向 active 卡片"而强制重置——用户主动切到"添加账号"
   * 卡片后应停留在那里，等点击按钮跳转登录页。
   */
  watch(cards, (newCards) => {
    const total = newCards.length + (hasAddCard.value ? 1 : 0)
    if (currentIndex.value >= total) {
      currentIndex.value = Math.max(0, total - 1)
    }
    // 用户主动停在"添加账号"卡片时，不要因账号列表变化把它拉回 active 卡片
    if (currentIndex.value === newCards.length) return
    // 如果当前索引不是 active 卡片，且存在 active 卡片，移过去
    const currentCard = newCards[currentIndex.value]
    const activeIndex = newCards.findIndex(c => c.isActive)
    if (activeIndex >= 0 && !currentCard?.isActive) {
      currentIndex.value = activeIndex
    }
  })

  /** 正在切换账号的锁，防止快速滑动时并发请求导致后端报错 */
  const switching = ref(false)

  /** 切换到指定索引（带边界检查）
   *
   * 视觉索引立即更新（保证拖动/滚轮流畅），账号切换异步进行不受 switching 锁阻塞。
   * switching 锁仅在 switchAccount 内部防止并发请求。
   */
  function switchTo(index: number) {
    if (index < 0 || index >= totalCards.value) return
    // 添加账号卡片（末尾），无需切换账号
    if (index === cards.value.length) {
      currentIndex.value = index
      return
    }
    const card = cards.value[index]
    if (!card) return
    // 先更新视觉索引
    currentIndex.value = index
    // 异步切换账号（switchAccount 内部有 switching 锁防并发）
    if (!card.isActive) {
      switchAccount(card.uuid, card.loginType)
    }
  }

  function prev() { if (currentIndex.value > 0) switchTo(currentIndex.value - 1) }
  function next() { if (currentIndex.value < totalCards.value - 1) switchTo(currentIndex.value + 1) }

  async function switchAccount(targetUuid: string, loginType: string) {
    if (authStore.currentUser?.uuid === targetUuid) return
    if (switching.value) return  // 正在切换中，忽略
    switching.value = true
    try {
      if (loginType === '正版') {
        await authStore.switchMsAccount(targetUuid)
      } else if (loginType === '外置') {
        // authlib 账号需要 server_url + uuid 双键定位
        const card = cards.value.find(c => c.uuid === targetUuid && c.loginType === '外置')
        if (card?.serverUrl) {
          await authStore.switchAuthlibAccount(card.serverUrl, targetUuid)
        } else {
          toastWarning('未找到该外置账号的服务器信息')
        }
      } else {
        await authStore.switchOfflineAccount(targetUuid)
      }
      // 切换账号不改变皮肤数据，无需 bumpSkinVersion（皮肤变更由 SkinManager 负责）
      toastSuccess('已切换账号')
    } catch (e) {
      toastError('切换账号失败：' + String(e))
      // 失败时回滚 currentIndex 到实际当前账号
      const activeIndex = cards.value.findIndex(c => c.isActive)
      if (activeIndex >= 0) currentIndex.value = activeIndex
    } finally {
      switching.value = false
    }
  }

  async function removeAccount(targetUuid: string, loginType: string, event: Event) {
    event.stopPropagation()
    try {
      if (loginType === '正版') {
        await authStore.removeMsAccount(targetUuid)
      } else if (loginType === '外置') {
        const card = cards.value.find(c => c.uuid === targetUuid && c.loginType === '外置')
        if (card?.serverUrl) {
          await authStore.removeAuthlibAccount(card.serverUrl, targetUuid)
        } else {
          toastWarning('未找到该外置账号的服务器信息')
        }
      } else {
        await authStore.removeOfflineAccount(targetUuid)
      }
      // 删除后调整索引
      if (currentIndex.value > 0) currentIndex.value--
      toastSuccess('账号已删除')
    }
    catch (e) { toastError('删除账号失败：' + String(e)) }
  }

  async function logout() {
    try {
      await authStore.logout()
      toastSuccess('已退出登录')
    } catch (e) {
      toastError('退出登录失败：' + String(e))
    }
  }

  onMounted(() => {
    authStore.loadMsAccounts()
    authStore.loadOfflineAccounts()
    // authlib 账号也需要加载，否则外置登录账号不显示在卡片栏
    authStore.loadAuthlibAccounts()
  })

  return {
    cards,
    currentIndex,
    hasAddCard,
    totalCards,
    isLoggedIn,
    currentUsername,
    currentLoginType,
    switching,
    switchTo,
    prev,
    next,
    switchAccount,
    removeAccount,
    logout,
  }
}
