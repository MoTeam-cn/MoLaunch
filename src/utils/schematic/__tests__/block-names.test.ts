import { describe, expect, it } from 'vitest'
import { blockDisplayName } from '../block-names'

describe('blockDisplayName', () => {
  it('should return the Chinese name for a known block', () => {
    expect(blockDisplayName('minecraft:jungle_planks')).toBe('丛林木板')
    expect(blockDisplayName('minecraft:stone')).toBe('石头')
  });

  it('should ignore blockstate properties when looking up', () => {
    expect(blockDisplayName('minecraft:jungle_planks[waterlogged=false]')).toBe('丛林木板')
  });

  it('should fall back to the bare id when the block is not listed', () => {
    expect(blockDisplayName('minecraft:not_a_real_block')).toBe('not_a_real_block')
    expect(blockDisplayName('not_a_real_block')).toBe('not_a_real_block')
  });
});
