import { describe, expect, it } from 'vitest'
import { clamp, orbitPosition, panTarget } from '../orbit'

describe('orbitPosition', () => {
  it('should place the camera on +z when yaw and pitch are zero', () => {
    const [x, y, z] = orbitPosition([0, 0, 0], 10, 0, 0)
    expect(x).toBeCloseTo(0)
    expect(y).toBeCloseTo(0)
    expect(z).toBeCloseTo(10)
  });

  it('should place the camera overhead at a quarter pitch', () => {
    const [x, y, z] = orbitPosition([0, 0, 0], 10, 0, Math.PI / 2)
    expect(x).toBeCloseTo(0)
    expect(y).toBeCloseTo(10)
    expect(z).toBeCloseTo(0)
  });

  it('should keep the distance to the target', () => {
    const target: [number, number, number] = [12, 34, 56]
    const pos = orbitPosition(target, 25, 1.1, 0.4)
    const distance = Math.hypot(pos[0] - target[0], pos[1] - target[1], pos[2] - target[2])
    expect(distance).toBeCloseTo(25)
  });
});

describe('panTarget', () => {
  it('should move along -x when dragging right at yaw zero', () => {
    // yaw = 0 时相机的右方向是世界 +x，向右拖拽画布等于目标点向左移
    const moved = panTarget([0, 0, 0], 10, 0, 0, 100, 0, 500)
    expect(moved[0]).toBeLessThan(0)
    expect(moved[1]).toBeCloseTo(0)
    expect(moved[2]).toBeCloseTo(0)
  });

  it('should raise the target when dragging down', () => {
    // 抓住画面往下拉，看到的是更上面的内容，目标点随之上移
    const moved = panTarget([0, 0, 0], 10, 0, 0, 0, 100, 500)
    expect(moved[1]).toBeGreaterThan(0)
  });

  it('should scale with distance so the drag feels the same at any zoom', () => {
    const near = panTarget([0, 0, 0], 5, 0, 0, 100, 0, 500)
    const far = panTarget([0, 0, 0], 50, 0, 0, 100, 0, 500)
    expect(Math.abs(far[0])).toBeGreaterThan(Math.abs(near[0]) * 5)
  });
});

describe('clamp', () => {
  it('should keep the value inside the range', () => {
    expect(clamp(5, 0, 10)).toBe(5)
    expect(clamp(-3, 0, 10)).toBe(0)
    expect(clamp(42, 0, 10)).toBe(10)
  });
});
