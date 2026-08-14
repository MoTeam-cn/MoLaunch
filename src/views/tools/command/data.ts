/**
 * 指令生成工具 - 常用数据表
 *
 * 纯前端静态数据（Minecraft Java 1.21+ 命名空间 ID）：
 * - 常用附魔表（附魔 ID + 中文名 + 自然最高等级）
 * - 常见可召唤实体表
 */

/** 常用附魔（Java 命名空间 ID，lvl 为自然获取最高等级） */
export interface EnchantmentOption {
  id: string
  name: string
  lvl: number
}

export const ENCHANTMENTS: EnchantmentOption[] = [
  { id: 'sharpness', name: '锋利', lvl: 5 },
  { id: 'smite', name: '亡灵杀手', lvl: 5 },
  { id: 'bane_of_arthropods', name: '节肢杀手', lvl: 5 },
  { id: 'efficiency', name: '效率', lvl: 5 },
  { id: 'fortune', name: '时运', lvl: 3 },
  { id: 'silk_touch', name: '精准采集', lvl: 1 },
  { id: 'unbreaking', name: '耐久', lvl: 3 },
  { id: 'protection', name: '保护', lvl: 4 },
  { id: 'fire_protection', name: '火焰保护', lvl: 4 },
  { id: 'blast_protection', name: '爆炸保护', lvl: 4 },
  { id: 'projectile_protection', name: '弹射物保护', lvl: 4 },
  { id: 'feather_falling', name: '摔落保护', lvl: 4 },
  { id: 'thorns', name: '荆棘', lvl: 3 },
  { id: 'depth_strider', name: '深海探索者', lvl: 3 },
  { id: 'frost_walker', name: '冰霜行者', lvl: 2 },
  { id: 'respiration', name: '水下呼吸', lvl: 3 },
  { id: 'aqua_affinity', name: '水下速掘', lvl: 1 },
  { id: 'looting', name: '抢夺', lvl: 3 },
  { id: 'mending', name: '经验修补', lvl: 1 },
  { id: 'infinity', name: '无限', lvl: 1 },
  { id: 'power', name: '力量', lvl: 5 },
  { id: 'punch', name: '冲击', lvl: 2 },
  { id: 'flame', name: '火矢', lvl: 1 },
  { id: 'knockback', name: '击退', lvl: 2 },
  { id: 'fire_aspect', name: '火焰附加', lvl: 2 },
  { id: 'sweeping', name: '横扫之刃', lvl: 3 },
  { id: 'luck_of_the_sea', name: '海之眷顾', lvl: 3 },
  { id: 'lure', name: '饵钓', lvl: 3 },
  { id: 'loyalty', name: '忠诚', lvl: 3 },
  { id: 'channeling', name: '引雷', lvl: 1 },
  { id: 'riptide', name: '激流', lvl: 3 },
  { id: 'impaling', name: '穿刺', lvl: 5 },
  { id: 'soul_speed', name: '灵魂疾行', lvl: 3 },
  { id: 'swift_sneak', name: '迅捷潜行', lvl: 3 },
]

/** 常见可召唤实体（Java 命名空间 ID + 中文名） */
export interface EntityOption {
  id: string
  name: string
}

export const ENTITIES: EntityOption[] = [
  { id: 'creeper', name: '苦力怕' },
  { id: 'zombie', name: '僵尸' },
  { id: 'skeleton', name: '骷髅' },
  { id: 'spider', name: '蜘蛛' },
  { id: 'enderman', name: '末影人' },
  { id: 'witch', name: '女巫' },
  { id: 'villager', name: '村民' },
  { id: 'iron_golem', name: '铁傀儡' },
  { id: 'snow_golem', name: '雪傀儡' },
  { id: 'cow', name: '牛' },
  { id: 'pig', name: '猪' },
  { id: 'sheep', name: '羊' },
  { id: 'chicken', name: '鸡' },
  { id: 'wolf', name: '狼' },
  { id: 'cat', name: '猫' },
  { id: 'horse', name: '马' },
  { id: 'parrot', name: '鹦鹉' },
  { id: 'axolotl', name: '美西螈' },
  { id: 'armor_stand', name: '盔甲架' },
  { id: 'item', name: '掉落物（物品）' },
  { id: 'arrow', name: '箭矢' },
  { id: 'ender_pearl', name: '末影珍珠' },
  { id: 'fireball', name: '火球' },
  { id: 'lightning_bolt', name: '闪电' },
  { id: 'boat', name: '船' },
  { id: 'warden', name: '循声守卫' },
  { id: 'allay', name: '悦灵' },
  { id: 'elder_guardian', name: '远古守卫者' },
  { id: 'ender_dragon', name: '末影龙' },
  { id: 'wither', name: '凋灵' },
]

/** 告示牌商店：常见的告示牌方块 ID */
export const SIGN_IDS = [
  'oak_sign',
  'spruce_sign',
  'birch_sign',
  'jungle_sign',
  'acacia_sign',
  'dark_oak_sign',
  'mangrove_sign',
  'cherry_sign',
  'bamboo_sign',
  'crimson_sign',
  'warped_sign',
  'pale_oak_sign',
]

/** 告示牌朝向（facing 方块状态） */
export const SIGN_FACINGS = [
  { id: 'north', name: '北 (north)' },
  { id: 'south', name: '南 (south)' },
  { id: 'east', name: '东 (east)' },
  { id: 'west', name: '西 (west)' },
]
