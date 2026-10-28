import { describe, it, expect } from 'vitest'
import { resolveSettingsTab } from '../settings-tab'

const allowed = ['launch', 'download', 'advanced', 'developer']

describe('resolveSettingsTab', () => {
  it('should return the id when the query matches an allowed tab', () => {
    expect(resolveSettingsTab('advanced', allowed)).toBe('advanced')
  })

  it('should return null when the tab does not exist', () => {
    expect(resolveSettingsTab('nope', allowed)).toBeNull()
  })

  it('should return null when the query is missing or not a string', () => {
    expect(resolveSettingsTab(undefined, allowed)).toBeNull()
    expect(resolveSettingsTab('', allowed)).toBeNull()
    expect(resolveSettingsTab(['advanced', 'download'], allowed)).toBeNull()
    expect(resolveSettingsTab(42, allowed)).toBeNull()
  })

  it('should trim surrounding spaces when the query has them', () => {
    expect(resolveSettingsTab(' advanced ', allowed)).toBe('advanced')
  })

  it('should be case sensitive so a wrong-case tab falls back to the current one', () => {
    expect(resolveSettingsTab('Advanced', allowed)).toBeNull()
  })
})
