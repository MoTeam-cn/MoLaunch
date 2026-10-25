import { describe, expect, it } from 'vitest'
import { ModLoaderFlags } from '@/types/community'
import {
  deriveInstanceLoader,
  loaderNames,
  precheckModInstall,
  supportsGameVersion,
  type InstalledModRef,
  type PrecheckInput,
} from '../mod-precheck'

function mod(partial: Partial<InstalledModRef>): InstalledModRef {
  return {
    file_name: 'x.jar',
    is_enabled: true,
    slug: '',
    loader_type: 'forge',
    ...partial,
  }
}

function input(partial: Partial<PrecheckInput>): PrecheckInput {
  return {
    target: { file_name: 'new.jar', mod_loaders: ModLoaderFlags.Forge, game_versions: ['1.20.1'] },
    installed: [],
    instanceGameVersion: '1.20.1',
    ...partial,
  }
}

describe('mod-precheck', () => {
  it('should block when loader does not match the instance', () => {
    const issues = precheckModInstall(input({
      target: { file_name: 'fabric.jar', mod_loaders: ModLoaderFlags.Fabric, game_versions: ['1.20.1'] },
      installed: [mod({ file_name: 'jei.jar', loader_type: 'forge', slug: 'jei' })],
    }))
    expect(issues.map((issue) => issue.code)).toContain('loader_mismatch')
    expect(issues[0].level).toBe('blocker')
  })

  it('should not block when target supports multiple loaders including the instance one', () => {
    const issues = precheckModInstall(input({
      target: { file_name: 'multi.jar', mod_loaders: ModLoaderFlags.Forge | ModLoaderFlags.Fabric, game_versions: ['1.20.1'] },
      installed: [mod({ loader_type: 'fabric', slug: 'fabric-api' })],
    }))
    expect(issues.some((issue) => issue.code === 'loader_mismatch')).toBe(false)
  })

  it('should block when the target does not support the instance game version', () => {
    const issues = precheckModInstall(input({
      target: { file_name: 'old.jar', mod_loaders: ModLoaderFlags.Forge, game_versions: ['1.19.2'] },
      instanceGameVersion: '1.20.1',
    }))
    const issue = issues.find((item) => item.code === 'game_version_mismatch')
    expect(issue?.level).toBe('blocker')
    expect(issue?.detail).toContain('1.20.1')
  })

  it('should accept a coarse version like 1.20 for an instance of 1.20.1', () => {
    expect(supportsGameVersion(['1.20'], '1.20.1')).toBe(true)
    expect(supportsGameVersion(['1.20.1'], '1.20')).toBe(true)
    expect(supportsGameVersion(['1.19.2'], '1.20.1')).toBe(false)
    expect(supportsGameVersion(['Forge', '1.20.1'], '1.20.1')).toBe(true)
  })

  it('should warn when the same modid is already installed and enabled', () => {
    const issues = precheckModInstall(input({
      projectSlug: 'jei',
      installed: [mod({ file_name: 'jei-old.jar', slug: 'JEI', loader_type: 'forge' })],
    }))
    const issue = issues.find((item) => item.code === 'duplicate_modid')
    expect(issue?.level).toBe('warning')
    expect(issue?.detail).toContain('jei-old.jar')
  })

  it('should not warn about duplicate when the installed copy is disabled', () => {
    const issues = precheckModInstall(input({
      projectSlug: 'jei',
      installed: [mod({ file_name: 'jei.jar.disabled', slug: 'jei', is_enabled: false })],
    }))
    expect(issues.some((item) => item.code === 'duplicate_modid')).toBe(false)
  })

  it('should warn when the target file name already exists', () => {
    const issues = precheckModInstall(input({
      target: { file_name: 'create.jar', mod_loaders: ModLoaderFlags.Forge, game_versions: ['1.20.1'] },
      installed: [mod({ file_name: 'create.jar', slug: 'create' })],
    }))
    expect(issues.some((item) => item.code === 'same_file_exists')).toBe(true)
  })

  it('should warn when the instance mixes forge family and fabric family mods', () => {
    const issues = precheckModInstall(input({
      installed: [
        mod({ file_name: 'a.jar', loader_type: 'forge', slug: 'a' }),
        mod({ file_name: 'b.jar', loader_type: 'fabric', slug: 'b' }),
      ],
    }))
    expect(issues.some((item) => item.code === 'mixed_loaders')).toBe(true)
  })

  it('should warn when OptiFine is installed and the target is fabric family', () => {
    const issues = precheckModInstall(input({
      target: { file_name: 'sodium.jar', mod_loaders: ModLoaderFlags.Fabric, game_versions: ['1.20.1'] },
      installed: [
        mod({ file_name: 'OptiFine_1.20.1_HD_U_I6.jar', loader_type: 'fabric', slug: 'optifine' }),
      ],
    }))
    expect(issues.some((item) => item.code === 'optifine_conflict')).toBe(true)
  })

  it('should derive the instance loader from the majority of installed mods', () => {
    expect(deriveInstanceLoader([
      mod({ loader_type: 'fabric' }),
      mod({ loader_type: 'fabric' }),
      mod({ loader_type: 'forge' }),
    ])).toBe(ModLoaderFlags.Fabric)
    expect(deriveInstanceLoader([])).toBe(ModLoaderFlags.None)
  })

  it('should render loader names for a bitmask', () => {
    expect(loaderNames(ModLoaderFlags.Forge | ModLoaderFlags.NeoForge)).toEqual(['Forge', 'NeoForge'])
    expect(loaderNames(0)).toEqual([])
  })
})
