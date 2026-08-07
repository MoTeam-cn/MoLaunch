import {
  BeakerIcon,
  Cog6ToothIcon,
  CubeIcon,
  HomeIcon,
  UserGroupIcon,
  WrenchScrewdriverIcon,
} from '@heroicons/vue/24/outline'

export interface TopNavItem {
  name: string
  path: string
  icon: typeof HomeIcon
  hasDblClick?: boolean
  cloudDependent?: boolean
}

export const topNavItems: TopNavItem[] = [
  { name: '首页', path: '/apps', icon: HomeIcon },
  { name: '下载', path: '/apps/versions', icon: CubeIcon, hasDblClick: true },
  { name: '联机', path: '/apps/online', icon: UserGroupIcon, cloudDependent: true },
  { name: '工具', path: '/apps/tools', icon: WrenchScrewdriverIcon },
  { name: '设置', path: '/apps/settings', icon: Cog6ToothIcon },
]

export const experimentalNavItem: TopNavItem = {
  name: '实验性',
  path: '/apps/experimental',
  icon: BeakerIcon,
}
