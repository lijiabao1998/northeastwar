//! Modern PLA & Russian combat vehicles (including active Russia-Ukraine war hardware)
//! and infantry/anti-armor/counter-drone weapon rosters.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Faction {
    Pla,
    Russia,
}

impl Faction {
    pub fn label(self) -> &'static str {
        match self {
            Self::Pla => "解放军",
            Self::Russia => "俄军",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum VehicleKind {
    // Russia (10 active / Russo-Ukrainian front-line vehicles)
    T72B3,
    T80Bvm,
    T90M,
    T62M,
    Tos1A,
    Bmpt,
    Bmp3,
    Btr82A,
    Bmd4M,
    TigrM,
    // PLA (7 active armored / AA / assault vehicles)
    Ztz99A,
    Ztz96B,
    Ztq15,
    Zbd04A,
    Zbl08,
    Pgz09,
    Csk181,
}

impl VehicleKind {
    pub const ALL: [Self; 17] = [
        Self::T72B3,
        Self::T80Bvm,
        Self::T90M,
        Self::T62M,
        Self::Tos1A,
        Self::Bmpt,
        Self::Bmp3,
        Self::Btr82A,
        Self::Bmd4M,
        Self::TigrM,
        Self::Ztz99A,
        Self::Ztz96B,
        Self::Ztq15,
        Self::Zbd04A,
        Self::Zbl08,
        Self::Pgz09,
        Self::Csk181,
    ];

    pub const fn index(self) -> usize {
        match self {
            Self::T72B3 => 0,
            Self::T80Bvm => 1,
            Self::T90M => 2,
            Self::T62M => 3,
            Self::Tos1A => 4,
            Self::Bmpt => 5,
            Self::Bmp3 => 6,
            Self::Btr82A => 7,
            Self::Bmd4M => 8,
            Self::TigrM => 9,
            Self::Ztz99A => 10,
            Self::Ztz96B => 11,
            Self::Ztq15 => 12,
            Self::Zbd04A => 13,
            Self::Zbl08 => 14,
            Self::Pgz09 => 15,
            Self::Csk181 => 16,
        }
    }

    pub const fn faction(self) -> Faction {
        match self {
            Self::T72B3
            | Self::T80Bvm
            | Self::T90M
            | Self::T62M
            | Self::Tos1A
            | Self::Bmpt
            | Self::Bmp3
            | Self::Btr82A
            | Self::Bmd4M
            | Self::TigrM => Faction::Russia,
            Self::Ztz99A
            | Self::Ztz96B
            | Self::Ztq15
            | Self::Zbd04A
            | Self::Zbl08
            | Self::Pgz09
            | Self::Csk181 => Faction::Pla,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::T72B3 => "T-72B3 主战坦克",
            Self::T80Bvm => "T-80BVM 燃气轮机坦克",
            Self::T90M => "T-90M「突破-3」主战坦克",
            Self::T62M => "T-62M 改装老式坦克",
            Self::Tos1A => "TOS-1A「喷火坦克」温压火箭炮",
            Self::Bmpt => "BMPT「终结者」坦克支援战车",
            Self::Bmp3 => "BMP-3 履带步战车",
            Self::Btr82A => "BTR-82A 轮式步战车",
            Self::Bmd4M => "BMD-4M 空降战车",
            Self::TigrM => "GAZ「虎」式高机动装甲车",
            Self::Ztz99A => "ZTZ-99A 主战坦克",
            Self::Ztz96B => "ZTZ-96B 主战坦克",
            Self::Ztq15 => "ZTQ-15 轻型坦克",
            Self::Zbd04A => "ZBD-04A 履带步战车",
            Self::Zbl08 => "ZBL-08「大八轮」步战车",
            Self::Pgz09 => "PGZ-09 双35mm 自行高炮",
            Self::Csk181 => "CSK-181「猛士三代」突击车",
        }
    }

    pub const fn max_hp(self) -> f32 {
        match self {
            Self::T72B3 => 340.0,
            Self::T80Bvm => 370.0,
            Self::T90M => 450.0,
            Self::T62M => 260.0,
            Self::Tos1A => 280.0,
            Self::Bmpt => 360.0,
            Self::Bmp3 => 190.0,
            Self::Btr82A => 160.0,
            Self::Bmd4M => 150.0,
            Self::TigrM => 115.0,
            Self::Ztz99A => 480.0,
            Self::Ztz96B => 390.0,
            Self::Ztq15 => 310.0,
            Self::Zbd04A => 230.0,
            Self::Zbl08 => 195.0,
            Self::Pgz09 => 220.0,
            Self::Csk181 => 135.0,
        }
    }

    pub const fn speed(self) -> f32 {
        match self {
            Self::T72B3 | Self::Ztz96B => 2.4,
            Self::T80Bvm | Self::Ztz99A => 3.1,
            Self::T90M | Self::Bmpt => 2.6,
            Self::T62M | Self::Tos1A => 2.0,
            Self::Ztq15 | Self::Zbd04A | Self::Pgz09 => 3.2,
            Self::Bmp3 | Self::Bmd4M => 3.4,
            Self::Btr82A | Self::Zbl08 => 4.1,
            Self::TigrM | Self::Csk181 => 5.0,
        }
    }

    pub const fn is_heavy_armor(self) -> bool {
        matches!(
            self,
            Self::T72B3
                | Self::T80Bvm
                | Self::T90M
                | Self::T62M
                | Self::Tos1A
                | Self::Bmpt
                | Self::Ztz99A
                | Self::Ztz96B
                | Self::Ztq15
        )
    }

    pub const fn is_aa(self) -> bool {
        matches!(self, Self::Pgz09)
    }

    pub const fn is_thermobaric(self) -> bool {
        matches!(self, Self::Tos1A)
    }

    /// Returns `(blast_radius, raw_dmg, structural_dmg, cooldown_s)`.
    pub const fn cannon_profile(self) -> (f32, f32, f32, f32) {
        match self {
            Self::T72B3 => (7.8, 95.0, 240.0, 5.5),
            Self::T80Bvm => (7.8, 100.0, 240.0, 5.0),
            Self::T90M => (8.2, 110.0, 260.0, 4.9),
            Self::T62M => (6.5, 85.0, 200.0, 4.5),
            Self::Tos1A => (11.5, 115.0, 320.0, 7.2),
            Self::Bmpt => (4.2, 46.0, 125.0, 1.8),
            Self::Bmp3 | Self::Bmd4M => (5.0, 56.0, 145.0, 3.0),
            Self::Btr82A => (4.2, 42.0, 115.0, 2.8),
            Self::TigrM | Self::Csk181 => (2.4, 24.0, 30.0, 1.6),
            Self::Ztz99A => (8.2, 115.0, 250.0, 4.8),
            Self::Ztz96B => (7.8, 102.0, 240.0, 5.1),
            Self::Ztq15 => (6.6, 88.0, 205.0, 4.2),
            Self::Zbd04A => (5.0, 58.0, 145.0, 2.9),
            Self::Zbl08 => (4.2, 45.0, 120.0, 2.7),
            Self::Pgz09 => (3.5, 36.0, 80.0, 1.3),
        }
    }

    pub const fn turret_mount_y(self) -> f32 {
        match self {
            Self::T72B3
            | Self::T80Bvm
            | Self::T90M
            | Self::T62M
            | Self::Tos1A
            | Self::Bmpt
            | Self::Ztz99A
            | Self::Ztz96B
            | Self::Ztq15 => 1.06,
            Self::Bmp3 | Self::Bmd4M | Self::Zbd04A | Self::Pgz09 => 1.12,
            Self::Btr82A | Self::Zbl08 => 1.44,
            Self::TigrM | Self::Csk181 => 1.68,
        }
    }

    pub const fn forces_key(self) -> Option<&'static str> {
        match self {
            Self::T72B3 => Some("t72b3"),
            Self::T80Bvm => Some("t80bvm"),
            Self::T90M => Some("t90m"),
            Self::T62M => Some("t62m"),
            Self::Tos1A => Some("tos1a"),
            Self::Bmpt => Some("bmpt"),
            Self::Bmp3 => Some("bmp3"),
            Self::Btr82A => Some("btr82a"),
            Self::Bmd4M => Some("bmd4m"),
            Self::TigrM => Some("tigr_m"),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, serde::Serialize, serde::Deserialize)]
pub enum PrimaryWeapon {
    #[default]
    Qbz191,
    Ak74m,
    Qbu191,
    Pkm,
    Saiga12,
    EwJammer,
}

impl PrimaryWeapon {
    pub const ALL: [Self; 6] = [
        Self::Qbz191,
        Self::Ak74m,
        Self::Qbu191,
        Self::Pkm,
        Self::Saiga12,
        Self::EwJammer,
    ];

    pub const fn index(self) -> usize {
        match self {
            Self::Qbz191 => 0,
            Self::Ak74m => 1,
            Self::Qbu191 => 2,
            Self::Pkm => 3,
            Self::Saiga12 => 4,
            Self::EwJammer => 5,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Qbz191 => "QBZ-191 5.8mm突击步枪",
            Self::Ak74m => "AK-74M 5.45mm步枪",
            Self::Qbu191 => "QBU-191 精准射手步枪",
            Self::Pkm => "PKM 7.62mm通用机枪",
            Self::Saiga12 => "Saiga-12 防无人机霰弹枪",
            Self::EwJammer => "反无人机电磁干扰枪",
        }
    }

    pub const fn short_label(self) -> &'static str {
        match self {
            Self::Qbz191 => "QBZ-191",
            Self::Ak74m => "AK-74M",
            Self::Qbu191 => "QBU-191",
            Self::Pkm => "PKM机枪",
            Self::Saiga12 => "Saiga12",
            Self::EwJammer => "电磁干扰",
        }
    }

    pub const fn damage(self) -> f32 {
        match self {
            Self::Qbz191 => 36.0,
            Self::Ak74m => 32.0,
            Self::Qbu191 => 68.0,
            Self::Pkm => 34.0,
            Self::Saiga12 => 18.0,
            Self::EwJammer => 0.0,
        }
    }

    pub const fn cooldown(self) -> f32 {
        match self {
            Self::Qbz191 => 0.24,
            Self::Ak74m => 0.28,
            Self::Qbu191 => 0.52,
            Self::Pkm => 0.13,
            Self::Saiga12 => 0.45,
            Self::EwJammer => 0.55,
        }
    }

    pub const fn range(self) -> f32 {
        match self {
            Self::Qbz191 => 80.0,
            Self::Ak74m => 72.0,
            Self::Qbu191 => 125.0,
            Self::Pkm => 85.0,
            Self::Saiga12 => 38.0,
            Self::EwJammer => 68.0,
        }
    }

    pub const fn uses_ammo(self) -> bool {
        !matches!(self, Self::EwJammer)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, serde::Serialize, serde::Deserialize)]
pub enum AtWeapon {
    #[default]
    Rpg7,
    Pf98,
    Hj12,
    RpoShmel,
}

impl AtWeapon {
    pub const ALL: [Self; 4] = [Self::Rpg7, Self::Pf98, Self::Hj12, Self::RpoShmel];

    pub const fn index(self) -> usize {
        match self {
            Self::Rpg7 => 0,
            Self::Pf98 => 1,
            Self::Hj12 => 2,
            Self::RpoShmel => 3,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Rpg7 => "RPG-7 火箭筒",
            Self::Pf98 => "PF-98 120mm串联破甲火箭",
            Self::Hj12 => "HJ-12 红箭攻顶导弹",
            Self::RpoShmel => "RPO-A「大黄蜂」温压筒",
        }
    }

    pub const fn short_label(self) -> &'static str {
        match self {
            Self::Rpg7 => "RPG-7",
            Self::Pf98 => "PF-98",
            Self::Hj12 => "HJ-12",
            Self::RpoShmel => "大黄蜂",
        }
    }

    pub const fn direct_tank_dmg(self) -> f32 {
        match self {
            Self::Rpg7 => 140.0,
            Self::Pf98 => 260.0,
            Self::Hj12 => 380.0,
            Self::RpoShmel => 105.0,
        }
    }

    pub const fn splash_dmg(self) -> f32 {
        match self {
            Self::Rpg7 => 165.0,
            Self::Pf98 => 140.0,
            Self::Hj12 => 140.0,
            Self::RpoShmel => 135.0,
        }
    }

    pub const fn radius(self) -> f32 {
        match self {
            Self::Rpg7 => 6.5,
            Self::Pf98 => 7.2,
            Self::Hj12 => 6.8,
            Self::RpoShmel => 11.5,
        }
    }

    pub const fn structural_dmg(self) -> f32 {
        match self {
            Self::Rpg7 => 200.0,
            Self::Pf98 => 260.0,
            Self::Hj12 => 220.0,
            Self::RpoShmel => 280.0,
        }
    }

    pub const fn is_top_attack(self) -> bool {
        matches!(self, Self::Hj12)
    }
}
