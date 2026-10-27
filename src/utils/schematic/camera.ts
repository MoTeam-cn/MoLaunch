/**
 * 3D 预览的相机数学
 *
 * 球面坐标转相机位置，以及把屏幕拖拽换算成世界平移——两者都只依赖
 * 目标点、距离和两个角度，抽出来便于单独验证。
 */

export const FOV = 70
export const MIN_RADIUS = 2

export type Vec3 = [number, number, number]

export function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value))
}

/** 相机位置：绕目标点的球面坐标 */
export function orbitPosition(
  target: Vec3,
  radius: number,
  yaw: number,
  pitch: number,
): Vec3 {
  const cosPitch = Math.cos(pitch)
  return [
    target[0] + radius * cosPitch * Math.sin(yaw),
    target[1] + radius * Math.sin(pitch),
    target[2] + radius * cosPitch * Math.cos(yaw),
  ]
}

/** 屏幕拖拽 → 目标点位移（相机基向量在水平/垂直方向上的投影） */
export function panTarget(
  target: Vec3,
  radius: number,
  yaw: number,
  pitch: number,
  dx: number,
  dy: number,
  viewportHeight: number,
): Vec3 {
  const viewHeight = 2 * radius * Math.tan((FOV * Math.PI) / 360)
  const scale = viewHeight / Math.max(viewportHeight, 1)
  const cosYaw = Math.cos(yaw)
  const sinYaw = Math.sin(yaw)
  const sinPitch = Math.sin(pitch)
  const cosPitch = Math.cos(pitch)
  return [
    target[0] - cosYaw * dx * scale + -sinPitch * sinYaw * dy * scale,
    target[1] + cosPitch * dy * scale,
    target[2] + sinYaw * dx * scale + -sinPitch * cosYaw * dy * scale,
  ]
}
