import { describe, expect, it } from 'vitest'
import { isDirectImageUrl } from '../ui-prefs'

describe('isDirectImageUrl', () => {
  it('should accept http and https when given a remote url', () => {
    expect(isDirectImageUrl('https://cdn.example.com/a.png')).toBe(true)
    expect(isDirectImageUrl('http://example.com/a.jpg')).toBe(true)
  })

  it('should accept data and blob when given an inline or object url', () => {
    expect(isDirectImageUrl('data:image/png;base64,iVBORw0KGgo=')).toBe(true)
    expect(isDirectImageUrl('blob:http://localhost/1234')).toBe(true)
  })

  it('should reject windows absolute path when given a local file', () => {
    expect(isDirectImageUrl('D:\\Pictures\\bg.png')).toBe(false)
  })

  it('should reject posix absolute path when given a local file', () => {
    expect(isDirectImageUrl('/home/user/bg.png')).toBe(false)
  })

  it('should reject empty string when nothing is configured', () => {
    expect(isDirectImageUrl('')).toBe(false)
    expect(isDirectImageUrl('   ')).toBe(false)
  })

  it('should be case insensitive when scheme is uppercase', () => {
    expect(isDirectImageUrl('HTTPS://cdn.example.com/a.png')).toBe(true)
  })
})
