/**
 * 关于页面数据 API
 *
 * 从 system_manager（get_about_data）获取特别鸣谢/技术栈/许可声明，
 * 数据源为 src-tauri/resources/about/ 下的 markdown 表格 txt（后端 markdown_table 解析）。
 */

import { SYSTEM_ACTIONS, systemManager } from './system-manager'

/** 作者信息 */
export interface Author {
  /** 作者姓名 */
  name: string
  /** 作者头像文件名（位于 src/assets/AboutIcon/），undefined 表示无头像 */
  avatar?: string
}

/** 特别鸣谢项 */
export interface AcknowledgementItem {
  /** 项目名称 */
  name: string
  /** 官网地址 */
  home: string
  /** 简介 */
  desc: string
  /** logo 资源文件名（位于 src/assets/AboutIcon/） */
  logo: string
  /** 作者列表（可能为空数组） */
  authors: Author[]
}

/** 技术栈依赖项（前端运行时 / 前端开发工具链 / 后端依赖 共用） */
export interface DependencyItem {
  /** 依赖名称 */
  name: string
  /** 版本号 */
  version: string
  /** 官网/仓库地址 */
  url: string
  /** 简介 */
  desc: string
}

/** 许可与版权声明项 */
export interface LicenseItem {
  /** 依赖名称 */
  name: string
  /** 版权声明 */
  copyright: string
  /** 许可类型 */
  license: string
  /** 来源网站 */
  sourceUrl: string
  /** 许可文档地址 */
  licenseUrl: string
}

/** 关于页面完整数据 */
export interface AboutData {
  /** 特别鸣谢列表 */
  acknowledgements: AcknowledgementItem[]
  /** 前端运行时依赖 */
  frontendDeps: DependencyItem[]
  /** 前端开发工具链 */
  frontendDevDeps: DependencyItem[]
  /** 后端依赖 */
  backendDeps: DependencyItem[]
  /** 许可与版权声明列表 */
  licenses: LicenseItem[]
}

/**
 * 拉取关于页面所需的全部数据
 *
 * 数据由后端从 `resources/about/` 下嵌入的 markdown 表格 txt 文件解析得到，
 * 一次 IPC 调用返回所有内容，避免多次往返。
 */
export async function getAboutData(): Promise<AboutData> {
  return systemManager<AboutData>(SYSTEM_ACTIONS.GET_ABOUT_DATA)
}

/**
 * 拉取项目许可协议全文
 *
 * 协议文本由 build.rs 从项目根目录 LICENSE 自动同步副本后嵌入后端二进制，
 * 经 IPC 返回，供「设置 - 更多 - 许可协议」页签展示。
 */
export async function getProjectLicense(): Promise<string> {
  return systemManager<string>(SYSTEM_ACTIONS.GET_PROJECT_LICENSE)
}
