/**
 * 指令生成工具 - 常用数据表
 *
 * 纯前端静态数据（Minecraft Java 1.21+ 命名空间 ID）：
 * - 常用附魔表（附魔 ID + 中文名 + 自然最高等级）
 * - 常见可召唤实体表
 */

/** 常用附魔（Java 命名空间 ID，lvl 为自然获取最高等级，legacy 为 1.12.2 及以前的数字附魔 ID） */
export interface EnchantmentOption {
  id: string
  name: string
  lvl: number
  /** 1.12.2 及以前使用的数字附魔 ID（1.13+ 新增附魔无此字段） */
  legacy?: number
}

export const ENCHANTMENTS: EnchantmentOption[] = [
  { id: 'sharpness', name: '锋利', lvl: 5, legacy: 16 },
  { id: 'smite', name: '亡灵杀手', lvl: 5, legacy: 17 },
  { id: 'bane_of_arthropods', name: '节肢杀手', lvl: 5, legacy: 18 },
  { id: 'efficiency', name: '效率', lvl: 5, legacy: 32 },
  { id: 'fortune', name: '时运', lvl: 3, legacy: 35 },
  { id: 'silk_touch', name: '精准采集', lvl: 1, legacy: 33 },
  { id: 'unbreaking', name: '耐久', lvl: 3, legacy: 34 },
  { id: 'protection', name: '保护', lvl: 4, legacy: 0 },
  { id: 'fire_protection', name: '火焰保护', lvl: 4, legacy: 1 },
  { id: 'blast_protection', name: '爆炸保护', lvl: 4, legacy: 3 },
  { id: 'projectile_protection', name: '弹射物保护', lvl: 4, legacy: 4 },
  { id: 'feather_falling', name: '摔落保护', lvl: 4, legacy: 2 },
  { id: 'thorns', name: '荆棘', lvl: 3, legacy: 7 },
  { id: 'depth_strider', name: '深海探索者', lvl: 3, legacy: 8 },
  { id: 'frost_walker', name: '冰霜行者', lvl: 2, legacy: 9 },
  { id: 'respiration', name: '水下呼吸', lvl: 3, legacy: 5 },
  { id: 'aqua_affinity', name: '水下速掘', lvl: 1, legacy: 6 },
  { id: 'looting', name: '抢夺', lvl: 3, legacy: 21 },
  { id: 'mending', name: '经验修补', lvl: 1, legacy: 70 },
  { id: 'infinity', name: '无限', lvl: 1, legacy: 51 },
  { id: 'power', name: '力量', lvl: 5, legacy: 48 },
  { id: 'punch', name: '冲击', lvl: 2, legacy: 49 },
  { id: 'flame', name: '火矢', lvl: 1, legacy: 50 },
  { id: 'knockback', name: '击退', lvl: 2, legacy: 19 },
  { id: 'fire_aspect', name: '火焰附加', lvl: 2, legacy: 20 },
  { id: 'sweeping', name: '横扫之刃', lvl: 3, legacy: 22 },
  { id: 'luck_of_the_sea', name: '海之眷顾', lvl: 3, legacy: 61 },
  { id: 'lure', name: '饵钓', lvl: 3, legacy: 62 },
  { id: 'loyalty', name: '忠诚', lvl: 3 },
  { id: 'channeling', name: '引雷', lvl: 1 },
  { id: 'riptide', name: '激流', lvl: 3 },
  { id: 'impaling', name: '穿刺', lvl: 5 },
  { id: 'soul_speed', name: '灵魂疾行', lvl: 3 },
  { id: 'swift_sneak', name: '迅捷潜行', lvl: 3 },
]

/** 1.12.2 及以前的附魔 ID 映射（id -> 数字 ID），供旧版本 /give 生成使用 */
export const ENCHANT_IDS_1_12: Record<string, number> = Object.fromEntries(
  ENCHANTMENTS.filter((e) => e.legacy !== undefined).map((e) => [e.id, e.legacy as number]),
)

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

/** 告示牌商店：常见告示牌方块（id 为方块 ID，name 为界面显示名） */
export const SIGN_IDS = [
  { id: 'oak_sign', name: '橡木告示牌' },
  { id: 'spruce_sign', name: '云杉木告示牌' },
  { id: 'birch_sign', name: '白桦木告示牌' },
  { id: 'jungle_sign', name: '丛林木告示牌' },
  { id: 'acacia_sign', name: '金合欢木告示牌' },
  { id: 'dark_oak_sign', name: '深色橡木告示牌' },
  { id: 'mangrove_sign', name: '红树木告示牌' },
  { id: 'cherry_sign', name: '樱花木告示牌' },
  { id: 'bamboo_sign', name: '竹告示牌' },
  { id: 'crimson_sign', name: '绯红木告示牌' },
  { id: 'warped_sign', name: '诡异木告示牌' },
  { id: 'pale_oak_sign', name: '苍白橡木告示牌' },
]

/** 告示牌朝向（facing 方块状态） */
export const SIGN_FACINGS = [
  { id: 'north', name: '北' },
  { id: 'south', name: '南' },
  { id: 'east', name: '东' },
  { id: 'west', name: '西' },
]
