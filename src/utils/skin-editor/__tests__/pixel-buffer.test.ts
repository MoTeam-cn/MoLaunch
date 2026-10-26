import { describe, expect, it } from 'vitest'
import { PixelBuffer } from '../pixel-buffer'
import { detectSkinLayout, findPartAt, getSkinParts, rectContains } from '../layout'

const RED = { r: 255, g: 0, b: 0, a: 255 }
const BLUE = { r: 0, g: 0, b: 255, a: 255 }

describe('PixelBuffer', () => {
  it('should read back written pixel when setPixel called', () => {
    const buffer = PixelBuffer.blank(64)
    buffer.beginStroke()
    buffer.setPixel(3, 4, RED)
    buffer.endStroke()
    expect(buffer.getPixel(3, 4)).toEqual(RED)
  })

  it('should ignore writes when coordinates are out of range', () => {
    const buffer = PixelBuffer.blank(64)
    expect(buffer.setPixel(-1, 0, RED)).toBe(false)
    expect(buffer.setPixel(64, 0, RED)).toBe(false)
  })

  it('should restore previous pixels when undo called', () => {
    const buffer = PixelBuffer.blank(64)
    buffer.beginStroke()
    buffer.setPixel(1, 1, RED)
    buffer.endStroke()
    expect(buffer.undo()).toBe(true)
    expect(buffer.getPixel(1, 1).a).toBe(0)
  })

  it('should reapply pixels when redo called after undo', () => {
    const buffer = PixelBuffer.blank(64)
    buffer.beginStroke()
    buffer.setPixel(1, 1, RED)
    buffer.endStroke()
    buffer.undo()
    expect(buffer.redo()).toBe(true)
    expect(buffer.getPixel(1, 1)).toEqual(RED)
  })

  it('should not push undo entry when stroke changes nothing', () => {
    const buffer = PixelBuffer.blank(64)
    buffer.beginStroke()
    expect(buffer.endStroke()).toBe(false)
    expect(buffer.canUndo()).toBe(false)
  })

  it('should treat repeated same-color writes as one undo step', () => {
    const buffer = PixelBuffer.blank(64)
    buffer.beginStroke()
    buffer.setPixel(2, 2, RED)
    buffer.setPixel(2, 2, RED)
    buffer.endStroke()
    buffer.undo()
    expect(buffer.getPixel(2, 2).a).toBe(0)
    expect(buffer.canUndo()).toBe(false)
  })

  it('should fill connected region only when floodFill called', () => {
    const buffer = PixelBuffer.blank(64)
    buffer.beginStroke()
    buffer.line(0, 0, 3, 0, RED)
    buffer.endStroke()
    const filled = buffer.floodFill(0, 1, BLUE)
    expect(filled).toBeGreaterThan(0)
    expect(buffer.getPixel(0, 1)).toEqual(BLUE)
    // 边界线本身不被覆盖
    expect(buffer.getPixel(0, 0)).toEqual(RED)
  })

  it('should draw connected line when line called', () => {
    const buffer = PixelBuffer.blank(64)
    buffer.beginStroke()
    buffer.line(0, 0, 4, 4, RED)
    buffer.endStroke()
    expect(buffer.getPixel(0, 0)).toEqual(RED)
    expect(buffer.getPixel(2, 2)).toEqual(RED)
    expect(buffer.getPixel(4, 4)).toEqual(RED)
  })

  it('should widen stroke when brushSize greater than one', () => {
    const buffer = PixelBuffer.blank(64)
    buffer.beginStroke()
    buffer.paint(5, 5, RED, 3)
    buffer.endStroke()
    expect(buffer.getPixel(4, 4)).toEqual(RED)
    expect(buffer.getPixel(6, 6)).toEqual(RED)
  })

  it('should drop oldest entry when undo stack exceeds limit', () => {
    const buffer = PixelBuffer.blank(64)
    for (let i = 0; i < 60; i++) {
      buffer.beginStroke()
      buffer.setPixel(i, 0, RED)
      buffer.endStroke()
    }
    let undone = 0
    while (buffer.undo()) undone++
    expect(undone).toBe(50)
  })
})

describe('skin layout', () => {
  it('should detect 64x64 as modern layout', () => {
    expect(detectSkinLayout(64, 64)).toBe(64)
  })

  it('should detect 64x32 as legacy layout', () => {
    expect(detectSkinLayout(64, 32)).toBe(32)
  })

  it('should reject unsupported size', () => {
    expect(detectSkinLayout(32, 32)).toBeNull()
  })

  it('should mark left limbs mirrored when legacy layout used', () => {
    const parts = getSkinParts(32)
    expect(parts.find((p) => p.id === 'leftArm')?.mirrored).toBe(true)
    expect(parts.find((p) => p.id === 'head')?.overlay).toBeUndefined()
  })

  it('should give overlay region to every modern part', () => {
    const parts = getSkinParts(64)
    expect(parts.every((p) => p.overlay !== undefined)).toBe(true)
  })

  it('should resolve part by coordinates', () => {
    const parts = getSkinParts(64)
    expect(findPartAt(parts, 8, 8)?.id).toBe('head')
    expect(findPartAt(parts, 20, 20)?.id).toBe('body')
    expect(findPartAt(parts, 63, 63)?.id).toBe('leftArm')
  })

  it('should treat rect as half-open when checking bounds', () => {
    expect(rectContains({ x: 0, y: 0, w: 4, h: 4 }, 3, 3)).toBe(true)
    expect(rectContains({ x: 0, y: 0, w: 4, h: 4 }, 4, 3)).toBe(false)
  })
})
