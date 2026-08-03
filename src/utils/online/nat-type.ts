/**
 * NAT 类型检测工具 + Tooltip 提示文案
 *
 * 基于 WebRTC ICE candidates 的 srflx/host 类型组合判断 NAT 类型，
 * 探测逻辑见 detect.ts，展示文案/颜色映射见 format.ts。
 */
export * from './detect'
export * from './format'