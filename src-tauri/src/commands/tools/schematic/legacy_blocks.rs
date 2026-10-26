//! 旧版（1.13 之前）数字 id + 元数据 → 现代方块 id 映射。
//!
//! 1.13 的「扁平化」把 id + 元数据拆成了独立的方块名与 blockstate，1.12 及更早的存档
//! 只有 id + 4 位元数据，因此需要还原。这里用两种方式组合：
//!
//! 1. **位规则**：多数方块的元数据是「变体编号 + 标志位」的位域（如原木的低 2 位是树种、
//!    高 2 位是轴向），按位解出来的结果对任意元数据都成立，不需要枚举。
//! 2. **精确表**：位语义不规则的方块（颜色表、枚举变体）逐个列出。
//!
//! 两者都查不到时返回 `None`，由调用方退化为 `minecraft:legacy_<id>_<meta>`——
//! 宁可显示成一眼能看出的占位符，也不猜一个错误方块。

/// 六种木材（1.12 的 0-5 序号）
const WOOD6: [&str; 6] = ["oak", "spruce", "birch", "jungle", "acacia", "dark_oak"];

/// 十六种染色
const COLORS16: [&str; 16] = [
    "white",
    "orange",
    "magenta",
    "light_blue",
    "yellow",
    "lime",
    "pink",
    "gray",
    "light_gray",
    "cyan",
    "purple",
    "blue",
    "brown",
    "green",
    "red",
    "black",
];

/// 石质半砖/双半砖的 8 种材质
const SLAB8: [&str; 8] = [
    "smooth_stone",
    "sandstone",
    "petrified_oak_slab",
    "cobblestone",
    "brick",
    "stone_brick",
    "nether_brick",
    "quartz",
];

/// 按位规则推导方块名
pub fn legacy_block_name(id: u16, meta: u8) -> Option<String> {
    let m = meta & 0x0f;
    let name = match id {
        // 基础方块
        // 石头：元数据是石种细分
        1 => match m {
            1 => "granite".to_string(),
            2 => "polished_granite".to_string(),
            3 => "diorite".to_string(),
            4 => "polished_diorite".to_string(),
            5 => "andesite".to_string(),
            6 => "polished_andesite".to_string(),
            _ => "stone".to_string(),
        },
        2 => "grass_block".to_string(),
        3 => match m {
            1 => "coarse_dirt".to_string(),
            2 => "podzol".to_string(),
            _ => "dirt".to_string(),
        },
        4 => "cobblestone".to_string(),
        7 => "bedrock".to_string(),
        9 => "water".to_string(),
        11 => "lava".to_string(),
        12 => if m == 1 { "red_sand" } else { "sand" }.to_string(),
        13 => "gravel".to_string(),
        14 => "gold_ore".to_string(),
        15 => "iron_ore".to_string(),
        16 => "coal_ore".to_string(),
        21 => "lapis_ore".to_string(),
        22 => "lapis_block".to_string(),
        41 => "gold_block".to_string(),
        42 => "iron_block".to_string(),
        45 => "bricks".to_string(),
        46 => "tnt".to_string(),
        47 => "bookshelf".to_string(),
        48 => "mossy_cobblestone".to_string(),
        49 => "obsidian".to_string(),
        52 => "spawner".to_string(),
        54 => "chest".to_string(),
        56 => "diamond_ore".to_string(),
        57 => "diamond_block".to_string(),
        58 => "crafting_table".to_string(),
        60 => "farmland".to_string(),
        61 => "furnace".to_string(),
        65 => "ladder".to_string(),
        66 => "rail".to_string(),
        67 => "cobblestone_stairs".to_string(),
        73 => "redstone_ore".to_string(),
        79 => "ice".to_string(),
        80 => "snow_block".to_string(),
        81 => "cactus".to_string(),
        82 => "clay".to_string(),
        83 => "sugar_cane".to_string(),
        87 => "netherrack".to_string(),
        89 => "glowstone".to_string(),
        101 => "iron_bars".to_string(),
        102 => "glass_pane".to_string(),
        106 => "vine".to_string(),
        111 => "lily_pad".to_string(),
        124 | 123 => "redstone_lamp".to_string(),
        129 => "emerald_ore".to_string(),
        133 => "emerald_block".to_string(),
        152 => "redstone_block".to_string(),
        169 => "sea_lantern".to_string(),
        172 => "terracotta".to_string(),
        206 => "end_stone_bricks".to_string(),
        208 => "dirt_path".to_string(),

        // 木材：低 3 位是树种
        5 => format!("{}_planks", WOOD6[(m & 7) as usize % WOOD6.len()]),

        // 原木：低 2 位树种，高 2 位轴向（0=Y 1=X 2=Z 3=全树皮）
        17 => {
            let species = WOOD6[(m & 3) as usize];
            if (m >> 2) & 3 == 3 {
                format!("{}_wood", species)
            } else {
                format!("{}_log", species)
            }
        }
        162 => {
            let species = ["acacia", "dark_oak"][(m & 1) as usize];
            if (m >> 2) & 3 == 3 {
                format!("{}_wood", species)
            } else {
                format!("{}_log", species)
            }
        }

        // 树叶：低 2 位树种（高位的衰减标志不影响外观）
        18 => format!("{}_leaves", WOOD6[(m & 3) as usize]),
        161 => format!("{}_leaves", ["acacia", "dark_oak"][(m & 1) as usize]),

        // 台阶：低 3 位树种，bit3 表示上半砖（位置属性，名称不变）
        126 => format!("{}_slab", WOOD6[(m & 7) as usize % WOOD6.len()]),
        125 => format!("{}_planks", WOOD6[(m & 7) as usize % WOOD6.len()]),
        44 => format!("{}_slab", SLAB8[(m & 7) as usize]),
        43 => SLAB8[(m & 7) as usize].to_string(),

        // 楼梯：元数据是朝向（不影响名称）
        53 => "oak_stairs".to_string(),
        108 => "brick_stairs".to_string(),
        109 => "stone_brick_stairs".to_string(),
        114 => "nether_brick_stairs".to_string(),
        128 => "sandstone_stairs".to_string(),
        134 => "spruce_stairs".to_string(),
        135 => "birch_stairs".to_string(),
        136 => "jungle_stairs".to_string(),
        156 => "quartz_stairs".to_string(),
        163 => "acacia_stairs".to_string(),
        164 => "dark_oak_stairs".to_string(),
        180 => "red_sandstone_stairs".to_string(),
        203 => "purpur_stairs".to_string(),

        // 门与活板门：木材种类由 id 决定，元数据是开合/朝向
        64 => "oak_door".to_string(),
        71 => "iron_door".to_string(),
        96 => "oak_trapdoor".to_string(),
        167 => "iron_trapdoor".to_string(),
        193 => "spruce_door".to_string(),
        194 => "birch_door".to_string(),
        195 => "jungle_door".to_string(),
        196 => "acacia_door".to_string(),
        197 => "dark_oak_door".to_string(),

        // 栅栏与栅栏门
        85 => "oak_fence".to_string(),
        113 => "nether_brick_fence".to_string(),
        183..=187 => format!("{}_fence_gate", WOOD6[(id - 183) as usize]),
        188..=192 => format!("{}_fence", WOOD6[(id - 188) as usize]),

        // 按钮与压力板
        70 => "stone_pressure_plate".to_string(),
        72 => "oak_pressure_plate".to_string(),
        147 => "oak_button".to_string(),
        148 => "spruce_button".to_string(),
        143 => "oak_button".to_string(),
        77 => "stone_button".to_string(),
        69 => "lever".to_string(),

        // 告示牌
        63 => "oak_sign".to_string(),
        68 => "oak_wall_sign".to_string(),

        // 火把：元数据是朝向，0 为站立
        50 => if m == 0 { "torch" } else { "wall_torch" }.to_string(),
        75 | 76 => if m == 0 {
            "redstone_torch"
        } else {
            "redstone_wall_torch"
        }
        .to_string(),

        // 染色系列：元数据即颜色序号
        35 => format!("{}_wool", COLORS16[(m & 15) as usize]),
        95 => format!("{}_stained_glass", COLORS16[(m & 15) as usize]),
        160 => format!("{}_stained_glass_pane", COLORS16[(m & 15) as usize]),
        159 => format!("{}_terracotta", COLORS16[(m & 15) as usize]),
        171 => format!("{}_carpet", COLORS16[(m & 15) as usize]),
        // 带釉陶瓦：颜色由 id 决定（235-250），元数据是朝向
        235..=250 => format!("{}_glazed_terracotta", COLORS16[(id - 235) as usize]),
        251 => format!("{}_concrete", COLORS16[(m & 15) as usize]),
        252 => format!("{}_concrete_powder", COLORS16[(m & 15) as usize]),

        // 树苗与花：元数据是品种
        6 => match m {
            1 => "spruce_sapling".to_string(),
            2 => "birch_sapling".to_string(),
            3 => "jungle_sapling".to_string(),
            4 => "acacia_sapling".to_string(),
            5 => "dark_oak_sapling".to_string(),
            _ => "oak_sapling".to_string(),
        },
        37 => "dandelion".to_string(),
        38 => match m {
            1 => "blue_orchid".to_string(),
            2 => "allium".to_string(),
            3 => "azure_bluet".to_string(),
            4 => "red_tulip".to_string(),
            5 => "orange_tulip".to_string(),
            6 => "white_tulip".to_string(),
            7 => "pink_tulip".to_string(),
            8 => "oxeye_daisy".to_string(),
            _ => "poppy".to_string(),
        },
        39 => "brown_mushroom".to_string(),
        40 => "red_mushroom".to_string(),

        // 蠹虫方块：外观等同于对应石砖，直接映射过去
        97 => match m {
            1 => "cobblestone".to_string(),
            2 => "stone_bricks".to_string(),
            3 => "mossy_stone_bricks".to_string(),
            4 => "cracked_stone_bricks".to_string(),
            5 => "chiseled_stone_bricks".to_string(),
            _ => "stone".to_string(),
        },
        182 => "red_sandstone_slab".to_string(),

        // 植物与装饰
        31 => match m {
            2 => "fern".to_string(),
            0 => "dead_bush".to_string(),
            _ => "grass".to_string(),
        },
        175 => match m & 7 {
            0 => "sunflower".to_string(),
            1 => "lilac".to_string(),
            2 => "tall_grass".to_string(),
            3 => "large_fern".to_string(),
            4 => "rose_bush".to_string(),
            _ => "peony".to_string(),
        },
        168 => match m {
            1 => "prismarine_bricks".to_string(),
            2 => "dark_prismarine".to_string(),
            _ => "prismarine".to_string(),
        },
        170 => "hay_block".to_string(),

        // 石砖 / 砂岩 / 石英的变体
        98 => match m {
            1 => "mossy_stone_bricks".to_string(),
            2 => "cracked_stone_bricks".to_string(),
            3 => "chiseled_stone_bricks".to_string(),
            _ => "stone_bricks".to_string(),
        },
        24 => match m {
            1 => "chiseled_sandstone".to_string(),
            2 => "cut_sandstone".to_string(),
            _ => "sandstone".to_string(),
        },
        179 => match m {
            1 => "chiseled_red_sandstone".to_string(),
            2 => "cut_red_sandstone".to_string(),
            _ => "red_sandstone".to_string(),
        },
        155 => match m {
            1 => "chiseled_quartz_block".to_string(),
            2..=4 => "quartz_pillar".to_string(),
            _ => "quartz_block".to_string(),
        },

        _ => return None,
    };
    Some(format!("minecraft:{}", name))
}

/// 表中收录的规则覆盖范围（供测试断言）
#[cfg(test)]
pub fn legacy_block_count() -> usize {
    // 统计 1.12 全量 (id, meta) 组合中能命名的数量
    let mut total = 0;
    for id in 0u16..=255 {
        for meta in 0u8..=15 {
            if legacy_block_name(id, meta).is_some() {
                total += 1;
            }
        }
    }
    total
}
