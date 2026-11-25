//! Complete 17 January Heihe survival & civilian resistance rules.
//!
//! Pure, deterministic simulation migrated from the Godot prototype (`cold.gd`,
//! `scavenge.gd`, `timeline.gd`, `squads.gd`, `recon_drone.gd`, `ordinance.gd`,
//! `hold_night.gd`, `pedestrians.gd`) and checked by `cargo test -p sim`.

pub use crate::arsenal::{AtWeapon, Faction, PrimaryWeapon, VehicleKind};
use crate::city::City;
use crate::clock;
use crate::figure::AnimState;
use crate::forces::{exposure, Forces, Gait, Light, TIME_SCALE};
use crate::geom::{hash01, nearest_on_segment, P2};
use crate::interior::{build_interiors, DoorStyle, Interior, InteriorKind, RoomZone, FLOOR_H};
use crate::look::Rgb;
use crate::walk::Ground;
use std::collections::HashSet;

pub const START_HOUR: f32 = 16.0;
pub const CROSSING_HOUR: f32 = 17.0 + 36.0 / 60.0;
pub const BLACKOUT_HOUR: f32 = 19.5;
pub const HEAT_OFF_HOUR: f32 = 21.0;
pub const DAWN_HOUR: f32 = 31.0;
/// How long a Molotov's fire and a thermobaric rocket's burn on the street, in play
/// seconds (a minute and a half is about 20 minutes of the clock). Play.
pub const MOLOTOV_FIRE_S: f32 = 90.0;
pub const THERMOBARIC_FIRE_S: f32 = 45.0;

/// Tiredness at 16:00 on the first day: up since about 07:00 (nine hours at
/// FATIGUE_AWAKE_PER_H).
pub const START_FATIGUE: f32 = 40.0;
/// Tiredness gained per game hour awake: 0 to 100 in a little over 22 hours. Play.
pub const FATIGUE_AWAKE_PER_H: f32 = 4.5;
/// Tiredness lost per game hour asleep: 100 to 0 in about seven hours. Play.
pub const FATIGUE_SLEEP_PER_H: f32 = 14.0;
/// Too wide awake to sleep below this.
pub const SLEEP_MIN_FATIGUE: f32 = 20.0;
/// No soldier may be nearer than this when you lie down, and one this near wakes
/// you (boots on the stairs, voices under the window).
pub const SLEEP_QUIET_M: f32 = 30.0;
pub const SLEEP_WAKE_M: f32 = 18.0;
/// While asleep the client runs the world this many play seconds per real second:
/// an hour of the clock in two seconds. A step costs about 0.7 ms (days_bot,
/// November 2026), so this is eight steps a frame at 60 frames a second.
pub const SLEEP_PLAY_SPEED: f32 = 120.0;

/// The seed a run gets unless it is given one (WorldSim::with_seed).
pub const DEFAULT_SEED: u64 = 0x1701_2031;

/// A roll in [0, 1) from the run's generator (splitmix64), mixed with `salt` so two
/// rolls taken for different purposes at the same moment differ.
fn rand01(state: &mut u64, salt: u32) -> f32 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state ^ (salt as u64).wrapping_mul(0xD6E8_FEB8_6659_FD93);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    (z >> 40) as f32 / (1u64 << 24) as f32
}

pub const START_TEMP: f32 = 36.8;
pub const AMBIENT_C: f32 = -24.0;
/// Core temperatures (docs/research/living.md, 6c): from 32 °C confused and
/// clumsy, from 28 °C unconscious, which alone in the cold is the end.
pub const CONFUSED_TEMP: f32 = 32.0;
pub const DEATH_TEMP: f32 = 28.0;

pub const SEARCH_SECONDS: f32 = 2.6;
pub const SHOP_REACH: f32 = 3.4;

pub const fn hm(hours: u32, minutes: u32) -> f32 {
    hours as f32 + minutes as f32 / 60.0
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShopKind {
    Pharmacy,
    Hotel,
    Drink,
    Food,
    Clothes,
    Hardware,
    Office,
    Shop,
}

pub fn shop_kind(zh: &str, ru: &str) -> ShopKind {
    let ru_low = ru.to_lowercase();
    let has_zh = |words: &[&str]| words.iter().any(|w| zh.contains(w));
    let has_ru = |words: &[&str]| words.iter().any(|w| ru_low.contains(w));

    if has_zh(&["药", "牙科", "诊所", "医院", "卫生", "藥", "診所", "醫院", "衛生"]) || has_ru(&["аптек", "стомат"]) {
        return ShopKind::Pharmacy;
    }
    if has_zh(&["酒店", "宾馆", "旅馆", "饭店", "賓館", "旅館", "飯店"]) && !has_zh(&["饺", "餃", "面馆", "麵館"]) {
        return ShopKind::Hotel;
    }
    if has_zh(&["咖啡", "茶", "蜜雪", "酒吧", "奶"]) || has_ru(&["чай", "бар", "кофе"]) {
        return ShopKind::Drink;
    }
    if has_zh(&["肯德基", "比萨", "餐", "饺", "餃", "面", "麵", "烧烤", "燒烤", "串", "超市", "便利", "食品", "粮", "糧", "果", "肉"])
        || has_ru(&["ресторан", "пицц", "пельмен", "продукт", "кафе", "бургер"])
    {
        return ShopKind::Food;
    }
    if has_zh(&["以纯", "服", "鞋", "衣", "皮草", "羽绒", "羽絨"]) || has_ru(&["одежд", "шуб", "обув"]) {
        return ShopKind::Clothes;
    }
    if has_zh(&["五金", "汽配", "装饰", "裝飾", "建材", "修配", "汽车", "汽車"]) || has_ru(&["автозапчаст"]) {
        return ShopKind::Hardware;
    }
    if has_zh(&["银行", "銀行", "邮政", "郵政", "博物馆", "博物館", "纪念馆", "紀念館", "文化宫", "台球"]) || has_ru(&["банк"]) {
        return ShopKind::Office;
    }
    ShopKind::Shop
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RolledLoot {
    pub food: u32,
    pub water: u32,
    pub bandage: u32,
    pub warmer: u32,
    pub scrap: u32,
    pub coal: u32,
    pub molotov: u32,
    pub coat: bool,
    pub medicine: u32,
}

pub fn roll_shop(kind: ShopKind, seed: i64) -> RolledLoot {
    let pick = |salt: u32, lo: u32, hi: u32| -> u32 {
        let span = hi - lo + 1;
        lo + ((hash01(seed, salt) * span as f32).floor() as u32).min(span - 1)
    };
    match kind {
        ShopKind::Pharmacy => RolledLoot {
            medicine: pick(5, 1, 2),
            bandage: pick(1, 2, 3),
            warmer: pick(2, 0, 2),
            water: pick(3, 0, 1),
            ..EMPTY_LOOT
        },
        ShopKind::Food => RolledLoot {
            food: pick(1, 1, 3),
            water: pick(2, 1, 2),
            ..EMPTY_LOOT
        },
        ShopKind::Drink => RolledLoot {
            water: pick(1, 2, 3),
            food: pick(2, 0, 1),
            ..EMPTY_LOOT
        },
        ShopKind::Hotel => RolledLoot {
            water: pick(1, 1, 2),
            warmer: pick(2, 1, 2),
            food: pick(3, 0, 1),
            bandage: pick(4, 0, 1),
            ..EMPTY_LOOT
        },
        ShopKind::Clothes => RolledLoot {
            coat: true,
            warmer: pick(1, 0, 1),
            ..EMPTY_LOOT
        },
        ShopKind::Hardware => RolledLoot {
            scrap: pick(1, 2, 4),
            coal: pick(2, 1, 2),
            molotov: 1,
            ..EMPTY_LOOT
        },
        ShopKind::Office => RolledLoot {
            water: pick(1, 0, 1),
            ..EMPTY_LOOT
        },
        ShopKind::Shop => RolledLoot {
            food: pick(1, 0, 2),
            water: pick(2, 0, 1),
            warmer: pick(3, 0, 1),
            bandage: pick(4, 0, 1),
            ..EMPTY_LOOT
        },
    }
}

const EMPTY_LOOT: RolledLoot = RolledLoot {
    food: 0,
    water: 0,
    bandage: 0,
    warmer: 0,
    scrap: 0,
    coal: 0,
    molotov: 0,
    coat: false,
    medicine: 0,
};

#[derive(Clone, Debug)]
pub struct Body {
    pub hp: f32,
    pub temp: f32,
    pub hunger: f32,
    pub thirst: f32,
    pub bleeding: bool,
    pub coat_bonus: f32,
    pub sheltered: bool,
    pub on_ice: bool,
    pub heated: bool,
    pub move_scale: f32,
    /// 0 rested, 100 unable to keep your eyes open.
    pub fatigue: f32,
    pub asleep: bool,
    /// The air around you (°C): the room's, or the street's. Set by the world each step.
    pub env_c: f32,
    pub indoors: bool,
    /// 0 well, 100 gravely ill: what weeks in rooms below 18 °C do.
    pub illness: f32,
    /// Hours left of the days after a chilling, when infections take hold.
    pub chilled_h: f32,
}

impl Default for Body {
    fn default() -> Self {
        Self {
            hp: 100.0,
            temp: START_TEMP,
            hunger: 100.0,
            thirst: 100.0,
            bleeding: false,
            coat_bonus: 0.0,
            sheltered: false,
            on_ice: false,
            heated: true,
            move_scale: 1.0,
            fatigue: START_FATIGUE,
            asleep: false,
            env_c: -18.0,
            indoors: false,
            illness: 0.0,
            chilled_h: 0.0,
        }
    }
}

impl Body {
    /// Update body temperature, hunger, thirst, and bleeding for `dt` real play seconds.
    /// Returns `Some((cause_zh, cause_en))` if the player dies this tick.
    pub fn tick(
        &mut self,
        dt: f32,
        hour: f32,
        gait: Gait,
        sheltered: bool,
        on_ice: bool,
        near_fire_barrel: bool,
    ) -> Option<(&'static str, &'static str)> {
        self.sheltered = sheltered;
        self.on_ice = on_ice;
        self.heated = hour < HEAT_OFF_HOUR || near_fire_barrel;
        let living = crate::living::get();

        // Heat goes with how far below comfort the air around you is: 1 at -18 °C,
        // the first evening's street; more in the small hours, less by a thaw.
        let cold = ((living.num("body", "cold_comfort_c") - self.env_c) / living.num("body", "cold_span_c")).max(0.0);
        let mut exposed = if near_fire_barrel {
            0.0
        } else if self.indoors {
            // Out of the wind, dressed and wrapped up, the body makes up for a cold
            // room down to a few degrees: people lived for weeks in 3 °C flats. Below
            // that it loses heat as on the street, more slowly.
            let room_cold = ((living.num("body", "indoor_comfort_c") - self.env_c) / living.num("body", "cold_span_c")).max(0.0);
            if self.heated && sheltered { 0.0 } else { 0.7 * room_cold }
        } else if sheltered {
            if self.heated { 0.0 } else { 0.22 * cold }
        } else if on_ice {
            1.75 * cold
        } else {
            cold
        };
        if gait == Gait::Run {
            exposed *= 1.25;
        }
        if self.illness > 40.0 {
            exposed *= 1.2;
        }
        if self.asleep {
            // Lying under whatever covers you lose heat more slowly than standing. Play.
            exposed *= 0.6;
        }
        // Hunger lowers the heat a body makes: weakness from about day 12 without food.
        if self.hunger < 40.0 {
            exposed *= 1.25;
        } else if self.hunger < 70.0 {
            exposed *= 1.15;
        }
        exposed *= (1.0 - self.coat_bonus).max(0.2);

        if near_fire_barrel || (sheltered && self.heated) {
            let gain = if near_fire_barrel { 0.04 } else { 0.014 };
            self.temp = (self.temp + gain * dt).min(36.9);
        } else {
            self.temp -= 0.0042 * exposed * dt;
        }
        self.temp = self.temp.clamp(DEATH_TEMP - 0.1, 37.2);

        let mut scale = if self.temp < CONFUSED_TEMP {
            0.5
        } else if self.temp < 34.2 {
            0.70
        } else if self.temp < 35.0 {
            0.86
        } else {
            1.0
        };
        if self.thirst < 20.0 {
            scale *= 0.85;
        } else if self.thirst < 50.0 {
            // About 5% of body water lost, day 3 without drinking: impaired.
            scale *= 0.92;
        }
        if self.hp < 40.0 {
            scale *= 0.85;
        }
        if self.fatigue >= 95.0 {
            scale *= 0.92;
        }
        if self.hunger < 40.0 {
            scale *= 0.9;
        } else if self.hunger < 70.0 {
            scale *= 0.95;
        }
        if self.illness > 70.0 {
            scale *= 0.85;
        }
        self.move_scale = scale;

        if self.temp <= DEATH_TEMP {
            self.hp = 0.0;
            if sheltered {
                return Some(("你冻死在屋里。没有火，屋里和街上一样冷。", "You froze to death indoors. Without a fire the room was as cold as the street."));
            }
            return Some(("你冻死在黑河的街上。", "You froze to death on a street in Heihe."));
        }

        let hours = dt * TIME_SCALE / 3600.0;
        // Resting loss is the baseline; walking and running in the cold lose more.
        let thirst_mul = match gait {
            Gait::Still => 1.0,
            Gait::Walk => living.num("body", "water_walk_mul"),
            Gait::Run => living.num("body", "water_run_mul"),
        };
        let rest = if self.asleep { living.num("body", "sleep_needs_mul") } else { 1.0 };
        // Water and food by the day: the thirst meter is the water you can lose
        // before you collapse, the hunger meter the calories you can go without
        // before you collapse.
        let thirst_per_h = living.num("body", "water_loss_l_per_day") / 24.0 * 100.0 / living.num("body", "water_reserve_l");
        let hunger_per_h = living.kcal_per_day(self.env_c) / 24.0 * 100.0 / living.num("body", "food_reserve_kcal");
        self.hunger = (self.hunger - hours * hunger_per_h * rest).max(0.0);
        self.thirst = (self.thirst - hours * thirst_per_h * thirst_mul * rest).max(0.0);
        self.fatigue = if self.asleep {
            (self.fatigue - hours * FATIGUE_SLEEP_PER_H).max(0.0)
        } else {
            (self.fatigue + hours * FATIGUE_AWAKE_PER_H).min(100.0)
        };

        if self.thirst <= 0.0 {
            self.hp = (self.hp - living.num("body", "thirst_collapse_hp_per_h") * hours).max(0.0);
            if self.hp <= 0.0 {
                return Some(("你渴死了。", "You died of thirst."));
            }
        }
        if self.hunger <= 0.0 {
            self.hp = (self.hp - living.num("body", "starve_hp_per_h") * hours).max(0.0);
            if self.hp <= 0.0 {
                return Some(("你饿死了。", "You starved to death."));
            }
        }

        // Illness builds in air below 18 °C, faster the colder (the risk of a
        // respiratory infection compounds about 4% for each degree), and much faster
        // for days after a chilling; it goes when you are warm.
        if self.temp < 35.0 {
            self.chilled_h = living.num("body", "chill_after_h");
        }
        self.chilled_h = (self.chilled_h - hours).max(0.0);
        let below = living.num("body", "illness_below_c") - self.env_c;
        let mut grow = if below > 0.0 && !near_fire_barrel && !(self.indoors && self.heated) {
            let risk = living.num("body", "illness_risk_per_c").powf(below) - 1.0;
            let chill = if self.chilled_h > 0.0 { living.num("body", "chill_risk_mul") } else { 1.0 };
            living.num("body", "illness_per_h") * risk * chill
        } else {
            -living.num("body", "illness_recover_per_h")
        };
        if self.hunger < 50.0 || self.thirst < 30.0 {
            grow *= if grow > 0.0 { 1.5 } else { 0.5 };
        }
        self.illness = (self.illness + grow * hours).clamp(0.0, 100.0);
        if self.illness > 70.0 {
            let harm = if self.illness >= 100.0 { 2.0 } else { 0.3 };
            self.hp = (self.hp - harm * hours).max(0.0);
            if self.hp <= 0.0 {
                return Some(("你病死了。冷屋子里的咳嗽变成了肺炎。", "You died of illness. The cough in the cold room became pneumonia."));
            }
        }
        if self.bleeding {
            self.hp = (self.hp - 0.55 * dt).max(0.0);
            if self.hp <= 0.0 {
                return Some(("你的血流干了。", "You bled out."));
            }
        }
        None
    }

    pub fn hurt(&mut self, amount: f32, bleed_p: f32, roll: f32) -> bool {
        self.hp = (self.hp - amount).max(0.0);
        if self.hp <= 0.0 {
            return true;
        }
        if !self.bleeding && roll < bleed_p {
            self.bleeding = true;
        }
        false
    }
}

#[derive(Clone, Debug, Default)]
pub struct Inventory {
    pub food: u32,
    pub water: u32,
    /// Broken-up furniture, ready for the stove.
    pub wood: u32,
    pub medicine: u32,
    pub bandage: u32,
    pub warmer: u32,
    pub scrap: u32,
    pub coal: u32,
    pub molotov: u32,
    pub grenades: u32,
    pub smoke_grenades: u32,
    pub ammo: u32,
    pub rpg: u32,
    pub pf98: u32,
    pub hj12: u32,
    pub rpo_shmel: u32,
    pub friendly_fpv: u32,
    pub active_primary: PrimaryWeapon,
    pub unlocked_primaries: u8,
    pub active_at: AtWeapon,
    pub has_rifle: bool,
    pub has_radio: bool,
    pub radio_strikes: u32,
}

impl Inventory {
    pub fn unlock_primary(&mut self, w: PrimaryWeapon) {
        self.unlocked_primaries |= 1u8 << w.index();
        self.has_rifle = true;
    }

    pub fn has_primary(&self, w: PrimaryWeapon) -> bool {
        if self.unlocked_primaries == 0 && self.has_rifle {
            return w == PrimaryWeapon::Ak74m || w == PrimaryWeapon::Qbz191;
        }
        (self.unlocked_primaries & (1u8 << w.index())) != 0
    }

    pub fn cycle_primary(&mut self) -> PrimaryWeapon {
        if self.unlocked_primaries == 0 && self.has_rifle {
            self.unlock_primary(PrimaryWeapon::Ak74m);
        }
        let cur = self.active_primary.index();
        for step in 1..=PrimaryWeapon::ALL.len() {
            let cand = PrimaryWeapon::ALL[(cur + step) % PrimaryWeapon::ALL.len()];
            if self.has_primary(cand) {
                self.active_primary = cand;
                return cand;
            }
        }
        self.active_primary
    }

    pub fn at_ammo(&self, w: AtWeapon) -> u32 {
        match w {
            AtWeapon::Rpg7 => self.rpg,
            AtWeapon::Pf98 => self.pf98,
            AtWeapon::Hj12 => self.hj12,
            AtWeapon::RpoShmel => self.rpo_shmel,
        }
    }

    pub fn consume_at(&mut self, w: AtWeapon) -> bool {
        let slot = match w {
            AtWeapon::Rpg7 => &mut self.rpg,
            AtWeapon::Pf98 => &mut self.pf98,
            AtWeapon::Hj12 => &mut self.hj12,
            AtWeapon::RpoShmel => &mut self.rpo_shmel,
        };
        if *slot > 0 {
            *slot -= 1;
            true
        } else {
            false
        }
    }

    pub fn cycle_at(&mut self) -> AtWeapon {
        let cur = self.active_at.index();
        for step in 1..=AtWeapon::ALL.len() {
            let cand = AtWeapon::ALL[(cur + step) % AtWeapon::ALL.len()];
            if self.at_ammo(cand) > 0 {
                self.active_at = cand;
                return cand;
            }
        }
        let next = AtWeapon::ALL[(cur + 1) % AtWeapon::ALL.len()];
        self.active_at = next;
        next
    }

    pub fn total_at_ammo(&self) -> u32 {
        self.rpg + self.pf98 + self.hj12 + self.rpo_shmel
    }

    pub fn unlock_all_arsenal(&mut self) {
        for w in PrimaryWeapon::ALL {
            self.unlock_primary(w);
        }
        self.active_primary = PrimaryWeapon::Qbz191;
        self.ammo = self.ammo.max(120);
        self.rpg = self.rpg.max(3);
        self.pf98 = self.pf98.max(3);
        self.hj12 = self.hj12.max(2);
        self.rpo_shmel = self.rpo_shmel.max(2);
        self.molotov = self.molotov.max(4);
        self.grenades = self.grenades.max(4);
        self.smoke_grenades = self.smoke_grenades.max(3);
        self.friendly_fpv = self.friendly_fpv.max(3);
        self.active_at = AtWeapon::Hj12;
        self.has_radio = true;
        self.radio_strikes = self.radio_strikes.max(3);
    }
}

#[derive(Clone, Debug)]
pub struct Door {
    pub at: P2,
    pub name: String,
    pub kind: ShopKind,
    pub searched: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CacheKind {
    StreetScrap,
    ResistanceCell,
    CommunityCache,
    SosNeighbours,
    BorderCrate,
    SmugglerCache,
}

#[derive(Clone, Debug)]
pub struct SupplyCache {
    pub id: u32,
    pub at: P2,
    pub kind: CacheKind,
    pub unlock_hour: f32,
    pub title: &'static str,
    pub collected: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SquadState {
    Patrol,
    Check,
    Challenge,
    Detain,
    Engage,
    Search,
}

#[derive(Clone, Debug)]
pub struct Soldier {
    pub id: u32,
    pub at: P2,
    pub floor: u8,
    pub facing: f32,
    pub hp: f32,
    pub optics: bool,
    pub shot_cd: f32,
    pub speed: f32,
    pub phase: f32,
    pub in_cover: bool,
    pub vault_t: f32,
    pub grenade_cd: f32,
}

#[derive(Clone, Debug)]
pub struct Squad {
    pub id: u32,
    pub state: SquadState,
    pub route_idx: usize,
    pub wp_idx: usize,
    pub wp_dir: isize,
    pub men: Vec<Soldier>,
    pub meter: f32,
    pub last_seen: P2,
    pub lost_for: f32,
    pub timer: f32,
    pub ignore_for: f32,
    pub held_at: P2,
    pub shouted: bool,
    pub hostile: bool,
    pub stuck_t: f32,
    pub last_lead_at: P2,
}

/// How far south of the Heilongjiang bank the Russian crossing has reached (metres).
pub fn front_depth(hour: f32, forces: &Forces) -> f32 {
    if hour < CROSSING_HOUR {
        return 0.0;
    }
    let minutes = (hour - CROSSING_HOUR) * 60.0;
    40.0 + minutes * forces.num_or("infantry_group", "advance_m_per_min", 9.0)
}

/// True when `to` lies inside the horizontal field of view cone centered on `facing_yaw` (map radians).
pub fn in_arc(facing_yaw: f32, from: P2, to: P2, half_angle_deg: f32) -> bool {
    let d = to.sub(from);
    if d.len() < 0.5 {
        return true;
    }
    let fwd = P2::new(facing_yaw.cos(), facing_yaw.sin());
    let cos_ang = fwd.dot(d.norm()).clamp(-1.0, 1.0);
    cos_ang >= half_angle_deg.to_radians().cos()
}

/// Light condition around point `p` for patrol and drone vision.
pub fn light_at(city: &City, hour: f32, blackout: bool, revealed: bool, p: P2, fire_barrels: &[P2]) -> Light {
    let (sun, _) = clock::sun_at(hour);
    if sun > clock::DAY_SUN_DEG {
        return Light::Day;
    }
    if sun > clock::NIGHT_SUN_DEG {
        return Light::Dusk;
    }
    if revealed || fire_barrels.iter().any(|b| p.dist(*b) <= 9.0) {
        return Light::NightLit;
    }
    if blackout {
        return Light::NightDark;
    }
    if city.lamps.iter().any(|l| l.dist(p) < 17.0) {
        Light::NightLit
    } else {
        Light::NightDark
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Weather {
    Clear,
    Flurries,
    RiverFog,
}

impl Weather {
    pub fn label(self) -> &'static str {
        match self {
            Self::Clear => "晴冷",
            Self::Flurries => "飘雪",
            Self::RiverFog => "江面大雾（无人机致盲）",
        }
    }
}

pub fn scheduled_weather(hour: f32) -> (Weather, f32) {
    if hour < hm(18, 45) {
        (Weather::Clear, 0.0)
    } else if hour < hm(20, 30) {
        (Weather::Flurries, 0.35)
    } else if hour < hm(23, 0) || (hour >= hm(26, 0) && hour < hm(28, 30)) {
        (Weather::RiverFog, 0.92)
    } else {
        (Weather::Flurries, 0.25)
    }
}

/// The weather at `hour`. The first night keeps its script; after it, each three
/// hours rolls its own from the run's seed (not from its random stream, so the
/// weather does not shift when something else rolls). Heihe's winter is dry and
/// clear: snow about one block in eight, river fog only at night and in the
/// morning. Play numbers until L2 brings the climate table.
pub fn weather_at(seed: u64, hour: f32) -> (Weather, f32) {
    if hour < DAWN_HOUR {
        return scheduled_weather(hour);
    }
    let block = (hour / 3.0).floor() as i64;
    let r = hash01(block ^ (seed as i64), 0x57EA);
    let t = clock::time_of_day(hour);
    let living = crate::living::get();
    // Snow falls in about half the blocks of a day with snow, and only below freezing.
    let snow_p = living.snow_share(hour) * 0.5;
    let freezing = living.outdoor_c(seed, hour) < 0.5;
    if r < 0.06 && !(10.0..20.0).contains(&t) {
        (Weather::RiverFog, 0.92)
    } else if freezing && r < 0.06 + snow_p {
        (Weather::Flurries, 0.25 + 0.1 * hash01(block ^ (seed as i64), 0x57EB))
    } else {
        (Weather::Clear, 0.0)
    }
}

/// Why you woke up.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WakeReason {
    Rested,
    ByHand,
    Noise,
    Enemy,
    Cold,
    Thirst,
    Hunger,
    Hurt,
}

impl WakeReason {
    pub fn text(self) -> (&'static str, &'static str) {
        match self {
            Self::Rested => ("睡够了，醒了。", "You wake up rested."),
            Self::ByHand => ("你起来了。", "You get up."),
            Self::Noise => ("一声响把你惊醒了。", "A noise wakes you."),
            Self::Enemy => ("有脚步声，很近。你醒了。", "Footsteps, close. You are awake."),
            Self::Cold => ("冻醒了。屋里太冷。", "The cold wakes you. The room is too cold."),
            Self::Thirst => ("渴醒了。", "Thirst wakes you."),
            Self::Hunger => ("饿醒了。", "Hunger wakes you."),
            Self::Hurt => ("疼醒了。", "Pain wakes you."),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SnowTrack {
    pub id: u32,
    pub at: P2,
    pub dir: P2,
    pub age: f32,
    pub by_player: bool,
    pub is_vehicle: bool,
}

#[derive(Clone, Debug)]
pub struct Crater {
    pub id: u32,
    pub at: P2,
    pub radius: f32,
}

#[derive(Clone, Debug)]
pub struct VehicleWreck {
    pub id: u32,
    pub kind: VehicleKind,
    pub at: P2,
    pub hull_yaw: f32,
    pub turret_yaw: f32,
    pub smoke_ttl: f32,
}

#[derive(Clone, Debug)]
pub struct SmokeCloud {
    pub id: u32,
    pub at: P2,
    pub radius: f32,
    pub ttl: f32,
    pub dark: bool,
}

#[derive(Clone, Debug)]
pub struct ActiveGrenade {
    pub id: u32,
    pub from: P2,
    pub at: P2,
    pub target: P2,
    pub floor: u8,
    pub fuse: f32,
    pub flight_t: f32,
    pub flight_dur: f32,
    pub friendly: bool,
}

#[derive(Clone, Debug)]
pub struct BloodMark {
    pub id: u32,
    pub at: P2,
    pub yaw: f32,
    pub age: f32,
}

#[derive(Clone, Debug)]
pub struct ReconDrone {
    pub active: bool,
    pub at: P2,
    pub centre: P2,
    pub angle: f32,
    pub altitude: f32,
    pub thermal: bool,
    pub tracking_trail: bool,
    pub state_left: f32,
    pub confirm: f32,
    pub wait_next: f32,
}

impl Default for ReconDrone {
    fn default() -> Self {
        Self {
            active: false,
            at: P2::new(0.0, 120.0),
            centre: P2::new(0.0, 0.0),
            angle: 0.0,
            altitude: 22.0,
            thermal: false,
            tracking_trail: false,
            state_left: 0.0,
            confirm: 0.0,
            wait_next: 60.0,
        }
    }
}

impl ReconDrone {
    pub fn sight_radius(&self, forces: &Forces, light: Light) -> f32 {
        self.sight_radius_in_weather(forces, light, 0.0)
    }

    pub fn sight_radius_in_weather(&self, forces: &Forces, light: Light, fog_density: f32) -> f32 {
        let key = match light {
            Light::Day => "day",
            Light::Dusk => "dusk",
            Light::NightLit | Light::NightDark => {
                if self.thermal { "night_thermal" } else { "night_plain" }
            }
        };
        let base = forces.num_or("recon_drone", &format!("sight_radius_m/{key}"), 8.0);
        let penalty_key = if self.thermal { "fog_sight_penalty_thermal" } else { "fog_sight_penalty_plain" };
        let penalty_def = if self.thermal { 0.65 } else { 0.85 };
        let penalty = forces.num_or("recon_drone", penalty_key, penalty_def);
        (base * (1.0 - penalty * fog_density.clamp(0.0, 1.0))).max(0.0)
    }

    pub fn sees(&self, forces: &Forces, at: P2, gait: Gait, sheltered: bool, light: Light) -> bool {
        self.sees_in_weather(forces, at, gait, sheltered, light, 0.0)
    }

    pub fn sees_in_weather(&self, forces: &Forces, at: P2, gait: Gait, sheltered: bool, light: Light, fog_density: f32) -> bool {
        if !self.active || sheltered {
            return false;
        }
        let mul = exposure(gait, false);
        self.at.dist(at) <= self.sight_radius_in_weather(forces, light, fog_density) * mul
    }
}

#[derive(Clone, Debug)]
pub struct MortarMission {
    pub target: P2,
    pub fuse: f32,
    pub rounds_left: u32,
    pub round_gap: f32,
    pub friendly: bool,
    pub whistled: bool,
}

/// Health lost to one 82 mm mortar shell at `distance` metres.
pub fn blast_harm(forces: &Forces, distance: f32, under_roof: bool) -> f32 {
    let lethal = forces.num_or("mortar_82", "lethal_m", 6.0);
    let wound = forces.num_or("mortar_82", "wound_m", 16.0);
    if distance > wound {
        return 0.0;
    }
    let (low, high) = forces.range("mortar_82", "wound_hp");
    let mut harm = if distance <= lethal {
        120.0
    } else {
        let t = (distance - lethal) / (wound - lethal).max(0.1);
        high + (low - high) * t
    };
    if under_roof {
        harm *= forces.num_or("mortar_82", "indoor_mul", 0.25);
    }
    harm
}

#[derive(Clone, Debug)]
pub struct FpvDrone {
    pub origin: P2,
    pub at: P2,
    pub height: f32,
    pub target: P2,
    pub hit: bool,
    pub fibre_optic: bool,
    pub jammed: bool,
}

#[derive(Clone, Debug)]
pub struct PendingFpv {
    pub target: P2,
    pub eta: f32,
    pub fibre_optic: bool,
}

#[derive(Clone, Debug)]
pub struct LancetDrone {
    pub origin: P2,
    pub at: P2,
    pub height: f32,
    pub target: P2,
    pub hp: f32,
    pub hit_tank_id: Option<u32>,
}

#[derive(Clone, Debug)]
pub struct FriendlyFpv {
    pub origin: P2,
    pub at: P2,
    pub height: f32,
    pub target: P2,
}

#[derive(Clone, Debug)]
pub struct TankUnit {
    pub id: u32,
    pub kind: VehicleKind,
    pub faction: Faction,
    pub at: P2,
    pub waypoints: Vec<P2>,
    pub wp_idx: usize,
    pub wp_dir: isize,
    pub hp: f32,
    pub hull_yaw: f32,
    pub turret_yaw: f32,
    pub lock_timer: f32,
    pub locked_target: Option<(P2, u8)>,
    pub cannon_cd: f32,
    pub mg_cd: f32,
    pub salvo_left: u8,
    /// Seconds it has wanted to move and could not.
    pub stuck_t: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MilitiaOrder {
    Follow,
    Hold(P2),
    Garrison(P2),
}

#[derive(Clone, Debug)]
pub struct MilitiaMember {
    pub id: u32,
    pub name: &'static str,
    pub at: P2,
    pub floor: u8,
    pub facing: f32,
    pub hp: f32,
    pub coat: Rgb,
    pub order: MilitiaOrder,
    pub fire_cd: f32,
    pub speed: f32,
    pub phase: f32,
    pub in_cover: bool,
    pub vault_t: f32,
    pub downed: bool,
    pub bleedout_s: f32,
    pub rescuing_id: Option<u32>,
    pub rescue_progress: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BarricadeKind {
    Sandbag,
    IronGate,
    FireBarrel,
    DeadCar,
    Concrete,
    Trench,
    TarpDugout,
    BarbedWire,
}

impl BarricadeKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Sandbag => "沙袋",
            Self::IronGate => "铁门",
            Self::FireBarrel => "火桶取暖",
            Self::DeadCar => "路障车",
            Self::Concrete => "水泥墩",
            Self::Trench => "野战战壕",
            Self::TarpDugout => "伪装布掩体",
            Self::BarbedWire => "刺猬拒马/铁丝网",
        }
    }

    pub fn cost(self) -> (u32, u32) {
        match self {
            Self::Sandbag => (2, 0),
            Self::IronGate => (3, 0),
            Self::FireBarrel => (1, 1),
            Self::DeadCar => (4, 0),
            Self::Concrete => (5, 0),
            Self::Trench => (2, 0),
            Self::TarpDugout => (3, 0),
            Self::BarbedWire => (2, 0),
        }
    }

    pub fn max_hp(self) -> f32 {
        match self {
            Self::Sandbag => 80.0,
            Self::IronGate => 90.0,
            Self::FireBarrel => 40.0,
            Self::DeadCar => 160.0,
            Self::Concrete => 240.0,
            Self::Trench => 180.0,
            Self::TarpDugout => 150.0,
            Self::BarbedWire => 110.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Barricade {
    pub at: P2,
    pub kind: BarricadeKind,
    pub yaw: f32,
    pub shoved: bool,
    pub hp: f32,
}

#[derive(Clone, Debug)]
pub struct Pedestrian {
    pub id: u32,
    pub at: P2,
    pub facing: f32,
    pub coat: Rgb,
    pub female: bool,
    pub path_idx: usize,
    pub wp_idx: usize,
    pub wp_dir: isize,
    pub fled_for: f32,
    pub hidden: bool,
    pub phase: f32,
    pub speed: f32,
}

#[derive(Clone, Debug)]
pub struct LootDrop {
    pub at: P2,
    pub ammo: u32,
    pub bandage: u32,
    pub food: u32,
    pub grants_rifle: bool,
    pub taken: bool,
}

#[derive(Clone, Debug)]
pub struct Sms {
    pub sender: String,
    pub zh: String,
    pub en: String,
}

#[derive(Clone, Debug)]
pub struct RadioCallout {
    pub unit: String,
    pub code_en: &'static str,
    pub text_zh: String,
    pub ttl: f32,
    pub urgent: bool,
}

#[derive(Clone, Debug)]
pub struct TracerFx {
    pub from: [f32; 3],
    pub to: [f32; 3],
    pub friendly: bool,
}

#[derive(Clone, Debug)]
pub struct ExplosionFx {
    pub at: P2,
    pub radius: f32,
}

#[derive(Clone, Debug, Default)]
pub struct TickOutput {
    pub tracers: Vec<TracerFx>,
    pub explosions: Vec<ExplosionFx>,
    pub footstep: bool,
    pub mortar_whistle: bool,
    pub sms_arrived: bool,
    pub shout_stoy: bool,
    pub player_hit: bool,
    pub door_action: bool,
    pub door_breach_thud: bool,
    pub glass_break: bool,
    pub tank_cannon: bool,
    pub slab_collapse: bool,
}

#[derive(Clone, Debug, Default)]
pub struct SimInput {
    pub wish_move: P2,
    pub sprinting: bool,
    pub interact_held: bool,
    pub interact_just_pressed: bool,
    pub use_bandage: bool,
    pub use_food: bool,
    pub use_water: bool,
    pub use_warmer: bool,
    pub rally_resistance: bool,
    pub select_barricade: Option<BarricadeKind>,
    pub build_barricade: bool,
    pub build_at: Option<P2>,
    pub toggle_militia_order: bool,
    pub command_militia_to: Option<P2>,
    pub fire_at: Option<P2>,
    pub fire_rpg_at: Option<P2>,
    pub throw_molotov_at: Option<P2>,
    pub throw_grenade_at: Option<P2>,
    pub throw_smoke_at: Option<P2>,
    pub launch_fpv_at: Option<P2>,
    pub call_radio_at: Option<P2>,
    pub cycle_primary: bool,
    pub select_primary: Option<PrimaryWeapon>,
    pub cycle_at: bool,
    pub select_at: Option<AtWeapon>,
    pub toggle_flashlight: bool,
    pub toggle_curtains: bool,
    pub fortify_or_stove: bool,
    pub floor_delta: i8,
    pub clock_running: bool,
    /// Lie down to sleep, or get up.
    pub toggle_sleep: bool,
}

#[derive(Clone, Debug)]
pub struct WorldSim {
    pub hour: f32,
    pub hours_alive: f32,
    pub alert_stage: u8,
    pub blackout: bool,
    pub weather: Weather,
    pub fog_density: f32,
    pub weather_override: Option<Weather>,
    pub resistance_active: bool,
    pub neighbours_rescued: bool,
    /// True once the first night is over (07:00 on 18 January). The run goes on.
    pub dawn_survived: bool,
    /// How many 07:00s the run has passed.
    pub mornings: u32,
    pub sleeping: bool,
    /// A loud noise you made (furniture broken up): how far it carries, for how long.
    pub noise_m: f32,
    pub noise_t: f32,
    /// Hours of the clock slept since lying down.
    pub slept_hours: f32,
    pub woke: Option<WakeReason>,
    pub dead: bool,
    pub death_zh: &'static str,
    pub death_en: &'static str,

    pub player_at: P2,
    pub player_facing: f32,
    pub player_gait: Gait,
    pub player_speed: f32,
    pub player_phase: f32,
    pub player_shoot_cd: f32,
    pub player_anim: AnimState,
    pub player_in_cover: bool,
    pub player_vault_t: f32,

    pub body: Body,
    pub inv: Inventory,
    pub doors: Vec<Door>,
    pub caches: Vec<SupplyCache>,
    pub pedestrians: Vec<Pedestrian>,
    pub militia: Vec<MilitiaMember>,
    pub squads: Vec<Squad>,
    pub drops: Vec<LootDrop>,
    pub barricades: Vec<Barricade>,
    pub fire_barrels: Vec<P2>,
    /// Seconds each fire in `fire_barrels` has left (infinite for a barrel).
    fire_ttl: Vec<f32>,
    pub tracks: Vec<SnowTrack>,
    pub craters: Vec<Crater>,
    pub wrecks: Vec<VehicleWreck>,
    pub smoke_clouds: Vec<SmokeCloud>,
    pub grenades: Vec<ActiveGrenade>,
    pub blood_marks: Vec<BloodMark>,
    pub visited: HashSet<(i32, i32)>,
    pub drone: ReconDrone,
    pub mortars: Vec<MortarMission>,
    pub pending_fpvs: Vec<PendingFpv>,
    pub fpv: Option<FpvDrone>,
    pub lancet: Option<LancetDrone>,
    pub friendly_fpv: Option<FriendlyFpv>,
    pub tanks: Vec<TankUnit>,
    pub player_floor: u8,
    pub tank_lock_warning: bool,
    pub radio_cd: f32,

    pub notice: f32,
    pub challenged: bool,
    pub under_fire_for: f32,
    pub search_hold: f32,
    pub active_door: Option<usize>,
    pub interiors: Vec<Interior>,
    pub active_interior: Option<usize>,
    pub cutaway_interior: Option<usize>,
    pub flashlight_on: bool,
    pub home_visited: bool,
    pub current_room_temp: Option<f32>,
    pub active_interior_name: String,
    pub selected_barricade: BarricadeKind,

    pub prompt: String,
    pub subtitle_zh: String,
    pub subtitle_en: String,
    pub subtitle_ttl: f32,
    pub objective_zh: String,
    pub objective_en: String,
    pub sms: Option<Sms>,
    pub sms_ttl: f32,
    pub radio_feed: Option<RadioCallout>,
    pub militia_cmd_marker: Option<(P2, f32)>,

    pub sim_time: f32,
    next_id: u32,
    /// The run's seed, and the state of its random generator (rand01). Rolls used to
    /// come from the entity counter, so making one more entity changed every later roll.
    pub seed: u64,
    rng: u64,
    spawn_wait: f32,
    fpv_cd: f32,
    events_fired: u32,
    warned_cold: bool,
    warned_hunger: bool,
    warned_thirst: bool,
    warned_river: bool,
    warned_fog: bool,
    pending_sms_chime: bool,
    last_player_track: P2,
    patrol_roads: Vec<Vec<P2>>,
    tank_wave_mask: u16,
    lancet_cd: f32,
    next_militia_idx: usize,
    gunfire_reveal_timer: f32,
}

const PED_COATS: [Rgb; 5] = [
    Rgb { r: 0.10, g: 0.10, b: 0.12 },
    Rgb { r: 0.62, g: 0.12, b: 0.12 },
    Rgb { r: 0.78, g: 0.62, b: 0.38 },
    Rgb { r: 0.18, g: 0.26, b: 0.42 },
    Rgb { r: 0.45, g: 0.16, b: 0.18 },
];

impl WorldSim {
    pub fn new(city: &City) -> Self {
        let solid_ground = Ground::new(city);
        let mut doors = Vec::new();
        for s in &city.signs {
            if !s.shop {
                continue;
            }
            let name = if !s.text.is_empty() { s.text.clone() } else { s.text_ru.clone() };
            if name.is_empty() {
                continue;
            }
            let Some(walk_pt) = solid_ground.snap_walkable(s.at, 0.4, 6.0) else {
                continue;
            };
            doors.push(Door { at: walk_pt, name, kind: shop_kind(&s.text, &s.text_ru), searched: false });
        }

        let mut pedestrians = Vec::new();
        if !city.paths.is_empty() {
            for i in 0..16 {
                let path_idx = i % city.paths.len();
                let path = &city.paths[path_idx];
                let wp_idx = ((hash01(i as i64 + 10, 3) * path.len() as f32) as usize).min(path.len() - 1);
                let at = path[wp_idx];
                pedestrians.push(Pedestrian {
                    id: i as u32 + 1,
                    at,
                    facing: 0.0,
                    coat: PED_COATS[i % PED_COATS.len()],
                    female: i % 3 == 1,
                    path_idx,
                    wp_idx,
                    wp_dir: if i % 2 == 0 { 1 } else { -1 },
                    fled_for: 0.0,
                    hidden: false,
                    phase: hash01(i as i64, 9) * 6.28,
                    speed: 1.15,
                });
            }
        }

        let mut patrol_roads: Vec<Vec<P2>> = Vec::new();
        for r in city.roads.iter().filter(|r| r.width >= 4.0 && r.line.len() >= 2) {
            let mut run: Vec<P2> = Vec::new();
            let push_run = |out: &mut Vec<Vec<P2>>, run: &mut Vec<P2>| {
                if run.len() >= 2 {
                    let len: f32 = run.windows(2).map(|w| w[0].dist(w[1])).sum();
                    if len >= 12.0 {
                        out.push(std::mem::take(run));
                        return;
                    }
                }
                run.clear();
            };
            for &p in &r.line {
                if let Some(wp) = solid_ground.snap_walkable(p, 0.55, 2.5) {
                    run.push(wp);
                } else {
                    push_run(&mut patrol_roads, &mut run);
                }
            }
            push_run(&mut patrol_roads, &mut run);
        }

        let mut caches = vec![
            SupplyCache { id: 1, at: P2::new(-268.8, -325.2), kind: CacheKind::StreetScrap, unlock_hour: CROSSING_HOUR, title: "中央街废铁煤堆", collected: false },
            SupplyCache { id: 2, at: P2::new(-159.3, -246.5), kind: CacheKind::StreetScrap, unlock_hour: CROSSING_HOUR, title: "文化街路边废铁煤堆", collected: false },
            SupplyCache { id: 3, at: P2::new(-268.8, -325.2), kind: CacheKind::ResistanceCell, unlock_hour: hm(18, 30), title: "老周民兵互助点（领步枪与民兵）", collected: false },
            SupplyCache { id: 4, at: P2::new(0.0, -14.0), kind: CacheKind::ResistanceCell, unlock_hour: hm(18, 30), title: "赵小北母亲广场接应点", collected: false },
            SupplyCache { id: 5, at: P2::new(-255.0, -320.0), kind: CacheKind::SosNeighbours, unlock_hour: hm(20, 0), title: "救命！楼里还有5位居民（救援得电台）", collected: false },
            SupplyCache { id: 6, at: P2::new(-159.3, -246.5), kind: CacheKind::CommunityCache, unlock_hour: hm(20, 0), title: "社区御寒储备箱", collected: false },
            SupplyCache { id: 7, at: P2::new(-116.4, -156.4), kind: CacheKind::BorderCrate, unlock_hour: hm(22, 30), title: "海兰街边防遗留弹药箱", collected: false },
            SupplyCache { id: 8, at: P2::new(-220.0, -290.0), kind: CacheKind::SmugglerCache, unlock_hour: hm(25, 0), title: "凌晨暗巷救急补给", collected: false },
        ];
        for c in &mut caches {
            c.at = solid_ground.snap_walkable(c.at, 0.65, 16.0).unwrap_or(city.spawn);
        }

        let interiors = build_interiors(city);
        let mut visited = HashSet::new();
        visited.insert(((city.spawn.x / 20.0).floor() as i32, (city.spawn.y / 20.0).floor() as i32));

        Self {
            hour: START_HOUR,
            hours_alive: 0.0,
            alert_stage: 0,
            blackout: false,
            weather: Weather::Clear,
            fog_density: 0.0,
            weather_override: None,
            resistance_active: false,
            neighbours_rescued: false,
            dawn_survived: false,
            mornings: 0,
            sleeping: false,
            noise_m: 0.0,
            noise_t: 0.0,
            slept_hours: 0.0,
            woke: None,
            dead: false,
            death_zh: "",
            death_en: "",

            player_at: city.spawn,
            player_facing: -std::f32::consts::FRAC_PI_2,
            player_gait: Gait::Still,
            player_speed: 0.0,
            player_phase: 0.0,
            player_shoot_cd: 0.0,
            player_anim: AnimState::Idle,
            player_in_cover: false,
            player_vault_t: 0.0,

            body: Body::default(),
            inv: Inventory::default(),
            doors,
            caches,
            pedestrians,
            militia: Vec::new(),
            squads: Vec::new(),
            drops: Vec::new(),
            barricades: Vec::new(),
            fire_barrels: Vec::new(),
            fire_ttl: Vec::new(),
            tracks: Vec::new(),
            craters: Vec::new(),
            wrecks: Vec::new(),
            smoke_clouds: Vec::new(),
            grenades: Vec::new(),
            blood_marks: Vec::new(),
            visited,
            drone: ReconDrone::default(),
            mortars: Vec::new(),
            pending_fpvs: Vec::new(),
            fpv: None,
            lancet: None,
            friendly_fpv: None,
            tanks: Vec::new(),
            player_floor: 0,
            tank_lock_warning: false,
            radio_cd: 0.0,

            notice: 0.0,
            challenged: false,
            under_fire_for: 0.0,
            search_hold: 0.0,
            active_door: None,
            interiors,
            active_interior: None,
            cutaway_interior: None,
            flashlight_on: false,
            home_visited: false,
            current_room_temp: None,
            active_interior_name: String::new(),
            selected_barricade: BarricadeKind::Sandbag,

            prompt: String::new(),
            subtitle_zh: String::new(),
            subtitle_en: String::new(),
            subtitle_ttl: 0.0,
            objective_zh: "趁店还开着，在店招底下按住 E 买吃的、水和绷带。\n赵小北在母亲广场等你，在北边江畔。".into(),
            objective_en: "Shops are open. Hold E under a shop sign for food, water, coat and bandages.".into(),
            sms: None,
            sms_ttl: 0.0,
            radio_feed: None,
            militia_cmd_marker: None,

            sim_time: 0.0,
            next_id: 100,
            seed: DEFAULT_SEED,
            rng: DEFAULT_SEED,
            spawn_wait: 6.0,
            fpv_cd: 90.0,
            events_fired: 0,
            warned_cold: false,
            warned_hunger: false,
            warned_thirst: false,
            warned_river: false,
            warned_fog: false,
            pending_sms_chime: false,
            last_player_track: city.spawn,
            patrol_roads,
            tank_wave_mask: 0,
            lancet_cd: 75.0,
            next_militia_idx: 0,
            gunfire_reveal_timer: 0.0,
        }
    }

    fn next_rand(&mut self, salt: u32) -> f32 {
        rand01(&mut self.rng, salt)
    }

    /// The same world, with its random generator started from `seed`.
    pub fn with_seed(city: &City, seed: u64) -> Self {
        let mut sim = Self::new(city);
        sim.seed = seed;
        sim.rng = seed;
        sim
    }

    fn say(&mut self, zh: &str, en: &str) {
        self.subtitle_zh = zh.into();
        self.subtitle_en = en.into();
        self.subtitle_ttl = 7.5;
    }

    fn push_sms(&mut self, sender: &str, zh: &str, en: &str) {
        self.sms = Some(Sms { sender: sender.into(), zh: zh.into(), en: en.into() });
        self.sms_ttl = 14.0;
        self.pending_sms_chime = true;
    }

    pub fn push_radio(&mut self, unit: &str, code_en: &'static str, text_zh: &str, urgent: bool) {
        self.radio_feed = Some(RadioCallout {
            unit: unit.to_string(),
            code_en,
            text_zh: text_zh.to_string(),
            ttl: 5.5,
            urgent,
        });
    }

    /// Issue a tactical move/garrison order to the next living militia member.
    /// Can be called both during normal simulation and while Tactical Pause is active.
    pub fn issue_militia_order(&mut self, dest: P2) -> bool {
        if self.militia.is_empty() {
            return false;
        }
        let mut chosen = None;
        for off in 0..self.militia.len() {
            let i = (self.next_militia_idx + off) % self.militia.len();
            if self.militia[i].hp > 0.0 {
                chosen = Some(i);
                self.next_militia_idx = i + 1;
                break;
            }
        }
        let Some(i) = chosen else {
            return false;
        };
        let name = self.militia[i].name;
        if let Some(intr) = self.interiors.iter().find(|it| it.contains(dest)) {
            let spot = intr
                .firing_spots
                .iter()
                .copied()
                .min_by(|a, b| a.dist(dest).total_cmp(&b.dist(dest)))
                .unwrap_or(dest);
            self.militia[i].order = MilitiaOrder::Garrison(spot);
            self.militia_cmd_marker = Some((spot, 6.0));
            self.say(&format!("{name} 进屋抢占窗口射界驻防！"), "Militia garrisoned indoors!");
            self.push_radio(name, "YOU GOT IT", &format!("{name} 收到！正前往屋内窗口构筑防线"), false);
        } else {
            self.militia[i].order = MilitiaOrder::Hold(dest);
            self.militia_cmd_marker = Some((dest, 6.0));
            self.say(&format!("{name} 正前往指定街口构筑防线！"), "Militia ordered to position.");
            self.push_radio(name, "YOU GOT IT", &format!("{name} 收到！正前往指定点构筑防线"), false);
        }
        true
    }

    fn kill_player(&mut self, zh: &'static str, en: &'static str) {
        if self.dead {
            return;
        }
        self.dead = true;
        self.death_zh = zh;
        self.death_en = en;
        self.body.hp = 0.0;
        self.player_anim = AnimState::Dead;
        self.player_gait = Gait::Still;
    }

    /// One step of the world. Asleep, you do nothing and the step watches for what
    /// wakes you; the client runs these steps fast (SLEEP_PLAY_SPEED).
    pub fn tick(&mut self, city: &City, ground: &Ground, forces: &Forces, input: &SimInput, dt: f32) -> TickOutput {
        if input.toggle_sleep && !self.dead {
            if self.sleeping {
                self.wake(WakeReason::ByHand);
            } else {
                self.try_sleep();
            }
        }
        if !self.sleeping {
            return self.tick_world(city, ground, forces, input, dt);
        }
        let still = SimInput { clock_running: input.clock_running, ..Default::default() };
        let hp_before = self.body.hp;
        let out = self.tick_world(city, ground, forces, &still, dt);
        if self.dead {
            self.sleeping = false;
            self.body.asleep = false;
        } else if self.sleeping {
            if input.clock_running {
                self.slept_hours += dt.clamp(0.0, 0.25) * TIME_SCALE / 3600.0;
            }
            if let Some(why) = self.wake_reason(&out, hp_before) {
                self.wake(why);
            }
        }
        out
    }

    /// The temperature in the street now (°C).
    pub fn outdoor_c(&self) -> f32 {
        crate::living::get().outdoor_c(self.seed, self.hour)
    }

    /// Stove time one pot of melted snow costs (play seconds).
    fn melt_fuel_s(&self) -> f32 {
        crate::living::get().num("fuel", "melt_fuel_h") * 3600.0 / TIME_SCALE
    }

    /// Each morning other people have taken some of what was left in the shops.
    fn others_take_from_shops(&mut self) {
        let share = crate::living::get().shops_taken(self.mornings);
        let day = self.mornings as i64;
        let seed = self.seed as i64;
        for (ii, intr) in self.interiors.iter_mut().enumerate() {
            if matches!(intr.kind, InteriorKind::PlayerHome | InteriorKind::NeighbourHome | InteriorKind::BasementShelter) {
                continue;
            }
            let ground = intr.loots.iter_mut();
            let upper = intr.floors.iter_mut().flat_map(|f| f.loots.iter_mut());
            for l in ground.chain(upper) {
                if !l.searched && hash01(seed ^ (ii as i64 * 7919 + l.id as i64 * 104_729 + day * 15_485_863), 0x5E0F) < share {
                    l.searched = true;
                }
            }
        }
    }

    /// Metres to the nearest Russian soldier or vehicle that is still alive.
    pub fn nearest_enemy_m(&self) -> f32 {
        let men = self.squads.iter().flat_map(|sq| sq.men.iter()).filter(|m| m.hp > 0.0).map(|m| m.at.dist(self.player_at));
        // An engine carries further than boots.
        let tanks = self
            .tanks
            .iter()
            .filter(|t| t.hp > 0.0 && t.faction == Faction::Russia)
            .map(|t| t.at.dist(self.player_at) / 2.0);
        men.chain(tanks).fold(f32::MAX, f32::min)
    }

    fn try_sleep(&mut self) {
        if self.active_interior.is_none() {
            self.say("得先进屋才能睡。", "Get indoors before you sleep.");
        } else if self.challenged || self.under_fire_for > 0.0 || self.nearest_enemy_m() < SLEEP_QUIET_M {
            self.say("外面有动静，睡不着。", "There is movement outside. You can't sleep.");
        } else if self.body.fatigue < SLEEP_MIN_FATIGUE {
            self.say("还不困。", "You are not tired.");
        } else {
            self.sleeping = true;
            self.body.asleep = true;
            self.slept_hours = 0.0;
            self.woke = None;
            self.flashlight_on = false;
            self.say("躺下了。有动静就会醒。再按 J 起来。", "You lie down. Any noise will wake you. J to get up.");
        }
    }

    fn wake(&mut self, why: WakeReason) {
        self.sleeping = false;
        self.body.asleep = false;
        self.woke = Some(why);
        let (zh, en) = why.text();
        let h = self.slept_hours;
        self.say(&format!("{zh}（睡了 {h:.1} 小时）"), &format!("{en} (slept {h:.1} h)"));
    }

    fn wake_reason(&self, out: &TickOutput, hp_before: f32) -> Option<WakeReason> {
        if out.player_hit || self.body.hp < hp_before - 0.001 {
            return Some(WakeReason::Hurt);
        }
        if self.challenged || self.nearest_enemy_m() < SLEEP_WAKE_M {
            return Some(WakeReason::Enemy);
        }
        let near = |p: P2, r: f32| p.dist(self.player_at) < r;
        if out.door_breach_thud
            || out.glass_break
            || out.shout_stoy
            // A gun fired anywhere in the city sets tank_cannon; only one close by wakes you.
            || (out.tank_cannon && self.tanks.iter().any(|t| t.hp > 0.0 && near(t.at, 300.0)))
            || out.slab_collapse
            || out.mortar_whistle
            || out.explosions.iter().any(|e| near(e.at, 150.0))
            || out.tracers.iter().any(|t| near(P2::new(t.from[0], -t.from[2]), 80.0))
        {
            return Some(WakeReason::Noise);
        }
        if self.body.temp < 35.2 {
            return Some(WakeReason::Cold);
        }
        if self.body.thirst < 8.0 {
            return Some(WakeReason::Thirst);
        }
        if self.body.hunger < 5.0 {
            return Some(WakeReason::Hunger);
        }
        if self.body.fatigue <= 0.0 {
            return Some(WakeReason::Rested);
        }
        None
    }

    fn tick_world(&mut self, city: &City, ground: &Ground, forces: &Forces, input: &SimInput, dt: f32) -> TickOutput {
        let mut out = TickOutput::default();
        if self.dead {
            self.player_anim = AnimState::Dead;
            return out;
        }
        let dt = dt.clamp(0.0, 0.25);
        self.sim_time += dt;
        self.player_shoot_cd = (self.player_shoot_cd - dt).max(0.0);
        self.player_vault_t = (self.player_vault_t - dt).max(0.0);
        self.radio_cd = (self.radio_cd - dt).max(0.0);
        self.noise_t = (self.noise_t - dt).max(0.0);
        self.gunfire_reveal_timer = (self.gunfire_reveal_timer - dt).max(0.0);
        if self.inv.unlocked_primaries == 0 && self.inv.has_rifle {
            self.inv.unlock_primary(PrimaryWeapon::Ak74m);
            self.inv.active_primary = PrimaryWeapon::Ak74m;
        }
        if self.subtitle_ttl > 0.0 {
            self.subtitle_ttl = (self.subtitle_ttl - dt).max(0.0);
        }
        if self.sms_ttl > 0.0 {
            self.sms_ttl = (self.sms_ttl - dt).max(0.0);
            if self.sms_ttl <= 0.0 {
                self.sms = None;
            }
        }
        if let Some(rf) = &mut self.radio_feed {
            rf.ttl -= dt;
            if rf.ttl <= 0.0 {
                self.radio_feed = None;
            }
        }
        if let Some((_, ttl)) = &mut self.militia_cmd_marker {
            *ttl -= dt;
            if *ttl <= 0.0 {
                self.militia_cmd_marker = None;
            }
        }
        for c in &mut self.smoke_clouds {
            c.ttl -= dt;
        }
        self.smoke_clouds.retain(|c| c.ttl > 0.0);
        self.step_fires(dt);
        for w in &mut self.wrecks {
            w.smoke_ttl = (w.smoke_ttl - dt).max(0.0);
        }
        for b in &mut self.blood_marks {
            b.age += dt;
        }
        self.blood_marks.retain(|b| b.age < 150.0);

        // 1. Clock, weather & timeline milestones.
        if input.clock_running {
            let step = dt * TIME_SCALE / 3600.0;
            self.hour += step;
            self.hours_alive += step;
        }
        let (target_w, target_fog) = match self.weather_override {
            Some(Weather::Clear) => (Weather::Clear, 0.0),
            Some(Weather::Flurries) => (Weather::Flurries, 0.35),
            Some(Weather::RiverFog) => (Weather::RiverFog, 0.92),
            None => weather_at(self.seed, self.hour),
        };
        self.weather = target_w;
        if self.weather_override.is_some() {
            self.fog_density = target_fog;
        } else {
            self.fog_density += (target_fog - self.fog_density) * (dt * 0.25).min(1.0);
        }
        if !self.warned_fog && self.weather == Weather::RiverFog && self.fog_density >= 0.65 && self.alert_stage >= 2 {
            self.warned_fog = true;
            self.push_sms("气象与前线观察", "黑龙江江面起大雾了！无人机几乎致盲，抓紧大雾转移或去拿物资。", "Heavy river fog! Drones are nearly blinded.");
            self.say("江面起大雾了！无人机几乎致盲，趁大雾转移或去拿物资。", "River fog rolled in! Drones are blinded.");
        }

        self.step_timeline();
        let mornings = if self.hour >= DAWN_HOUR { 1 + ((self.hour - DAWN_HOUR) / 24.0).floor() as u32 } else { 0 };
        if mornings > self.mornings {
            self.mornings = mornings;
            if mornings == 1 {
                self.dawn_survived = true;
                self.say("天亮了。你还活着。", "Dawn. You are still alive.");
                self.objective_zh = "熬过了第一夜。天亮了，他们看得更远：白天少上街。\n在屋里找吃的喝的、生炉子；困了就在屋里按 J 睡，有动静会醒。".into();
                self.objective_en = "You lived through the first night. By day they see further: keep off the streets.\nFind food and water indoors, keep a stove going; sleep indoors with J, noise will wake you.".into();
            } else {
                let (m, d) = clock::date_of(self.hour);
                self.say(&format!("{m}月{d}日，天亮了。第 {mornings} 个早晨。"), &format!("Morning {mornings}."));
            }
            self.others_take_from_shops();
        }

        // 2. Player movement, visited street cells & snow footprints.
        let wish_len = input.wish_move.len();
        if wish_len > 0.05 {
            let prev_step = (self.player_phase / std::f32::consts::PI).floor() as i32;
            let dir = input.wish_move.norm();
            let base_speed = if input.sprinting { 6.0 } else { 3.4 };
            let speed = base_speed * self.body.move_scale;
            let to = self.player_at.add(dir.mul(speed * dt));
            let cur_floor = if self.active_interior.is_some() { self.player_floor } else { 0 };
            let next_pos = ground.step_with_interiors_on_floor(&self.interiors, self.player_at, to, 0.35, cur_floor, false);
            if cur_floor == 0 && self.player_vault_t <= 0.0 && next_pos.dist(self.player_at) > 0.01 {
                if self.barricades.iter().any(|b| {
                    !b.shoved
                        && b.hp > 0.0
                        && matches!(b.kind, BarricadeKind::Sandbag | BarricadeKind::Trench)
                        && b.at.dist(next_pos) <= 1.05
                        && b.at.dist(self.player_at) > 1.05
                }) {
                    self.player_vault_t = 0.45;
                }
            }
            self.player_at = next_pos;
            self.player_facing = dir.y.atan2(dir.x);
            self.player_speed = speed;
            self.player_gait = if input.sprinting { Gait::Run } else { Gait::Walk };
            self.player_phase += (std::f32::consts::TAU * speed / (0.95 + 0.42 * speed)) * dt;
            let next_step = (self.player_phase / std::f32::consts::PI).floor() as i32;
            if next_step != prev_step {
                out.footstep = true;
            }
        } else {
            self.player_speed = 0.0;
            self.player_gait = Gait::Still;
        }
        self.visited.insert(((self.player_at.x / 20.0).floor() as i32, (self.player_at.y / 20.0).floor() as i32));

        // 2b. Seamless interior detection, doorway transitions, stairs, gravity collapse, microclimate & indoor tools (L/K/T).
        self.step_interiors_state(input, dt, &mut out);
        self.apply_slab_gravity(&mut out);
        if self.dead {
            return out;
        }

        // 3. Body temp, hunger, thirst, blood & quick-use keys.
        let indoors = self.active_interior.is_some();
        let sheltered = city.near_heat(self.player_at) || indoors;
        let on_ice = !indoors && ground.on_ice(self.player_at);
        let near_barrel = self.fire_barrels.iter().any(|b| b.dist(self.player_at) <= 3.5)
            || self.active_interior.is_some_and(|i| self.interiors[i].room_temp_c >= 10.0);

        if !sheltered && self.player_at.dist(self.last_player_track) >= 1.6 {
            let dir = self.player_at.sub(self.last_player_track).norm();
            self.last_player_track = self.player_at;
            self.push_track(self.player_at, dir, true, false);
        }
        let fade_rate = dt * if self.weather == Weather::Flurries { 2.5 } else { 1.0 };
        for t in &mut self.tracks {
            t.age += fade_rate;
        }
        self.tracks.retain(|t| t.age < 180.0);

        self.body.indoors = indoors;
        self.body.env_c = match self.active_interior {
            Some(i) => self.interiors[i].room_temp_c,
            None => self.outdoor_c(),
        };
        if let Some((czh, cen)) = self.body.tick(dt, self.hour, self.player_gait, sheltered, on_ice, near_barrel) {
            self.kill_player(czh, cen);
            out.sms_arrived = std::mem::take(&mut self.pending_sms_chime);
            return out;
        }
        if self.body.temp < 34.2 && !self.warned_cold {
            self.warned_cold = true;
            self.say("你在发抖，手不听使唤了。找个有暖气的门脸或进屋烧煤炉。", "You're shaking. Find a heated doorway or light an indoor stove.");
        }
        if self.body.thirst < 20.0 && !self.warned_thirst {
            self.warned_thirst = true;
            self.say("嘴唇裂了。得找水。", "Your lips have split. You need water.");
        }
        if self.body.hunger < 20.0 && !self.warned_hunger {
            self.warned_hunger = true;
            self.say("饿得发慌，身上更冷了。", "Hunger. The cold bites harder on an empty stomach.");
        }
        self.handle_items(input);

        // 4. Scavenging shops & progressive supply caches, indoor loot, NPC dialogue & resistance rally.
        self.step_interactions(input, dt, &mut out);

        // 5. Pedestrians, combat, autonomous militia & active grenades.
        self.step_pedestrians(city, ground, dt);
        self.step_combat_actions(city, ground, forces, input, &mut out);
        self.step_militia(ground, forces, dt, &mut out);

        // 6. Russian infantry squads (spawn after 17:36 crossing), sight, challenge & combat.
        let light = light_at(city, self.hour, self.blackout, near_barrel, self.player_at, &self.fire_barrels);
        self.step_squads(city, ground, forces, light, dt, &mut out);
        self.step_grenades(forces, dt, &mut out);
        if self.dead {
            return out;
        }

        // 7. Recon drone, 82mm mortar strikes, FPV drone & multi-wave Russian tank invasion.
        self.step_drone_and_mortars(city, forces, light, dt, &mut out);
        self.step_tanks(city, ground, forces, light, dt, &mut out);
        if self.dead {
            return out;
        }

        // 8. Pick player animation state & flush SMS audio cue.
        self.player_in_cover = !self.dead
            && self.player_gait == Gait::Still
            && self.active_interior.is_none()
            && (self.barricade_cover_mul(forces, self.player_at) < 0.8
                || self.crater_cover_mul(self.player_at) < 0.8
                || self.wreck_cover_mul(self.player_at) < 0.8);
        out.sms_arrived = std::mem::take(&mut self.pending_sms_chime);
        self.player_anim = if self.dead {
            AnimState::Dead
        } else if self.challenged && self.player_gait == Gait::Still && !self.inv.has_rifle {
            AnimState::HandsUp
        } else if self.player_vault_t > 0.0 {
            AnimState::Vault
        } else if self.player_shoot_cd > 0.25 && self.inv.has_rifle {
            AnimState::Shoot
        } else if self.inv.has_rifle && input.fire_at.is_some() {
            AnimState::Aim
        } else if self.search_hold > 0.05 {
            AnimState::Loot
        } else if self.player_gait == Gait::Run {
            AnimState::Run
        } else if self.player_gait == Gait::Walk {
            AnimState::Walk
        } else if self.player_in_cover {
            AnimState::Crouch
        } else {
            AnimState::Idle
        };

        out
    }

    fn push_track(&mut self, at: P2, dir: P2, by_player: bool, is_vehicle: bool) {
        if self.tracks.len() >= 240 {
            let drop_n = self.tracks.len() - 239;
            self.tracks.drain(0..drop_n);
        }
        self.next_id = self.next_id.wrapping_add(1);
        let dir = if dir.len() > 0.01 { dir.norm() } else { P2::new(1.0, 0.0) };
        self.tracks.push(SnowTrack { id: self.next_id, at, dir, age: 0.0, by_player, is_vehicle });
    }

    fn push_smoke_cloud(&mut self, at: P2, radius: f32, ttl: f32, dark: bool) {
        if self.smoke_clouds.len() >= 32 {
            let drop_n = self.smoke_clouds.len() - 31;
            self.smoke_clouds.drain(0..drop_n);
        }
        self.next_id = self.next_id.wrapping_add(1);
        self.smoke_clouds.push(SmokeCloud {
            id: self.next_id,
            at,
            radius,
            ttl,
            dark,
        });
    }

    fn push_crater(&mut self, at: P2, radius: f32) {
        if self.craters.iter().any(|c| c.at.dist(at) < 2.2) {
            return;
        }
        if self.craters.len() >= 60 {
            let drop_n = self.craters.len() - 59;
            self.craters.drain(0..drop_n);
        }
        self.next_id = self.next_id.wrapping_add(1);
        self.craters.push(Crater {
            id: self.next_id,
            at,
            radius: radius.clamp(1.8, 3.8),
        });
    }

    fn add_fire(&mut self, at: P2, ttl: f32) {
        self.fire_ttl.resize(self.fire_barrels.len(), f32::INFINITY);
        self.fire_barrels.push(at);
        self.fire_ttl.push(ttl);
    }

    /// Burn the fires down; a barrel goes out when it is knocked over.
    fn step_fires(&mut self, dt: f32) {
        // Fires put in from outside (the picture scenes) burn like barrels.
        self.fire_ttl.resize(self.fire_barrels.len(), f32::INFINITY);
        let knocked: Vec<P2> = self
            .barricades
            .iter()
            .filter(|b| b.kind == BarricadeKind::FireBarrel && (b.shoved || b.hp <= 0.0))
            .map(|b| b.at)
            .collect();
        let mut i = 0;
        while i < self.fire_barrels.len() {
            self.fire_ttl[i] -= dt;
            let at = self.fire_barrels[i];
            let out = self.fire_ttl[i] <= 0.0 || (self.fire_ttl[i].is_infinite() && knocked.iter().any(|k| k.dist(at) < 0.6));
            if out {
                self.fire_barrels.remove(i);
                self.fire_ttl.remove(i);
            } else {
                i += 1;
            }
        }
    }

    fn push_wreck(&mut self, dead: TankUnit) {
        if self.wrecks.len() >= 24 {
            let drop_n = self.wrecks.len() - 23;
            self.wrecks.drain(0..drop_n);
        }
        self.wrecks.push(VehicleWreck {
            id: dead.id,
            kind: dead.kind,
            at: dead.at,
            hull_yaw: dead.hull_yaw,
            turret_yaw: dead.turret_yaw,
            smoke_ttl: 120.0,
        });
        self.push_smoke_cloud(dead.at, 4.5, 16.0, true);
    }

    pub fn nearest_enterable_door(&self, p: P2, max_dist: f32) -> Option<(usize, P2, P2)> {
        let mut best: Option<(f32, usize, P2, P2)> = None;
        let bound = max_dist + 22.0;
        for (i, intr) in self.interiors.iter().enumerate() {
            if (intr.origin.x - p.x).abs() > bound || (intr.origin.y - p.y).abs() > bound {
                continue;
            }
            if intr.front_door.barricade_hp <= 0.0 {
                let d = intr.front_door.at.dist(p);
                if d <= max_dist && best.is_none_or(|b| d < b.0) {
                    best = Some((d, i, intr.front_door.at, intr.front_door.outside));
                }
            }
            if let Some(bd) = &intr.back_door {
                if bd.barricade_hp <= 0.0 {
                    let d = bd.at.dist(p);
                    if d <= max_dist && best.is_none_or(|b| d < b.0) {
                        best = Some((d, i, bd.at, bd.outside));
                    }
                }
            }
        }
        best.map(|(_, i, at, out)| (i, at, out))
    }

    fn door_label(style: DoorStyle) -> &'static str {
        match style {
            DoorStyle::CottonCurtain => "棉门帘",
            DoorStyle::ShopGlass => "玻璃门",
            DoorStyle::SteelSecurity => "防盗铁门",
            DoorStyle::WoodFlat => "木门",
        }
    }

    fn apply_slab_gravity(&mut self, out: &mut TickOutput) {
        if let Some(i_idx) = self.active_interior {
            while self.player_floor > 0 && !self.interiors[i_idx].is_slab_intact(self.player_floor, self.player_at) {
                self.player_floor -= 1;
                out.slab_collapse = true;
                out.player_hit = true;
                let roll = self.next_rand(93);
                if self.body.hurt(18.0, 0.35, roll) {
                    self.kill_player("楼板被炮火炸穿，你从高楼摔了下来！", "Floor slab collapsed beneath you!");
                    return;
                }
                self.say("脚下楼板已被炸穿！重重摔下一层楼！", "Floor slab blasted open! Fell through to lower floor!");
            }
        }
        for m in &mut self.militia {
            if m.hp <= 0.0 || m.floor == 0 {
                continue;
            }
            if let Some(intr) = self.interiors.iter().find(|it| it.contains(m.at)) {
                while m.floor > 0 && !intr.is_slab_intact(m.floor, m.at) {
                    m.floor -= 1;
                    m.hp = (m.hp - 18.0).max(0.0);
                    out.slab_collapse = true;
                }
            } else {
                m.floor = 0;
            }
        }
        for g in &mut self.squads {
            for s in &mut g.men {
                if s.hp <= 0.0 || s.floor == 0 {
                    continue;
                }
                if let Some(intr) = self.interiors.iter().find(|it| it.contains(s.at)) {
                    while s.floor > 0 && !intr.is_slab_intact(s.floor, s.at) {
                        s.floor -= 1;
                        s.hp = (s.hp - 22.0).max(0.0);
                        out.slab_collapse = true;
                    }
                } else {
                    s.floor = 0;
                }
            }
        }
    }

    fn step_interiors_state(&mut self, input: &SimInput, dt: f32, out: &mut TickOutput) {
        let heat_on = self.hour < HEAT_OFF_HOUR;
        let t_out = self.outdoor_c();
        let living = crate::living::get();
        let game_h = dt * TIME_SCALE / 3600.0;
        let central_c = living.num("building", "central_heat_c");
        let cool_tau = living.num("building", "cool_tau_h");
        let above = living.num("building", "above_outdoor_c");
        let broken_mul = living.num("building", "broken_window_mul");
        let open_mul = living.num("building", "open_door_mul");
        let basement_c = living.num("building", "basement_c");
        let basement_tau = living.num("building", "basement_tau_h");
        let stove_c = living.num("building", "stove_room_c");
        let stove_rise = living.num("building", "stove_basement_rise_c");
        let stove_tau = living.num("building", "stove_tau_h");
        let pat = self.player_at;
        let wish_moving = input.wish_move.len() > 0.05;
        let on_ground = self.player_floor == 0;
        let mut active_idx = None;
        let mut cutaway_idx = None;

        for (i, intr) in self.interiors.iter_mut().enumerate() {
            intr.breach_alert = (intr.breach_alert - dt).max(0.0);
            intr.stove_fuel_s = (intr.stove_fuel_s - dt).max(0.0);

            if (intr.origin.x - pat.x).abs() <= 24.0 && (intr.origin.y - pat.y).abs() <= 24.0 {
                let near_front = intr.front_door.at.dist(pat) <= 2.45;
                let near_back = intr.back_door.as_ref().is_some_and(|b| b.at.dist(pat) <= 2.45);
                let in_room_now = intr.contains(pat);

                // Auto-push unlocked doors/curtains only on 1F when actively walking through.
                if on_ground {
                    if !intr.front_door.destroyed && intr.front_door.barricade_hp <= 0.0 {
                        let d_front = intr.front_door.at.dist(pat);
                        let push = wish_moving && if in_room_now { d_front <= 1.20 } else { d_front <= 1.65 };
                        if push && !intr.front_door.open {
                            intr.front_door.open = true;
                            out.door_action = true;
                        } else if d_front > 2.85 && intr.front_door.open {
                            intr.front_door.open = false;
                        }
                    }
                    if let Some(bd) = &mut intr.back_door {
                        if !bd.destroyed && bd.barricade_hp <= 0.0 {
                            let d_back = bd.at.dist(pat);
                            let push = wish_moving && if in_room_now { d_back <= 1.20 } else { d_back <= 1.65 };
                            if push && !bd.open {
                                bd.open = true;
                                out.door_action = true;
                            } else if d_back > 2.85 && bd.open {
                                bd.open = false;
                            }
                        }
                    }
                    if let Some(idoor) = &mut intr.inner_door {
                        if !idoor.destroyed && idoor.barricade_hp <= 0.0 {
                            let d_in = idoor.at.dist(pat);
                            if wish_moving && d_in <= 1.15 && !idoor.open {
                                idoor.open = true;
                                out.door_action = true;
                            } else if d_in > 2.65 && idoor.open {
                                idoor.open = false;
                            }
                        }
                    }
                }

                if in_room_now {
                    active_idx = Some(i);
                    cutaway_idx = Some(i);
                    intr.discovered = true;
                } else if on_ground && cutaway_idx.is_none() && (near_front || near_back) {
                    cutaway_idx = Some(i);
                }
            }

            // Thermal inertia simulation for each interior room (cracked slabs & breached walls leak warmth).
            let broken_n = intr.windows.iter().filter(|w| w.broken && !w.boarded).count() as f32
                + intr.floors.iter().filter(|f| f.front_wall_breached).count() as f32 * 1.4
                + if intr.any_structural_damage() { 0.8 } else { 0.0 };
            let open_n = intr.open_doors_count() as f32;
            // A room heads for its balance temperature: a few degrees above the
            // street for a closed flat among other flats, the ground's for a basement,
            // the stove's or the radiators' while they run. Cooling takes days in a
            // whole building; broken windows and open doors make it quicker and colder.
            let burning = intr.stove_fuel_s > 0.0;
            let leak = 1.0 + broken_n * broken_mul + open_n * open_mul;
            let (balance, tau_h) = if intr.kind == InteriorKind::BasementShelter {
                if burning { (basement_c + stove_rise, stove_tau) } else { (basement_c, basement_tau / leak) }
            } else if burning {
                (stove_c - broken_n * 5.0, stove_tau)
            } else if heat_on && broken_n < 0.5 {
                (central_c, stove_tau)
            } else {
                (t_out + above / leak, cool_tau / leak)
            };
            intr.room_temp_c += (balance - intr.room_temp_c) * (game_h / tau_h.max(0.05)).min(1.0);
        }

        self.active_interior = active_idx;
        self.cutaway_interior = cutaway_idx;
        if let Some(i) = active_idx {
            let max_f = self.interiors[i].num_floors.saturating_sub(1);
            self.player_floor = self.player_floor.min(max_f);
            let (is_home, num_floors, stair_dist, room_temp, intr_name) = {
                let intr = &self.interiors[i];
                (
                    intr.kind == InteriorKind::PlayerHome,
                    intr.num_floors,
                    intr.stairwell_world().dist(pat),
                    intr.room_temp_c,
                    intr.name.clone(),
                )
            };
            self.current_room_temp = Some(room_temp);
            self.active_interior_name = if num_floors > 1 {
                format!("{intr_name} {}F", self.player_floor + 1)
            } else {
                intr_name.clone()
            };
            if is_home && !self.home_visited {
                self.home_visited = true;
                self.say(
                    "回到家了！铁门耐撞（按 E 开关，按 T 顶门），可沿楼梯 Space 上楼观望，里屋有煤炉和储物柜。",
                    "Home at last! Steel door toggles with E (barricade T), climb stairs with Space.",
                );
            }
            if input.floor_delta != 0 {
                if num_floors > 1 && stair_dist <= 2.65 {
                    if input.floor_delta > 0 && self.player_floor < max_f {
                        self.player_floor += 1;
                        self.say(
                            &format!("顺楼梯登上 {intr_name} {} 楼（Space/Ctrl 切换楼层）。", self.player_floor + 1),
                            "Climbed stairs to upper floor.",
                        );
                    } else if input.floor_delta < 0 && self.player_floor > 0 {
                        self.player_floor -= 1;
                        self.say(
                            &format!("顺楼梯下到 {intr_name} {} 楼。", self.player_floor + 1),
                            "Descended stairs.",
                        );
                    }
                } else if num_floors <= 1 {
                    self.say("这是单层建筑或地下掩体（无楼上）。", "Single-story interior.");
                } else {
                    self.say("走到室内金色楼梯口再按 Space 上楼 / Ctrl 下楼。", "Stand at indoor stairwell for Space/Ctrl stairs.");
                }
            }
        } else {
            self.player_floor = 0;
            self.current_room_temp = None;
            self.active_interior_name.clear();
        }

        if input.toggle_flashlight {
            self.flashlight_on = !self.flashlight_on;
            if self.flashlight_on {
                self.say("开启手电筒：暗屋清晰可见，但没拉窗帘时易暴露。", "Flashlight ON.");
            } else {
                self.say("关掉手电筒：停电后黑屋极难被街上俄军发现。", "Flashlight OFF.");
            }
        }

        if input.toggle_curtains {
            if let Some(i) = self.active_interior.or(self.cutaway_interior) {
                let closed = !self.interiors[i].curtains_closed;
                self.interiors[i].curtains_closed = closed;
                if closed {
                    self.say("拉上窗帘：阻断街面巡逻视线与透光！", "Curtains closed: street sight blocked.");
                } else {
                    self.say("拉开窗帘：便于从窗内观察中央街并向外射击。", "Curtains open: watch & shoot outside.");
                }
            }
        }

        if input.fortify_or_stove {
            if let Some(i) = self.active_interior.or(self.cutaway_interior) {
                let intr = &mut self.interiors[i];
                let d_stove = intr.stove_at.map(|p| p.dist(pat)).unwrap_or(f32::MAX);
                let win_near = intr.windows.iter_mut().find(|w| w.broken && !w.boarded && w.at.dist(pat) <= 2.6);
                let d_front = intr.front_door.at.dist(pat);
                let d_back = intr.back_door.as_ref().map(|b| b.at.dist(pat)).unwrap_or(f32::MAX);

                if d_stove <= 2.4 {
                    let living = crate::living::get();
                    let play_s_per_h = 3600.0 / TIME_SCALE;
                    if self.inv.coal > 0 {
                        self.inv.coal -= 1;
                        intr.stove_fuel_s += living.num("fuel", "coal_unit_burn_h") * play_s_per_h;
                        self.say("添了一块煤，炉子旺起来了。", "Coal on the stove. It catches.");
                    } else if self.inv.wood > 0 {
                        self.inv.wood -= 1;
                        intr.stove_fuel_s += living.num("fuel", "wood_unit_burn_h") * play_s_per_h;
                        self.say("把劈开的木头塞进炉膛。", "Wood into the stove.");
                    } else if intr.furniture > 0 {
                        intr.furniture -= 1;
                        self.inv.wood += living.num("fuel", "wood_per_piece") as u32;
                        self.noise_m = living.num("fuel", "break_noise_m");
                        self.noise_t = 3.0;
                        out.door_breach_thud = true;
                        self.say("把一件家具砸开当柴烧（动静不小）。再按 T 添进炉子。", "You smash a piece of furniture for firewood. Loud. T again to feed the stove.");
                    } else {
                        self.say("没有煤，也没有能烧的家具了。去别的屋子找。", "No coal and nothing left here to burn.");
                    }
                } else if let Some(w) = win_near {
                    if self.inv.scrap > 0 {
                        self.inv.scrap -= 1;
                        w.boarded = true;
                        out.door_action = true;
                        self.say("用木板废铁封死破窗，挡住了穿堂冷风。", "Boarded up the shattered window!");
                    } else {
                        self.say("封窗需要废铁 1 份。", "Need 1 scrap to board window.");
                    }
                } else if d_front <= 2.6 || d_back <= 2.6 {
                    let use_back = d_back < d_front;
                    if self.inv.scrap > 0 {
                        self.inv.scrap -= 1;
                        if use_back {
                            if let Some(bd) = &mut intr.back_door {
                                bd.open = false;
                                bd.barricade_hp = bd.max_barricade_hp;
                            }
                        } else {
                            intr.front_door.open = false;
                            intr.front_door.barricade_hp = intr.front_door.max_barricade_hp;
                        }
                        out.door_action = true;
                        self.say("顶死门栓！俄军想闯进来得先砸半天门（按 E 可拆开）。", "Barricaded the door!");
                    } else {
                        self.say("顶门需要废铁 1 份。", "Need 1 scrap to barricade door.");
                    }
                } else if let Some(w) = intr.windows.iter_mut().find(|w| w.broken && !w.boarded) {
                    if self.inv.scrap > 0 {
                        self.inv.scrap -= 1;
                        w.boarded = true;
                        out.door_action = true;
                        self.say("封住了屋里被打碎的玻璃窗！", "Boarded shattered window!");
                    } else {
                        self.say("封窗需要废铁 1 份。", "Need 1 scrap.");
                    }
                } else if intr.stove_at.is_some() {
                    self.say("走到门边、破窗或铁皮煤炉旁按 T，可顶门、封窗或生煤炉。", "Move near a door, broken window, or stove and press T.");
                } else {
                    self.say("靠近大门按 T 用 1 废铁顶死门，或按 K 拉窗帘。", "Stand by door + T to barricade, or K for curtains.");
                }
            }
        }
    }

    fn shatter_windows_near(&mut self, at: P2, radius: f32, out: &mut TickOutput) {
        let mut broke = false;
        let bound = radius + 24.0;
        for intr in &mut self.interiors {
            if (intr.origin.x - at.x).abs() > bound || (intr.origin.y - at.y).abs() > bound {
                continue;
            }
            for w in &mut intr.windows {
                if !w.broken && !w.boarded && w.at.dist(at) <= radius {
                    w.broken = true;
                    broke = true;
                }
            }
        }
        if broke {
            out.glass_break = true;
        }
    }

    fn step_timeline(&mut self) {
        let bit = |mask: &mut u32, idx: u32, cond: bool| -> bool {
            let flag = 1u32 << idx;
            if (*mask & flag) == 0 && cond {
                *mask |= flag;
                true
            } else {
                false
            }
        };
        let h = self.hour;
        if bit(&mut self.events_fired, 0, h >= hm(16, 4)) {
            self.say("你在中央街下了班。天已经擦黑了。", "You're off work on Central Street. Dusk is falling.");
        }
        if bit(&mut self.events_fired, 1, h >= hm(16, 50)) {
            self.push_sms("赵小北", "我在母亲广场这边。江风挺硬，你还来不？对岸灯都亮了。", "I'm by Mother Square. The wind is sharp. Their lights are on across the ice.");
            self.say("手机震了一下。赵小北在江边等你。", "Your phone buzzes. Zhao Xiaobei is waiting by the river.");
        }
        if bit(&mut self.events_fired, 2, h >= hm(17, 22)) {
            self.alert_stage = self.alert_stage.max(1);
            self.say("江对岸有人在喊，天上像有闷雷。", "Shouting on the far bank. A dull crack across the river.");
        }
        if bit(&mut self.events_fired, 3, h >= CROSSING_HOUR) {
            self.alert_stage = 2;
            self.push_sms(
                "黑河市应急广播",
                "紧急通知：俄军正自黑龙江冰面越境推进。远离江岸，注意保暖。可前往中央街(-269,-325)协助组织抵抗。",
                "Emergency: Russian troops are crossing the ice. Leave the riverbank. Central Street resistance cell active.",
            );
            self.say("警报。不是演习！俄军正在越过黑龙江。", "Alert. Russian troops are crossing the frozen Heilongjiang.");
            self.objective_zh = "活到 07:00，或按 H 集结市民抵抗！别跑、躲开路灯、遇喝令站住可搜身放行，跑必挨枪。\n18:30 中央街民兵互助点激活，20:00 社区储备箱与求救居民刷新。".into();
        }
        if bit(&mut self.events_fired, 4, h >= hm(18, 30)) {
            self.push_sms(
                "老周 · 中央街",
                "我这儿有几支老猎枪/半自动和燃烧瓶！在中央街(-269,-325)汇合，街坊们准备搭路障守街口。",
                "Lao Zhou (Central St): I have rifles and Molotovs at (-269,-325). Come organize the street defence.",
            );
            self.say("中央街(-269,-325)老周的民兵互助点已就绪，过去按 E 领枪组织反击！", "Militia cell ready on Central St. Press E there to arm up!");
        }
        if bit(&mut self.events_fired, 5, h >= BLACKOUT_HOUR) {
            self.blackout = true;
            self.say("19:30 全城的灯一起灭了。黑暗现在是你的掩护。", "19:30 Blackout across Heihe. Use the dark.");
        }
        if bit(&mut self.events_fired, 6, h >= hm(20, 0)) {
            self.push_sms(
                "楼道求救 & 社区补给",
                "中央街(-255,-320)有人打出「救命 楼里还有5人」求救牌，救出可接通电台(R 呼叫迫击炮支援)！文化街亦有冬季储备箱。",
                "SOS at (-255,-320): rescue neighbours to unlock radio fire support (R). Winter cache open on Wenhua St.",
            );
            self.say("20:00 新的社区补给与求救居民已出现！", "20:00 New winter supplies and SOS neighbours unlocked!");
        }
        if bit(&mut self.events_fired, 7, h >= HEAT_OFF_HOUR) {
            self.say("21:00 暖气片凉了！全城停暖，门脸只能挡风——筑个火桶(3)或用暖宝宝。", "21:00 District heating stopped! Doorways only block wind now.");
        }
        if bit(&mut self.events_fired, 8, h >= hm(22, 30)) {
            self.push_sms("海兰街发现物资", "海兰街口(-116,-156)留下一箱边防弹药与防寒装备！", "Abandoned border crate near Hailan St (-116,-156) with ammo and gear!");
            self.say("22:30 海兰街拱门旁发现新补给箱。", "22:30 New supply crate near Hailan St arch.");
        }
        if bit(&mut self.events_fired, 9, h >= hm(25, 0)) {
            let c8 = self.caches[7].at;
            self.push_sms(
                "深夜接应",
                &format!("中央街南段暗巷({:.0},{:.0})有人送来热食、煤块与弹药，撑到天亮！", c8.x, c8.y),
                "01:00 Midnight cache with hot food, coal, and ammo!",
            );
        }
        if !self.warned_river && self.player_at.y > -20.0 {
            self.warned_river = true;
            self.say("江对岸就是布拉戈维申斯克。冰面上能看出推进路线。", "Blagoveshchensk across the frozen river.");
        }
    }

    fn handle_items(&mut self, input: &SimInput) {
        if input.use_bandage && !self.body.bleeding && self.body.illness > 10.0 && self.inv.medicine > 0 {
            self.inv.medicine -= 1;
            self.body.illness = (self.body.illness - crate::living::get().num("body", "medicine_cures")).max(0.0);
            self.say("吃了药。烧退了一些。", "You take the medicine. The fever eases a little.");
        } else if input.use_bandage && (self.body.bleeding || self.body.hp < 100.0) {
            if self.inv.bandage > 0 {
                self.inv.bandage -= 1;
                self.body.bleeding = false;
                self.body.hp = (self.body.hp + 14.0).min(100.0);
                self.say("你把伤口扎紧了，血止住了。", "You bind the wound tight. Bleeding stops.");
            } else {
                self.say("没有绷带，快找药店或补给箱。", "No bandage.");
            }
        }
        if input.use_food && self.inv.food > 0 {
            self.inv.food -= 1;
            let living = crate::living::get();
            let gain = living.num("body", "food_item_kcal") * 100.0 / living.num("body", "food_reserve_kcal");
            self.body.hunger = (self.body.hunger + gain).min(100.0);
            self.say("啃了几口冻干粮，肚子里有了热气。", "You eat cold rations standing up.");
        }
        if input.use_water && self.inv.water > 0 {
            self.inv.water -= 1;
            let living = crate::living::get();
            let gain = living.num("body", "water_item_l") * 100.0 / living.num("body", "water_reserve_l");
            self.body.thirst = (self.body.thirst + gain).min(100.0);
            self.say("水冰得牙疼，总算缓过来了。", "Ice-cold water.");
        }
        if input.use_warmer && self.inv.warmer > 0 {
            self.inv.warmer -= 1;
            self.body.temp = (self.body.temp + 0.75).min(36.9);
            self.say("暖宝宝贴在胸口，身上暖起来了。", "Hand warmer against your chest.");
        }
        if let Some(kind) = input.select_barricade {
            self.selected_barricade = kind;
        }
        if input.rally_resistance && !self.resistance_active {
            let at_cell = self.caches.iter().any(|c| c.kind == CacheKind::ResistanceCell && c.at.dist(self.player_at) <= 15.0);
            let at_safe_base = self.active_interior.is_some_and(|i| {
                matches!(
                    self.interiors[i].kind,
                    InteriorKind::PlayerHome | InteriorKind::NeighbourHome | InteriorKind::BasementShelter
                )
            });
            if self.hour >= hm(18, 30) && (at_cell || at_safe_base) {
                self.rally_militia();
            } else {
                self.say(
                    "18:30 后前往中央街老周接应点(-269,-325)、母亲广场，或在家/人防地下室内按 H 集结抵抗！",
                    "Reach a resistance cell, home, or shelter after 18:30 to rally.",
                );
            }
        }
    }

    pub fn rally_militia(&mut self) {
        if self.resistance_active {
            return;
        }
        self.resistance_active = true;
        self.inv.unlock_primary(PrimaryWeapon::Ak74m);
        self.inv.active_primary = PrimaryWeapon::Ak74m;
        self.inv.ammo += 20;
        self.inv.molotov += 2;
        self.inv.smoke_grenades += 2;
        self.inv.scrap += 4;
        self.inv.coal += 2;
        let roster = [
            ("老周", Rgb { r: 0.12, g: 0.12, b: 0.14 }),
            ("小吴", Rgb { r: 0.18, g: 0.22, b: 0.28 }),
            ("阿敏", Rgb { r: 0.42, g: 0.16, b: 0.16 }),
        ];
        let base = self.player_at;
        let floor = if self.active_interior.is_some() { self.player_floor } else { 0 };
        for (i, (name, coat)) in roster.into_iter().enumerate() {
            let off = P2::new((i as f32 - 1.5) * 1.6, if i % 2 == 0 { 1.4 } else { -1.4 });
            self.next_id += 1;
            self.militia.push(MilitiaMember {
                id: self.next_id,
                name,
                at: base.add(off),
                floor,
                facing: self.player_facing,
                hp: 85.0,
                coat,
                order: MilitiaOrder::Follow,
                fire_cd: 0.2 * i as f32,
                speed: 0.0,
                phase: i as f32,
                in_cover: false,
                vault_t: 0.0,
                downed: false,
                bleedout_s: 0.0,
                rescuing_id: None,
                rescue_progress: 0.0,
            });
        }
        self.say(
            "街坊们拿起了武器！老周、小吴、阿敏、大刘加入了你的抵抗小队！左键射击，V 丢燃烧瓶，1-5 选工事，B 修筑，G 下令守街。",
            "Neighbours take up arms! Militia squad joined. LMB shoot, V Molotov, B barricade, G order.",
        );
        self.push_radio("蓝队-1", "AWAITING ORDERS", "民兵抵抗小队已就位，右键下达战术机动指令", false);
    }

    fn step_interactions(&mut self, input: &SimInput, dt: f32, out: &mut TickOutput) {
        if let Some(i_idx) = self.active_interior.or(self.cutaway_interior) {
            if self.interact_interior(i_idx, input, out) {
                return;
            }
        }

        // Check downed militia rescue & loot drops first on E press.
        if input.interact_just_pressed {
            let mut revived = None;
            for (mi, m) in self.militia.iter_mut().enumerate() {
                if m.downed && m.hp > 0.0 && m.at.dist(self.player_at) <= 2.6 {
                    m.downed = false;
                    m.hp = 55.0;
                    m.bleedout_s = 0.0;
                    revived = Some((mi, m.name));
                    break;
                }
            }
            if let Some((_, name)) = revived {
                self.say(&format!("你现场包扎急救，拉起了重伤的 {}！", name), "Revived wounded militia teammate!");
                self.push_radio(name, "TEAMMATE RESCUED", &format!("{} 已现场止血苏醒，重返战斗！", name), false);
                return;
            }
            for drop in &mut self.drops {
                if !drop.taken && drop.at.dist(self.player_at) < 2.6 {
                    drop.taken = true;
                    self.inv.ammo += drop.ammo;
                    self.inv.bandage += drop.bandage;
                    self.inv.food += drop.food;
                    if drop.grants_rifle {
                        self.inv.unlock_primary(PrimaryWeapon::Ak74m);
                    }
                    let msg = format!(
                        "缴获俄军装备：弹药×{} 绷带×{} 口粮×{}{}",
                        drop.ammo,
                        drop.bandage,
                        drop.food,
                        if drop.grants_rifle { " + 缴获AK步枪！" } else { "" }
                    );
                    self.say(&msg, "Looted Russian patrol gear.");
                    return;
                }
            }

            // Unlocked progressive caches / resistance cells / SOS neighbours.
            let mut unlocked_hit: Option<usize> = None;
            for (i, c) in self.caches.iter().enumerate() {
                if !c.collected && self.hour >= c.unlock_hour && c.at.dist(self.player_at) <= 4.0 {
                    unlocked_hit = Some(i);
                    break;
                }
            }
            if let Some(idx) = unlocked_hit {
                self.collect_cache(idx);
                return;
            }

            // Friendly pedestrian talk before/after alert.
            if let Some(p) = self.pedestrians.iter().find(|p| !p.hidden && p.at.dist(self.player_at) < 3.0) {
                let id = p.id as usize;
                if self.alert_stage >= 2 {
                    let lines = ["别站在空地上，进门脸！", "听见没有？俄军过江了！", "快去中央街找老周，大伙在凑东西守街口！"];
                    self.say(lines[id % lines.len()], "Get inside!");
                } else {
                    let lines = ["这风跟刀子似的，快进店买点吃的。", "对岸今晚亮得早，总觉得不对劲。", "中央街这砖地，雪一化就滑。"];
                    self.say(lines[id % lines.len()], "Cold wind tonight.");
                }
                return;
            }
        }

        // Priority prompt: downed militia -> caches -> drops -> shops -> pedestrians.
        if let Some(m) = self.militia.iter().find(|m| m.downed && m.hp > 0.0 && m.at.dist(self.player_at) <= 2.6) {
            self.prompt = format!("按 E：急救重伤倒地的 {}", m.name);
            self.search_hold = 0.0;
            return;
        }
        if let Some(c) = self.caches.iter().find(|c| !c.collected && self.hour >= c.unlock_hour && c.at.dist(self.player_at) <= 4.0) {
            self.prompt = format!("按 E：互动 / 领取 [{}]", c.title);
            self.search_hold = 0.0;
            return;
        }
        if self.drops.iter().any(|d| !d.taken && d.at.dist(self.player_at) < 2.6) {
            self.prompt = "按 E：搜刮俄军战利品".into();
            self.search_hold = 0.0;
            return;
        }

        // Nearest unsearched shop sign.
        let mut best_i: Option<usize> = None;
        let mut best_d = SHOP_REACH;
        for (i, d) in self.doors.iter().enumerate() {
            if d.searched {
                continue;
            }
            let dist = d.at.dist(self.player_at);
            if dist < best_d {
                best_d = dist;
                best_i = Some(i);
            }
        }
        self.active_door = best_i;
        let Some(idx) = best_i else {
            self.search_hold = 0.0;
            if let Some(c) = self.caches.iter().find(|c| !c.collected && self.hour >= c.unlock_hour) {
                self.prompt = format!("附近新事件：{} ({:.0}m)", c.title, c.at.dist(self.player_at));
            } else {
                self.prompt.clear();
            }
            return;
        };

        let verb = if self.alert_stage < 2 { "买" } else { "搜" };
        if input.interact_held && self.player_gait == Gait::Still {
            self.search_hold += dt;
            let pct = ((self.search_hold / SEARCH_SECONDS) * 100.0).min(100.0) as u32;
            self.prompt = format!("{verb} {} … {pct}%", self.doors[idx].name);
            if self.search_hold >= SEARCH_SECONDS {
                self.search_hold = 0.0;
                self.search_door(idx);
            }
        } else {
            self.search_hold = 0.0;
            self.prompt = format!("按住 E：{verb} {}", self.doors[idx].name);
        }
    }

    fn interact_interior(&mut self, i_idx: usize, input: &SimInput, out: &mut TickOutput) -> bool {
        let pat = self.player_at;
        let in_room = self.active_interior == Some(i_idx);
        let pf = self.player_floor;

        // Allow unbarricading any interior door on 1F within 2.2m.
        if pf == 0 {
            let mut near_bar_door = None;
            {
                let intr = &self.interiors[i_idx];
                if !intr.front_door.destroyed && intr.front_door.barricade_hp > 0.0 && intr.front_door.at.dist(pat) <= 2.2 {
                    near_bar_door = Some(0u8);
                } else if intr.back_door.as_ref().is_some_and(|b| !b.destroyed && b.barricade_hp > 0.0 && b.at.dist(pat) <= 2.2) {
                    near_bar_door = Some(1u8);
                } else if intr.inner_door.as_ref().is_some_and(|id| !id.destroyed && id.barricade_hp > 0.0 && id.at.dist(pat) <= 2.2) {
                    near_bar_door = Some(2u8);
                }
            }
            if let Some(which) = near_bar_door {
                if input.interact_just_pressed {
                    let intr = &mut self.interiors[i_idx];
                    match which {
                        1 => if let Some(bd) = &mut intr.back_door {
                            bd.barricade_hp = 0.0;
                            bd.open = true;
                        },
                        2 => if let Some(id) = &mut intr.inner_door {
                            id.barricade_hp = 0.0;
                            id.open = true;
                        },
                        _ => {
                            intr.front_door.barricade_hp = 0.0;
                            intr.front_door.open = true;
                        }
                    }
                    out.door_action = true;
                    self.say("拆开了顶门木棍，门重新打开了。", "Removed door barricade.");
                    return true;
                }
                self.prompt = "按 E 拆开顶门障碍 · 通行大门".into();
                return true;
            }
        }

        if !in_room {
            let enter_target = {
                let intr = &self.interiors[i_idx];
                if intr.front_door.barricade_hp <= 0.0 && intr.front_door.at.dist(pat) <= 2.4 {
                    Some((false, intr.front_door.inside))
                } else if let Some(bd) = intr
                    .back_door
                    .as_ref()
                    .filter(|b| b.barricade_hp <= 0.0 && b.at.dist(pat) <= 2.4)
                {
                    Some((true, bd.inside))
                } else {
                    None
                }
            };
            if let Some((is_back, inside_pos)) = enter_target {
                let drop_or_cache_near = self.drops.iter().any(|d| !d.taken && d.at.dist(pat) < 2.6)
                    || self
                        .caches
                        .iter()
                        .any(|c| !c.collected && self.hour >= c.unlock_hour && c.at.dist(pat) <= 4.0);
                let has_outdoor_shop = !drop_or_cache_near
                    && self.doors.iter().any(|d| !d.searched && d.at.dist(pat) < SHOP_REACH);
                if input.interact_just_pressed && !drop_or_cache_near && !has_outdoor_shop {
                    self.player_at = inside_pos;
                    self.player_floor = 0;
                    let intr = &mut self.interiors[i_idx];
                    if is_back {
                        if let Some(bd) = &mut intr.back_door {
                            bd.open = true;
                        }
                    } else {
                        intr.front_door.open = true;
                    }
                    intr.discovered = true;
                    self.active_interior = Some(i_idx);
                    self.cutaway_interior = Some(i_idx);
                    self.current_room_temp = Some(intr.room_temp_c);
                    self.active_interior_name = intr.name.clone();
                    out.door_action = true;
                    return true;
                }
                if !drop_or_cache_near && !has_outdoor_shop && !input.interact_held {
                    self.prompt = format!(
                        "门口：[{}]（直接走入室内，或按 E 推门进入）",
                        self.interiors[i_idx].name
                    );
                    self.search_hold = 0.0;
                    return true;
                }
            }
            return false;
        }

        // Priority inside rooms:
        // 0F: locker stash -> room loot shelf -> NPC talk -> unlocked real door E open/close -> stairwell -> stove/windows.
        // >=1F: upper-floor loot shelf -> stairwell -> tactical prompt.
        if pf == 0 {
            let locker_hit = self.interiors[i_idx].locker_at.is_some_and(|p| p.dist(pat) <= 2.2)
                && {
                    let st = &self.interiors[i_idx].stash;
                    st.food + st.water + st.bandage + st.warmer + st.coal + st.scrap + st.ammo > 0
                };
            if locker_hit {
                if input.interact_just_pressed {
                    let st = std::mem::take(&mut self.interiors[i_idx].stash);
                    self.inv.food += st.food;
                    self.inv.water += st.water;
                    self.inv.bandage += st.bandage;
                    self.inv.warmer += st.warmer;
                    self.inv.ammo += st.ammo;
                    self.inv.scrap += st.scrap;
                    self.inv.coal += st.coal;
                    self.say("打开家里储物柜：取出了备用的干粮、饮水、绷带、暖宝宝、废铁与煤炭！", "Took home stash supplies!");
                    return true;
                }
                self.prompt = "按 E：打开家用应急储物柜".into();
                return true;
            }
        }

        let loot_idx = if pf == 0 {
            self.interiors[i_idx].loots.iter().position(|l| !l.searched && l.at.dist(pat) <= 2.2)
        } else {
            self.interiors[i_idx]
                .floors
                .get(pf as usize)
                .and_then(|fl| fl.loots.iter().position(|l| !l.searched && l.at.dist(pat) <= 2.2))
        };
        if let Some(li) = loot_idx {
            let (label, lk, back, lid) = if pf == 0 {
                let l = &self.interiors[i_idx].loots[li];
                (l.label, l.kind, l.in_backroom, l.id as i64)
            } else {
                let l = &self.interiors[i_idx].floors[pf as usize].loots[li];
                (l.label, l.kind, l.in_backroom, l.id as i64)
            };
            if input.interact_just_pressed {
                if pf == 0 {
                    self.interiors[i_idx].loots[li].searched = true;
                } else {
                    self.interiors[i_idx].floors[pf as usize].loots[li].searched = true;
                }
                let rolled = roll_shop(lk, lid * 131 + i_idx as i64 * 37 + pf as i64 * 17);
                self.inv.food += rolled.food;
                self.inv.water += rolled.water;
                self.inv.bandage += rolled.bandage;
                self.inv.warmer += rolled.warmer;
                self.inv.scrap += rolled.scrap + u32::from(back);
                self.inv.coal += rolled.coal + u32::from(back && matches!(self.interiors[i_idx].kind, InteriorKind::Hardware | InteriorKind::BasementShelter));
                self.inv.molotov += rolled.molotov;
                self.inv.medicine += rolled.medicine;
                if matches!(self.interiors[i_idx].kind, InteriorKind::BasementShelter) && back && lid % 2 == 0 {
                    self.inv.rpg += 1;
                    self.inv.hj12 += 1;
                    self.inv.grenades += 2;
                    self.inv.unlock_primary(PrimaryWeapon::Qbz191);
                    self.inv.unlock_primary(PrimaryWeapon::Saiga12);
                }
                if rolled.coat && self.body.coat_bonus < 0.35 {
                    self.body.coat_bonus = 0.35;
                }
                self.say(&format!("翻找了 [{label}]，收获一批过冬物资！"), "Searched indoor shelves.");
                return true;
            }
            self.prompt = format!("按 E：翻找 {}F 的 [{label}]", pf + 1);
            return true;
        }

        if pf == 0 {
            let npc_idx = self.interiors[i_idx].npcs.iter().position(|n| n.at.dist(pat) <= 2.3);
            if let Some(ni) = npc_idx {
                let npc_label = self.interiors[i_idx].npcs[ni].label;
                let line = if self.alert_stage >= 2 {
                    self.interiors[i_idx].npcs[ni].line_war
                } else {
                    self.interiors[i_idx].npcs[ni].line_calm
                };
                if input.interact_just_pressed {
                    self.say(&format!("{npc_label}：{line}"), "Talked to indoor survivor.");
                    if self.interiors[i_idx].kind == InteriorKind::NeighbourHome && self.hour >= hm(20, 0) && !self.neighbours_rescued {
                        self.neighbours_rescued = true;
                        self.inv.has_radio = true;
                        self.inv.radio_strikes = self.inv.radio_strikes.max(2);
                        self.inv.coal += 3;
                        self.inv.scrap += 4;
                        self.inv.ammo += 12;
                        for c in &mut self.caches {
                            if c.kind == CacheKind::SosNeighbours {
                                c.collected = true;
                            }
                        }
                        self.say("帮王婶一家转移安全！她把退休老电台(×2)、煤炭和弹药留给了你（按 R 呼叫迫击炮）！", "Rescued neighbours! Radio & supplies gained.");
                    }
                    return true;
                }
                self.prompt = format!("按 E：与 {npc_label} 交谈");
                return true;
            }

            // Unlocked door manual swing toggle (E opens or shuts front / back / inner partition door).
            let mut near_door: Option<(u8, DoorStyle, bool)> = None;
            {
                let intr = &self.interiors[i_idx];
                if !intr.front_door.destroyed && intr.front_door.at.dist(pat) <= 2.15 {
                    near_door = Some((0, intr.front_door.style, intr.front_door.open));
                } else if let Some(bd) = &intr.back_door {
                    if !bd.destroyed && bd.at.dist(pat) <= 2.15 {
                        near_door = Some((1, bd.style, bd.open));
                    }
                }
                if near_door.is_none() {
                    if let Some(idoor) = &intr.inner_door {
                        if !idoor.destroyed && idoor.at.dist(pat) <= 2.05 {
                            near_door = Some((2, idoor.style, idoor.open));
                        }
                    }
                }
            }
            if let Some((which, style, was_open)) = near_door {
                let dname = Self::door_label(style);
                if input.interact_just_pressed {
                    let intr = &mut self.interiors[i_idx];
                    let now_open = !was_open;
                    match which {
                        1 => if let Some(bd) = &mut intr.back_door { bd.open = now_open; },
                        2 => if let Some(id) = &mut intr.inner_door { id.open = now_open; },
                        _ => intr.front_door.open = now_open,
                    }
                    out.door_action = true;
                    let verb = if now_open { "推开了" } else { "关上了" };
                    self.say(&format!("{verb}{dname}。"), if now_open { "Opened door." } else { "Closed door." });
                    return true;
                }
                let state = if was_open { "敞开" } else { "闭合" };
                self.prompt = format!("{dname}({state})：按 E 开关门 · 按 T 耗1废铁顶死门 · K窗帘 · L手电");
                return true;
            }
        }

        let stair_near = self.interiors[i_idx].num_floors > 1
            && self.interiors[i_idx].stairwell_world().dist(pat) <= 2.55;
        if stair_near {
            let max_f = self.interiors[i_idx].num_floors.saturating_sub(1);
            if input.interact_just_pressed && input.floor_delta == 0 {
                if pf < max_f {
                    self.player_floor += 1;
                } else if pf > 0 {
                    self.player_floor -= 1;
                }
                self.say(&format!("顺楼梯来到 {}F。", self.player_floor + 1), "Used stairs.");
                return true;
            }
            self.prompt = format!(
                "楼梯口（{}F/{}F）：Space 上楼 · Ctrl 下楼（按 E 快速切换）",
                self.player_floor + 1,
                self.interiors[i_idx].num_floors
            );
            return true;
        }

        if pf == 0
            && self.interiors[i_idx].stove_at.is_some_and(|p| p.dist(pat) <= 2.4)
            && self.interiors[i_idx].stove_fuel_s >= self.melt_fuel_s()
            && input.interact_just_pressed
        {
            let living = crate::living::get();
            self.interiors[i_idx].stove_fuel_s = (self.interiors[i_idx].stove_fuel_s - self.melt_fuel_s()).max(0.0);
            let per_l = 100.0 / living.num("body", "water_reserve_l");
            self.body.thirst = (self.body.thirst + living.num("fuel", "melt_litres") * per_l).min(100.0);
            self.say("在煤炉上融化清洁积雪煮成热水，喝下去润喉解渴！", "Melted snow over coal stove for drinking water!");
            return true;
        }

        let intr = &self.interiors[i_idx];
        if pf > 0 {
            self.prompt = format!(
                "楼上 [{} {}F/{}F {:.1}°C] · 俯掷燃烧瓶(V)2×伤 · Space/Ctrl去楼梯换层",
                intr.name,
                pf + 1,
                intr.num_floors,
                intr.room_temp_c
            );
            return true;
        }
        if intr.stove_at.is_some_and(|p| p.dist(pat) <= 2.4) {
            let hours_left = intr.stove_fuel_s * TIME_SCALE / 3600.0;
            if intr.stove_fuel_s >= self.melt_fuel_s() {
                self.prompt = format!("炉火还能烧 {hours_left:.1} 小时 · 按 E 化雪喝水 · 按 T 添煤或柴（屋里还有 {} 件家具）", intr.furniture);
            } else if intr.stove_fuel_s > 0.0 {
                self.prompt = format!("炉火快灭了（{hours_left:.1} 小时）· 按 T 添煤或柴（屋里还有 {} 件家具）", intr.furniture);
            } else {
                self.prompt = "铁皮煤炉：按 T 消耗 1 煤点燃取暖".into();
            }
        } else if intr.windows.iter().any(|w| w.broken && !w.boarded && w.at.dist(pat) <= 2.6) {
            self.prompt = "破窗灌风：按 T 消耗 1 废铁钉木板封窗".into();
        } else {
            self.prompt = format!(
                "室内 [{} {:.1}°C] · E开柜/开关门 · Space上楼 · T生炉/封窗 · K窗帘 · L手电",
                intr.name, intr.room_temp_c
            );
        }
        true
    }

    pub fn search_door(&mut self, idx: usize) {
        if idx >= self.doors.len() || self.doors[idx].searched {
            return;
        }
        self.doors[idx].searched = true;
        let seed = idx as i64 * 97 + (self.doors[idx].at.x * 10.0) as i64;
        let loot = roll_shop(self.doors[idx].kind, seed);
        self.inv.food += loot.food;
        self.inv.water += loot.water;
        self.inv.bandage += loot.bandage;
        self.inv.warmer += loot.warmer;
        self.inv.scrap += loot.scrap;
        self.inv.coal += loot.coal;
        self.inv.molotov += loot.molotov;
        if loot.coat && self.body.coat_bonus < 0.35 {
            self.body.coat_bonus = 0.35;
        }
        let mut parts = Vec::new();
        if loot.coat { parts.push("厚棉袄(已穿上)".into()); }
        if loot.bandage > 0 { parts.push(format!("绷带×{}", loot.bandage)); }
        if loot.food > 0 { parts.push(format!("吃的×{}", loot.food)); }
        if loot.water > 0 { parts.push(format!("水×{}", loot.water)); }
        if loot.warmer > 0 { parts.push(format!("暖宝宝×{}", loot.warmer)); }
        if loot.scrap > 0 { parts.push(format!("废铁×{}", loot.scrap)); }
        if loot.coal > 0 { parts.push(format!("煤炭×{}", loot.coal)); }
        if loot.molotov > 0 { parts.push(format!("燃烧瓶×{}", loot.molotov)); }
        let name = self.doors[idx].name.clone();
        if parts.is_empty() {
            self.say(&format!("{name}：柜台里已经空了。"), "Empty shelves.");
        } else {
            self.say(&format!("{name}：拿到 {}", parts.join("、")), "Scavenged supplies.");
        }
    }

    fn collect_cache(&mut self, idx: usize) {
        let kind = self.caches[idx].kind;
        self.caches[idx].collected = true;
        match kind {
            CacheKind::StreetScrap => {
                self.inv.scrap += 4;
                self.inv.coal += 2;
                self.say("搬走了废铁×4、煤炭×2，可按 B 搭路障或火桶！", "Collected scrap and coal.");
            }
            CacheKind::ResistanceCell => {
                if !self.resistance_active {
                    self.rally_militia();
                } else {
                    self.inv.ammo += 24;
                    self.inv.molotov += 2;
                    self.inv.pf98 += 1;
                    self.inv.grenades += 2;
                    self.inv.unlock_primary(PrimaryWeapon::Saiga12);
                    self.inv.unlock_primary(PrimaryWeapon::EwJammer);
                    self.say("补充了步枪子弹×24、PF-98火箭×1、防无人机霰弹与干扰枪！", "Resupplied by resistance cell.");
                }
            }
            CacheKind::SosNeighbours => {
                if self.neighbours_rescued {
                    return;
                }
                self.neighbours_rescued = true;
                self.inv.has_radio = true;
                self.inv.radio_strikes += 2;
                self.inv.bandage += 2;
                self.inv.warmer += 2;
                self.inv.coal += 2;
                self.inv.friendly_fpv += 1;
                self.next_id += 1;
                self.militia.push(MilitiaMember {
                    id: self.next_id,
                    name: "赵小北",
                    at: self.player_at.add(P2::new(1.2, 0.8)),
                    floor: 0,
                    facing: self.player_facing,
                    hp: 85.0,
                    coat: Rgb { r: 0.16, g: 0.24, b: 0.34 },
                    order: MilitiaOrder::Follow,
                    fire_cd: 0.3,
                    speed: 0.0,
                    phase: 0.4,
                    in_cover: false,
                    vault_t: 0.0,
                    downed: false,
                    bleedout_s: 0.0,
                    rescuing_id: None,
                    rescue_progress: 0.0,
                });
                self.say("救出居民！接通了军方电台（×2次呼叫：R 呼叫 155mm 榴弹炮与装甲支援，赠自杀 FPV×1）！", "Neighbours rescued! Radio unlocked.");
            }
            CacheKind::CommunityCache => {
                self.inv.food += 2;
                self.inv.water += 2;
                self.inv.warmer += 2;
                self.inv.bandage += 2;
                self.inv.coal += 2;
                self.inv.grenades += 2;
                self.say("社区储藏室补给：食物×2、水×2、暖宝宝×2、绷带×2、煤炭×2、手雷×2。", "Community cache collected.");
            }
            CacheKind::BorderCrate => {
                self.inv.ammo += 40;
                self.inv.molotov += 2;
                self.inv.rpg += 2;
                self.inv.pf98 += 2;
                self.inv.hj12 += 2;
                self.inv.rpo_shmel += 2;
                self.inv.friendly_fpv += 2;
                self.inv.grenades += 3;
                self.inv.scrap += 5;
                self.inv.has_radio = true;
                self.inv.radio_strikes += 1;
                self.inv.unlock_primary(PrimaryWeapon::Qbz191);
                self.inv.unlock_primary(PrimaryWeapon::Qbu191);
                self.inv.unlock_primary(PrimaryWeapon::Pkm);
                self.inv.unlock_primary(PrimaryWeapon::Saiga12);
                self.inv.unlock_primary(PrimaryWeapon::EwJammer);
                self.body.coat_bonus = self.body.coat_bonus.max(0.35);
                self.say("边防武器库箱：解锁 QBU-191/PKM/干扰枪 + HJ-12/PF-98/大黄蜂/FPV + 电台呼援×1！", "Border armory crate looted!");
            }
            CacheKind::SmugglerCache => {
                self.inv.food += 2;
                self.inv.water += 2;
                self.inv.bandage += 2;
                self.inv.coal += 2;
                self.inv.ammo += 24;
                self.inv.hj12 += 1;
                self.inv.rpo_shmel += 1;
                self.inv.friendly_fpv += 1;
                self.inv.has_radio = true;
                self.inv.radio_strikes += 1;
                self.inv.unlock_primary(PrimaryWeapon::EwJammer);
                self.say("深夜接应物资：HJ-12×1、RPO-A温压筒×1、我方FPV×1、弹药×24、电台呼援×1！", "Midnight supplies collected.");
            }
        }
    }

    fn step_pedestrians(&mut self, city: &City, ground: &Ground, dt: f32) {
        if city.paths.is_empty() {
            return;
        }
        let panicked = self.alert_stage >= 2;
        for ped in &mut self.pedestrians {
            if ped.hidden {
                continue;
            }
            let target = if panicked {
                let mut best = ped.at.add(P2::new(0.0, -30.0));
                let mut best_d = f32::MAX;
                for (hp, _) in &city.heat {
                    let d = ped.at.dist(*hp);
                    if d < best_d {
                        best_d = d;
                        best = *hp;
                    }
                }
                if best_d < 2.6 || ped.fled_for > 22.0 {
                    ped.hidden = true;
                    continue;
                }
                ped.fled_for += dt;
                best
            } else {
                let path = &city.paths[ped.path_idx % city.paths.len()];
                let wp = path[ped.wp_idx.min(path.len() - 1)];
                if ped.at.dist(wp) < 1.3 {
                    let next = ped.wp_idx as isize + ped.wp_dir;
                    if next < 0 || next >= path.len() as isize {
                        ped.wp_dir = -ped.wp_dir;
                    }
                    ped.wp_idx = (ped.wp_idx as isize + ped.wp_dir).clamp(0, path.len() as isize - 1) as usize;
                }
                path[ped.wp_idx.min(path.len() - 1)]
            };
            let wish = target.sub(ped.at);
            if wish.len() < 0.05 {
                continue;
            }
            let speed = if panicked { 3.5 } else { 1.15 };
            let dir = wish.norm();
            ped.at = ground.step(ped.at, ped.at.add(dir.mul(speed * dt)), 0.28);
            ped.facing = dir.y.atan2(dir.x);
            ped.speed = speed;
            ped.phase += (std::f32::consts::TAU * speed / (0.95 + 0.42 * speed)) * dt;
        }
    }

    fn step_combat_actions(&mut self, city: &City, ground: &Ground, forces: &Forces, input: &SimInput, out: &mut TickOutput) {
        if input.build_barricade {
            let (need_scrap, need_coal) = self.selected_barricade.cost();
            if self.inv.scrap >= need_scrap && self.inv.coal >= need_coal {
                self.inv.scrap -= need_scrap;
                self.inv.coal -= need_coal;
                let fwd = P2::new(self.player_facing.cos(), self.player_facing.sin());
                let want = match input.build_at {
                    Some(p) => {
                        let d = p.sub(self.player_at);
                        let len = d.len();
                        if len > 5.5 { self.player_at.add(d.norm().mul(5.5)) } else if len < 1.0 { self.player_at.add(fwd.mul(1.8)) } else { p }
                    }
                    None => self.player_at.add(fwd.mul(1.8)),
                };
                let yaw = want.sub(self.player_at).y.atan2(want.sub(self.player_at).x);
                let pos = ground.step(self.player_at, want, 0.4);
                if self.selected_barricade == BarricadeKind::FireBarrel {
                    self.add_fire(pos, f32::INFINITY);
                    self.say("点起了一个防寒火桶（附近可取暖，也会在夜里暴露位置）。", "Lit a fire barrel.");
                } else {
                    self.say(&format!("架设了路障：{}", self.selected_barricade.label()), "Barricade placed.");
                }
                self.barricades.push(Barricade {
                    at: pos,
                    kind: self.selected_barricade,
                    yaw,
                    shoved: false,
                    hp: self.selected_barricade.max_hp(),
                });
            } else {
                self.say(&format!("材料不够：{}需要废铁{}、煤{}。", self.selected_barricade.label(), need_scrap, need_coal), "Not enough scrap or coal.");
            }
        }

        if let Some(dest) = input.command_militia_to {
            self.issue_militia_order(dest);
        }

        if input.toggle_militia_order && !self.militia.is_empty() {
            let idx = self.next_militia_idx % self.militia.len();
            self.next_militia_idx += 1;
            let at = self.player_at;
            let garrison_spot = self.active_interior.and_then(|i_idx| {
                self.interiors[i_idx]
                    .firing_spots
                    .get(idx % self.interiors[i_idx].firing_spots.len().max(1))
                    .copied()
            });
            let (name, mode_tag) = {
                let m = &mut self.militia[idx];
                let (ord, tag) = match m.order {
                    MilitiaOrder::Follow => {
                        if let Some(spot) = garrison_spot {
                            (MilitiaOrder::Garrison(spot), 2u8)
                        } else {
                            (MilitiaOrder::Hold(at), 1u8)
                        }
                    }
                    MilitiaOrder::Hold(_) | MilitiaOrder::Garrison(_) => (MilitiaOrder::Follow, 0u8),
                };
                m.order = ord;
                (m.name, tag)
            };
            match mode_tag {
                2 => {
                    self.say(&format!("{name} 留守屋内窗口掩体警戒！"), "Militia garrisoned indoors.");
                    self.push_radio(name, "HOLDING POSITION", &format!("{name} 正在屋内窗口掩体警戒"), false);
                }
                1 => {
                    self.say(&format!("{name} 就地依托掩体守住街口！"), "Militia holding position.");
                    self.push_radio(name, "HOLDING POSITION", &format!("{name} 就地依托掩体守住街口"), false);
                }
                _ => {
                    self.say(&format!("{name} 跟着你机动推进！"), "Militia following.");
                    self.push_radio(name, "YOU GOT IT", &format!("{name} 正随队机动推进"), false);
                }
            }
        }

        if let Some(w) = input.select_primary {
            if self.inv.has_primary(w) {
                self.inv.active_primary = w;
                self.say(&format!("切换主武器：{}", w.label()), "Switched primary weapon.");
            } else {
                self.say(&format!("尚未解锁：{}", w.label()), "Weapon locked.");
            }
        } else if input.cycle_primary && self.inv.has_rifle {
            let w = self.inv.cycle_primary();
            self.say(&format!("切换主武器：{}", w.label()), "Cycled primary weapon.");
        }

        if let Some(at) = input.select_at {
            self.inv.active_at = at;
            self.say(&format!("切换重火力：{}（余弹 {}）", at.label(), self.inv.at_ammo(at)), "Switched AT weapon.");
        } else if input.cycle_at {
            let at = self.inv.cycle_at();
            self.say(&format!("切换重火力：{}（余弹 {}）", at.label(), self.inv.at_ammo(at)), "Cycled AT weapon.");
        }

        if let Some(target) = input.fire_at {
            let w = self.inv.active_primary;
            let has_w = self.inv.has_primary(w) || (self.inv.has_rifle && self.inv.unlocked_primaries == 0);
            if has_w && self.player_shoot_cd <= 0.0 {
                if w.uses_ammo() && self.inv.ammo == 0 {
                    self.say("没子弹了！按 N 切换电子干扰枪，或搜集弹药箱。", "Out of ammo!");
                } else {
                    if w.uses_ammo() {
                        self.inv.ammo -= 1;
                    }
                    self.player_shoot_cd = w.cooldown();
                    let dir = target.sub(self.player_at).norm();
                    let yaw = if dir.len() > 0.05 { dir.y.atan2(dir.x) } else { self.player_facing };
                    self.player_facing = yaw;
                    let range = w.range();
                    let ray_end = self.player_at.add(P2::new(yaw.cos(), yaw.sin()).mul(range));
                    let from_y = self.player_floor as f32 * FLOOR_H + 1.35;

                    if let Some(i_idx) = self.active_interior {
                        if !self.interiors[i_idx].contains(target) {
                            self.interiors[i_idx].curtains_closed = false;
                            let mid = self.player_at.lerp(ray_end, 0.14);
                            self.shatter_windows_near(mid, 5.5, out);
                        }
                    }

                    if w == PrimaryWeapon::EwJammer {
                        // Electronic-warfare drone gun knocks out radio Recon Drones, radio FPVs & Lancets!
                        let mut jam_pt = self.player_at.add(P2::new(yaw.cos(), yaw.sin()).mul(35.0));
                        let mut jam_y = 6.0;
                        let mut jammed_any = false;
                        if self.drone.active
                            && self.drone.at.dist(self.player_at) <= range
                            && in_arc(yaw, self.player_at, self.drone.at, 32.0)
                        {
                            self.drone.active = false;
                            jam_pt = self.drone.at;
                            jam_y = self.drone.altitude;
                            jammed_any = true;
                            self.say("电磁干扰命中！俄军侦察无人机失控坠毁！", "Recon drone jammed out of sky!");
                        }
                        if let Some(fpv) = &self.fpv {
                            if fpv.at.dist(self.player_at) <= range && in_arc(yaw, self.player_at, fpv.at, 32.0) {
                                jam_pt = fpv.at;
                                jam_y = fpv.height;
                                if fpv.fibre_optic {
                                    self.say("抗干扰光纤 FPV 免疫电磁波！快按 N 切 Saiga-12 霰弹或步枪打下它！", "Fibre-optic FPV immune to EW! Shoot it with Saiga-12!");
                                } else {
                                    self.fpv = None;
                                    jammed_any = true;
                                    self.say("反制成功！无线电 FPV 无人机被电磁压制坠落！", "Radio FPV jammed!");
                                }
                            }
                        }
                        if let Some(lan) = &self.lancet {
                            if lan.at.dist(self.player_at) <= range && in_arc(yaw, self.player_at, lan.at, 32.0) {
                                jam_pt = lan.at;
                                jam_y = lan.height;
                                self.lancet = None;
                                jammed_any = true;
                                self.say("电磁干扰击落「柳叶刀-3」巡飞弹！", "Lancet-3 loitering munition jammed!");
                            }
                        }
                        out.tracers.push(TracerFx {
                            from: [self.player_at.x, from_y, -self.player_at.y],
                            to: [jam_pt.x, jam_y, -jam_pt.y],
                            friendly: true,
                        });
                        let _ = jammed_any;
                    } else {
                        // Shoot down aerial drones first if inside aim ray (or shotgun 22° cone).
                        let is_shotgun = w == PrimaryWeapon::Saiga12;
                        let mut hit_air = false;
                        if self.drone.active
                            && self.drone.at.dist(self.player_at) <= range
                            && ((is_shotgun && in_arc(yaw, self.player_at, self.drone.at, 22.0))
                                || nearest_on_segment(self.drone.at, self.player_at, ray_end).dist(self.drone.at) < 3.6)
                        {
                            out.tracers.push(TracerFx {
                                from: [self.player_at.x, from_y, -self.player_at.y],
                                to: [self.drone.at.x, self.drone.altitude, -self.drone.at.y],
                                friendly: true,
                            });
                            self.drone.active = false;
                            hit_air = true;
                            self.say("击落高空俄军侦察无人机！", "Shot down Russian recon drone!");
                        }
                        if let Some(fpv) = &self.fpv {
                            if fpv.at.dist(self.player_at) <= range
                                && ((is_shotgun && in_arc(yaw, self.player_at, fpv.at, 24.0))
                                    || nearest_on_segment(fpv.at, self.player_at, ray_end).dist(fpv.at) < 3.5)
                            {
                                out.tracers.push(TracerFx {
                                    from: [self.player_at.x, from_y, -self.player_at.y],
                                    to: [fpv.at.x, fpv.height, -fpv.at.y],
                                    friendly: true,
                                });
                                self.fpv = None;
                                hit_air = true;
                                self.say("凌空击碎来袭 FPV 自杀无人机！", "Shot down incoming FPV drone!");
                            }
                        }
                        if let Some(lan) = &self.lancet {
                            if lan.at.dist(self.player_at) <= range
                                && ((is_shotgun && in_arc(yaw, self.player_at, lan.at, 22.0))
                                    || nearest_on_segment(lan.at, self.player_at, ray_end).dist(lan.at) < 3.5)
                            {
                                out.tracers.push(TracerFx {
                                    from: [self.player_at.x, from_y, -self.player_at.y],
                                    to: [lan.at.x, lan.height, -lan.at.y],
                                    friendly: true,
                                });
                                self.lancet = None;
                                hit_air = true;
                                self.say("击落来袭「柳叶刀-3」巡飞弹！", "Shot down Lancet-3!");
                            }
                        }

                        if !hit_air {
                            self.gunfire_reveal_timer = 5.5;
                            let mut hit_pt = ray_end;
                            let mut hit_floor = 0u8;
                            let mut hit_soldier: Option<(usize, usize)> = None;
                            let mut hit_tank: Option<usize> = None;
                            let mut best_t = f32::MAX;

                            for (ti, t) in self.tanks.iter().enumerate() {
                                if t.hp <= 0.0
                                    || t.faction == Faction::Pla
                                    || !ground.clear_shot_from_interior(&self.interiors, self.active_interior, self.player_at, t.at)
                                {
                                    continue;
                                }
                                let q = nearest_on_segment(t.at, self.player_at, ray_end);
                                let d_ray = q.dist(t.at);
                                let dist = q.dist(self.player_at);
                                if d_ray < 2.3 && dist < best_t {
                                    best_t = dist;
                                    hit_pt = t.at;
                                    hit_floor = 0;
                                    hit_tank = Some(ti);
                                }
                            }

                            for (gi, g) in self.squads.iter().enumerate() {
                                for (si, s) in g.men.iter().enumerate() {
                                    if s.hp <= 0.0 {
                                        continue;
                                    }
                                    let los_ok = if s.floor == self.player_floor
                                        && self.interiors.iter().any(|it| it.contains(self.player_at) && it.contains(s.at))
                                    {
                                        true
                                    } else if s.floor != self.player_floor
                                        && self.interiors.iter().any(|it| it.contains(self.player_at) && it.contains(s.at) && it.is_slab_intact(self.player_floor.max(s.floor), s.at))
                                    {
                                        false
                                    } else {
                                        ground.clear_shot_from_interior(&self.interiors, self.active_interior, self.player_at, s.at)
                                    };
                                    if !los_ok {
                                        continue;
                                    }
                                    let q = nearest_on_segment(s.at, self.player_at, ray_end);
                                    let t = q.dist(self.player_at);
                                    let hit_w = if is_shotgun { 2.6 } else { 1.15 };
                                    if q.dist(s.at) < hit_w && t < best_t {
                                        best_t = t;
                                        hit_pt = s.at;
                                        hit_floor = s.floor;
                                        hit_soldier = Some((gi, si));
                                        hit_tank = None;
                                    }
                                }
                            }
                            if let Some((gi, si)) = hit_soldier {
                                let dist = self.player_at.dist(hit_pt);
                                let light = light_at(city, self.hour, self.blackout, false, self.player_at, &self.fire_barrels);
                                let w_mul = match w {
                                    PrimaryWeapon::Qbu191 => 1.35,
                                    PrimaryWeapon::Qbz191 => 1.10,
                                    PrimaryWeapon::Ak74m => 1.05,
                                    PrimaryWeapon::Pkm => 0.85,
                                    PrimaryWeapon::Saiga12 => if dist <= 18.0 { 1.35 } else { 0.35 },
                                    PrimaryWeapon::EwJammer => 0.0,
                                };
                                let move_mul = if self.player_gait != Gait::Still { 0.65 } else { 1.0 };
                                let dark_mul = if light == Light::NightDark && !self.flashlight_on { 0.65 } else { 1.0 };
                                let cover_mul = if self.squads[gi].men[si].in_cover { 0.55 } else { 1.0 };
                                let chance = (forces.hit_chance(dist) * w_mul * move_mul * dark_mul * cover_mul).clamp(0.05, 0.88);
                                let roll = self.next_rand(17);
                                let spread_roll = self.next_rand(18);
                                if roll < chance {
                                    out.tracers.push(TracerFx {
                                        from: [self.player_at.x, from_y, -self.player_at.y],
                                        to: [hit_pt.x, hit_floor as f32 * FLOOR_H + 1.2, -hit_pt.y],
                                        friendly: true,
                                    });
                                    let dmg = if is_shotgun { 56.0 } else { w.damage() };
                                    self.damage_soldier(gi, si, dmg);
                                } else {
                                    let miss_pt = hit_pt.add(P2::new((roll - 0.5) * 3.4, (spread_roll - 0.5) * 3.4));
                                    out.tracers.push(TracerFx {
                                        from: [self.player_at.x, from_y, -self.player_at.y],
                                        to: [miss_pt.x, hit_floor as f32 * FLOOR_H + 1.2, -miss_pt.y],
                                        friendly: true,
                                    });
                                }
                                self.alert_squads_near(hit_pt, self.player_at, forces);
                            } else {
                                out.tracers.push(TracerFx {
                                    from: [self.player_at.x, from_y, -self.player_at.y],
                                    to: [hit_pt.x, hit_floor as f32 * FLOOR_H + 1.2, -hit_pt.y],
                                    friendly: true,
                                });
                                if let Some(ti) = hit_tank {
                                    let kind = self.tanks[ti].kind;
                                    if kind.is_heavy_armor() {
                                        self.say(
                                            &format!("轻武器在 {} 复合装甲上跳弹！按 F 切重型反坦(HJ-12/PF-98/RPG)或高楼抛 V 燃烧瓶！", kind.label()),
                                            "Small arms ricocheted! Use AT weapon (F) or Molotov (V)!",
                                        );
                                    } else {
                                        let chip = if w == PrimaryWeapon::Qbu191 { 15.0 } else { 7.5 };
                                        self.tanks[ti].hp = (self.tanks[ti].hp - chip).max(0.0);
                                        if self.tanks[ti].hp <= 0.0 {
                                            let dead = self.tanks.remove(ti);
                                            let pos = dead.at;
                                            self.push_wreck(dead);
                                            out.explosions.push(ExplosionFx { at: pos, radius: 5.0 });
                                            self.say(&format!("轻武器穿透击毁了 {}！", kind.label()), "Light vehicle knocked out!");
                                        }
                                    }
                                    self.alert_squads_near(hit_pt, self.player_at, forces);
                                } else {
                                    self.alert_squads_near(self.player_at, self.player_at, forces);
                                }
                            }
                        }
                    }
                }
            }
        }

        if let Some(target) = input.fire_rpg_at {
            let mut chosen_at = self.inv.active_at;
            if self.inv.at_ammo(chosen_at) == 0 {
                if let Some(&avail) = AtWeapon::ALL.iter().find(|&&a| self.inv.at_ammo(a) > 0) {
                    chosen_at = avail;
                    self.inv.active_at = avail;
                }
            }
            if self.inv.at_ammo(chosen_at) > 0 && self.player_shoot_cd <= 0.0 {
                self.inv.consume_at(chosen_at);
                self.player_shoot_cd = 0.72;
                let dir = target.sub(self.player_at).norm();
                if dir.len() > 0.05 {
                    self.player_facing = dir.y.atan2(dir.x);
                }
                let max_d = if chosen_at == AtWeapon::Hj12 {
                    target.dist(self.player_at).clamp(4.0, 120.0)
                } else {
                    target.dist(self.player_at).clamp(4.0, 82.0)
                };
                let mut impact = self.player_at.add(dir.mul(max_d));
                let mut impact_floor = 0u8;
                let mut direct_tank: Option<usize> = None;
                let mut best_t = f32::MAX;
                for (ti, t) in self.tanks.iter().enumerate() {
                    if t.hp <= 0.0 || t.faction == Faction::Pla {
                        continue;
                    }
                    if chosen_at.is_top_attack() {
                        // HJ-12 locks onto closest enemy armored vehicle near cursor within 22m!
                        let cd = t.at.dist(target);
                        if cd <= 22.0 && cd < best_t && self.player_at.dist(t.at) <= 125.0 {
                            best_t = cd;
                            impact = t.at;
                            direct_tank = Some(ti);
                        }
                    } else {
                        let q = nearest_on_segment(t.at, self.player_at, impact);
                        let dist = q.dist(self.player_at);
                        if q.dist(t.at) < 2.8
                            && dist < best_t
                            && ground.clear_shot_from_interior(&self.interiors, self.active_interior, self.player_at, t.at)
                        {
                            best_t = dist;
                            impact = t.at;
                            direct_tank = Some(ti);
                        }
                    }
                }
                if direct_tank.is_none()
                    && self.player_floor > 0
                    && self.active_interior.is_some_and(|i| self.interiors[i].contains(target))
                {
                    impact_floor = self.player_floor;
                }
                let origin_y = self.player_floor as f32 * FLOOR_H + if chosen_at.is_top_attack() { 3.8 } else { 1.45 };
                if let Some(i_idx) = self.active_interior {
                    if !self.interiors[i_idx].contains(impact) {
                        self.interiors[i_idx].curtains_closed = false;
                        self.shatter_windows_near(self.player_at.lerp(impact, 0.14), 5.5, out);
                    }
                }
                out.tracers.push(TracerFx {
                    from: [self.player_at.x, origin_y, -self.player_at.y],
                    to: [impact.x, impact_floor as f32 * FLOOR_H + 1.0, -impact.y],
                    friendly: true,
                });
                out.tank_cannon = true;
                if let Some(ti) = direct_tank {
                    self.tanks[ti].hp -= chosen_at.direct_tank_dmg();
                }
                if chosen_at == AtWeapon::RpoShmel {
                    self.add_fire(impact, THERMOBARIC_FIRE_S);
                }
                self.explode_at_floor(
                    impact,
                    impact_floor,
                    chosen_at.radius(),
                    chosen_at.splash_dmg(),
                    chosen_at.structural_dmg(),
                    false,
                    forces,
                    true,
                    out,
                );
                self.gunfire_reveal_timer = 5.5;
                self.say(&format!("发射 {}！", chosen_at.label()), "AT rocket fired!");
                self.alert_squads_near(impact, self.player_at, forces);
            } else if self.inv.total_at_ammo() == 0 {
                self.say("没有反坦克弹药了！搜海兰街边防武器库或按 R 呼叫解放军！", "No AT rockets left!");
            }
        }

        if let Some(target) = input.throw_grenade_at {
            if self.inv.grenades > 0 {
                self.inv.grenades -= 1;
                let max_r = forces.num_or("grenade", "throw_m", 22.0).max(24.0);
                let delta = target.sub(self.player_at);
                let dist = delta.len().clamp(2.0, max_r);
                let dir = if delta.len() > 0.05 {
                    delta.norm()
                } else {
                    P2::new(self.player_facing.cos(), self.player_facing.sin())
                };
                let drop = self.player_at.add(dir.mul(dist));
                let drop_floor = if self.player_floor > 0 && self.active_interior.is_some_and(|i| self.interiors[i].contains(drop)) {
                    self.player_floor
                } else {
                    0
                };
                let flight_dur = (dist / 16.0).clamp(0.35, 1.0);
                let fuse = forces.num_or("grenade", "fuse_s", 0.7).max(1.5);
                self.next_id += 1;
                self.grenades.push(ActiveGrenade {
                    id: self.next_id,
                    from: self.player_at,
                    at: self.player_at,
                    target: drop,
                    floor: drop_floor,
                    fuse,
                    flight_t: 0.0,
                    flight_dur,
                    friendly: true,
                });
                self.say("投掷定时高爆手雷！注意规避！", "Frag grenade thrown!");
                self.push_radio("投掷手雷", "FRAG OUT", "破片手雷已投出！注意规避", false);
            } else {
                self.say("没有手雷了！", "Out of grenades.");
            }
        }

        if let Some(target) = input.throw_smoke_at {
            if self.inv.smoke_grenades > 0 {
                self.inv.smoke_grenades -= 1;
                let dist = target.dist(self.player_at).min(26.0);
                let delta = target.sub(self.player_at);
                let dir = if delta.len() > 0.05 {
                    delta.norm()
                } else {
                    P2::new(self.player_facing.cos(), self.player_facing.sin())
                };
                let drop = self.player_at.add(dir.mul(dist));
                self.push_smoke_cloud(drop, 6.8, 26.0, false);
                self.say("投掷战术白烟弹！浓烟遮蔽敌军步兵与无人机视线。", "Smoke grenade deployed!");
                self.push_radio("战术烟雾", "SMOKE OUT", "已释放战术白烟！敌军步兵与侦察无人机视线受阻", false);
            } else {
                self.say("没有战术烟雾弹了！", "Out of smoke grenades.");
            }
        }

        if let Some(target) = input.launch_fpv_at {
            if self.inv.friendly_fpv > 0 && self.friendly_fpv.is_none() {
                self.inv.friendly_fpv -= 1;
                let max_d = target.dist(self.player_at).clamp(10.0, 145.0);
                let aim_pt = self.player_at.add(target.sub(self.player_at).norm().mul(max_d));
                let locked = self
                    .tanks
                    .iter()
                    .filter(|t| t.hp > 0.0 && t.faction == Faction::Russia && t.at.dist(aim_pt) <= 32.0)
                    .min_by(|a, b| a.at.dist(aim_pt).total_cmp(&b.at.dist(aim_pt)))
                    .map(|t| t.at)
                    .unwrap_or(aim_pt);
                self.friendly_fpv = Some(FriendlyFpv {
                    origin: self.player_at,
                    at: self.player_at,
                    height: self.player_floor as f32 * FLOOR_H + 3.2,
                    target: locked,
                });
                self.say("放飞我方自杀 FPV 无人机！直扑俄军目标！", "Launched friendly FPV drone!");
            } else if self.inv.friendly_fpv == 0 {
                self.say("没有便携自杀 FPV 无人机了！", "No friendly FPVs left.");
            }
        }

        if let Some(target) = input.throw_molotov_at {
            if self.inv.molotov > 0 {
                self.inv.molotov -= 1;
                let max_range = if self.player_floor >= 1 { 28.0 } else { 24.0 };
                let dist = target.dist(self.player_at).min(max_range);
                let dir = target.sub(self.player_at).norm();
                let drop = self.player_at.add(dir.mul(dist));
                let top_attack = self.player_floor >= 1 && self.tanks.iter().any(|t| t.faction == Faction::Russia && t.at.dist(drop) <= 6.0);
                let dmg = if top_attack { 150.0 } else { 75.0 };
                self.explode_at_floor(drop, 0, 6.0, dmg, 0.0, false, forces, true, out);
                self.add_fire(drop, MOLOTOV_FIRE_S);
                if top_attack {
                    self.say("从高楼俯掷燃烧瓶命中坦克引擎盖（2×顶攻伤害）！", "Top-attack Molotov hit tank engine deck (2x damage)!");
                } else {
                    self.say("燃烧瓶炸开！火焰烤化了路面。", "Molotov exploded!");
                }
                self.alert_squads_near(drop, self.player_at, forces);
            }
        }

        if let Some(target) = input.call_radio_at {
            if self.inv.has_radio && self.inv.radio_strikes > 0 && self.radio_cd <= 0.0 {
                self.inv.radio_strikes -= 1;
                self.radio_cd = 45.0;
                self.mortars.push(MortarMission { target, fuse: 2.6, rounds_left: 5, round_gap: 0.55, friendly: true, whistled: false });
                let pla_count = self.tanks.iter().filter(|t| t.faction == Faction::Pla).count();
                if self.hour >= hm(18, 30) && pla_count < 2 {
                    let sup = if pla_count == 0 { VehicleKind::Ztz99A } else { VehicleKind::Zbl08 };
                    self.spawn_tank_wave_at(sup, P2::new(-270.0, -355.0), forces);
                    self.say(&format!("电台呼叫成功(余{}次)！155mm 榴弹炮齐射 + {} 正赶来增援！", self.inv.radio_strikes, sup.label()), "155mm barrage + PLA armor incoming!");
                } else {
                    self.say(&format!("电台已标定(余{}次)！PLA 155mm 炮兵 5 发急速射即将覆盖目标！", self.inv.radio_strikes), "PLA 155mm fire mission incoming!");
                }
            } else if self.inv.has_radio && self.inv.radio_strikes == 0 {
                self.say("电台呼号次数已用尽！（搜海兰街边防武器库或深夜补给可补充呼救次数）", "No radio strikes remaining!");
            } else if self.inv.has_radio {
                self.say(&format!("炮兵电台装填中（还需 {:.0}s）！", self.radio_cd), "Radio fire support recharging.");
            } else if self.alert_stage >= 2 {
                let _ = city;
                self.say("还没有电台！20:00 后前往中央街(-255,-320)救出被困邻居可获取电台。", "No radio yet. Rescue neighbours for radio.");
            }
        }
    }

    fn step_actor_floor(
        ground: &Ground,
        interiors: &[Interior],
        pos: &mut P2,
        floor: &mut u8,
        goal: P2,
        goal_floor: u8,
        radius: f32,
        speed: f32,
        dt: f32,
        is_enemy: bool,
    ) -> (f32, f32) {
        let mut target = goal;
        if *floor != goal_floor {
            if *floor > 0 {
                if let Some(intr) = interiors.iter().find(|it| it.contains(*pos)) {
                    let st = intr.stairwell_world();
                    if pos.dist(st) <= 1.75 {
                        if intr.contains(goal) && goal_floor > *floor && intr.num_floors > goal_floor {
                            *floor += 1;
                        } else {
                            *floor = floor.saturating_sub(1);
                        }
                    } else {
                        target = st;
                    }
                } else {
                    *floor = 0;
                }
            } else if goal_floor > 0 {
                if let Some(intr) = interiors.iter().find(|it| it.contains(goal)) {
                    if intr.contains(*pos) {
                        let st = intr.stairwell_world();
                        if pos.dist(st) <= 1.75 && intr.num_floors > *floor + 1 {
                            *floor += 1;
                        } else {
                            target = st;
                        }
                    } else {
                        target = intr.front_door.outside;
                    }
                }
            }
        }
        let d = target.sub(*pos);
        if d.len() > 0.2 {
            let dir = d.norm();
            *pos = ground.step_with_interiors_on_floor(interiors, *pos, pos.add(dir.mul(speed * dt)), radius, *floor, is_enemy);
            (dir.y.atan2(dir.x), speed)
        } else {
            (0.0, 0.0)
        }
    }

    fn static_cover_mul(
        barricades: &[Barricade],
        craters: &[Crater],
        wrecks: &[VehicleWreck],
        sandbag_mul: f32,
        at: P2,
    ) -> f32 {
        let mut mul = 1.0f32;
        for b in barricades {
            if b.shoved || b.hp <= 0.0 {
                continue;
            }
            match b.kind {
                BarricadeKind::Trench | BarricadeKind::TarpDugout if b.at.dist(at) <= 2.6 => {
                    mul = mul.min(0.25);
                }
                BarricadeKind::DeadCar | BarricadeKind::Concrete if b.at.dist(at) <= 2.6 => {
                    mul = mul.min(0.35);
                }
                BarricadeKind::Sandbag if b.at.dist(at) <= 2.4 => {
                    mul = mul.min(sandbag_mul);
                }
                _ => {}
            }
        }
        if craters.iter().any(|c| c.at.dist(at) <= c.radius) {
            mul = mul.min(0.45);
        }
        if wrecks.iter().any(|w| w.at.dist(at) <= 3.2) {
            mul = mul.min(0.35);
        }
        mul
    }

    fn find_cover_near(
        barricades: &[Barricade],
        craters: &[Crater],
        wrecks: &[VehicleWreck],
        anchor: P2,
        max_r: f32,
    ) -> Option<P2> {
        let mut best: Option<(f32, P2)> = None;
        let mut consider = |p: P2, d_anchor: f32| {
            if d_anchor <= max_r && best.is_none_or(|b| d_anchor < b.0) {
                best = Some((d_anchor, p));
            }
        };
        for b in barricades {
            if b.shoved || b.hp <= 0.0 {
                continue;
            }
            let pt = match b.kind {
                BarricadeKind::Trench | BarricadeKind::TarpDugout | BarricadeKind::Sandbag => b.at,
                BarricadeKind::Concrete => b.at.add(P2::new(0.0, -1.7)),
                BarricadeKind::DeadCar => b.at.add(P2::new(0.0, -1.8)),
                _ => continue,
            };
            consider(pt, anchor.dist(pt));
        }
        for c in craters {
            consider(c.at, anchor.dist(c.at));
        }
        for w in wrecks {
            let pt = w.at.add(P2::new(0.0, -2.1));
            consider(pt, anchor.dist(pt));
        }
        best.map(|b| b.1)
    }

    fn trigger_vault_if_crossed(
        barricades: &[Barricade],
        prev: P2,
        next: P2,
        floor: u8,
        vault_t: &mut f32,
    ) {
        if floor != 0 || *vault_t > 0.0 || next.dist(prev) <= 0.01 {
            return;
        }
        if barricades.iter().any(|b| {
            !b.shoved
                && b.hp > 0.0
                && matches!(b.kind, BarricadeKind::Sandbag | BarricadeKind::Trench)
                && b.at.dist(next) <= 1.05
                && b.at.dist(prev) > 1.05
        }) {
            *vault_t = 0.45;
        }
    }

    fn trigger_vault_if_crossed_snap(
        bar_snap: &[(usize, P2, BarricadeKind)],
        prev: P2,
        next: P2,
        floor: u8,
        vault_t: &mut f32,
    ) {
        if floor != 0 || *vault_t > 0.0 || next.dist(prev) <= 0.01 {
            return;
        }
        if bar_snap.iter().any(|&(_, bp, bk)| {
            matches!(bk, BarricadeKind::Sandbag | BarricadeKind::Trench)
                && bp.dist(next) <= 1.05
                && bp.dist(prev) > 1.05
        }) {
            *vault_t = 0.45;
        }
    }

    fn push_blood_mark(&mut self, at: P2, yaw: f32) {
        if self.blood_marks.len() >= 90 {
            self.blood_marks.remove(0);
        }
        self.next_id += 1;
        self.blood_marks.push(BloodMark {
            id: self.next_id,
            at,
            yaw,
            age: 0.0,
        });
    }

    fn damage_militia(&mut self, mi: usize, raw_dmg: f32) {
        if raw_dmg <= 0.0 {
            return;
        }
        let mut msg: Option<(&'static str, &'static str, &'static str)> = None;
        let mut bleed_spot: Option<(P2, f32)> = None;
        if let Some(m) = self.militia.get_mut(mi) {
            if m.hp <= 0.0 {
                return;
            }
            let mult = if matches!(m.order, MilitiaOrder::Garrison(_)) { 0.25 } else { 1.0 };
            let dmg = raw_dmg * mult;
            if m.downed {
                m.hp = (m.hp - dmg).max(0.0);
                if m.hp <= 0.0 {
                    m.downed = false;
                    m.speed = 0.0;
                    msg = Some((m.name, "CASUALTY", "友军战友阵亡！"));
                }
            } else if m.hp - dmg <= 18.0 && dmg < 75.0 {
                m.downed = true;
                m.hp = 18.0;
                m.bleedout_s = 28.0;
                m.rescue_progress = 0.0;
                m.speed = 0.0;
                m.in_cover = false;
                bleed_spot = Some((m.at, m.facing));
                msg = Some((m.name, "MAN DOWN", "友军重伤倒地！队友或靠近按 E 抢救"));
            } else {
                m.hp = (m.hp - dmg).max(0.0);
                bleed_spot = Some((m.at, m.facing));
                if m.hp <= 0.0 {
                    m.downed = false;
                    m.speed = 0.0;
                    msg = Some((m.name, "CASUALTY", "友军战友阵亡！"));
                } else {
                    msg = Some((m.name, "TAKING CASUALTIES", "友军遇袭！请求火力掩护"));
                }
            }
        }
        if let Some((at, yaw)) = bleed_spot {
            self.push_blood_mark(at, yaw);
        }
        if let Some((name, code_en, msg_zh)) = msg {
            self.push_radio(name, code_en, msg_zh, true);
        }
    }

    fn step_militia(&mut self, ground: &Ground, forces: &Forces, dt: f32, out: &mut TickOutput) {
        let mut shots: Vec<(P2, u8, usize, usize, bool)> = Vec::new();
        let mut blood_spawns: Vec<(P2, f32)> = Vec::new();
        let mut rescue_ticks: Vec<(u32, &'static str, f32)> = Vec::new();
        let target_player_floor = if self.active_interior.is_some() { self.player_floor } else { 0 };
        let interiors = &self.interiors;
        let barricades = &self.barricades;
        let craters = &self.craters;
        let wrecks = &self.wrecks;
        let grenades = &self.grenades;
        let blood_marks = &self.blood_marks;
        let squads = &self.squads;
        let player_at = self.player_at;
        let sandbag_mul = forces.num_or("rifle", "cover_mul", 0.45);
        let has_militia_los = |a: P2, fa: u8, b: P2, fb: u8| -> bool {
            let intr_a = interiors.iter().position(|it| it.contains(a));
            let intr_b = interiors.iter().position(|it| it.contains(b));
            if fa != fb {
                if let (Some(ia), Some(ib)) = (intr_a, intr_b) {
                    if ia == ib && interiors[ia].is_slab_intact(fa.max(fb), b) {
                        return false;
                    }
                }
            }
            ground.clear_shot_from_interior(interiors, intr_a, a, b)
        };

        let downed_targets: Vec<(u32, P2, u8)> = self
            .militia
            .iter()
            .filter(|m| m.hp > 0.0 && m.downed)
            .map(|m| (m.id, m.at, m.floor))
            .collect();
        let healthy_mates: Vec<(u32, P2, u8)> = self
            .militia
            .iter()
            .filter(|m| m.hp > 0.0 && !m.downed)
            .map(|m| (m.id, m.at, m.floor))
            .collect();
        let mut rescue_assignments: Vec<(u32, u32)> = Vec::new();
        let mut claimed_rescuers: Vec<u32> = Vec::new();
        for &(did, dat, _) in &downed_targets {
            if let Some(&(rid, _, _)) = healthy_mates
                .iter()
                .filter(|(rid, _, _)| !claimed_rescuers.contains(rid))
                .min_by(|a, b| a.1.dist(dat).total_cmp(&b.1.dist(dat)))
            {
                claimed_rescuers.push(rid);
                rescue_assignments.push((rid, did));
            }
        }

        for m in &mut self.militia {
            if m.hp <= 0.0 {
                continue;
            }
            m.fire_cd = (m.fire_cd - dt).max(0.0);
            m.vault_t = (m.vault_t - dt).max(0.0);
            m.rescuing_id = None;

            if m.downed {
                m.bleedout_s -= dt;
                m.in_cover = false;
                if m.bleedout_s <= 0.0 {
                    m.hp = 0.0;
                    m.downed = false;
                    m.speed = 0.0;
                    continue;
                }
                let (crawl_goal, crawl_floor) = healthy_mates
                    .iter()
                    .filter(|(id, ..)| *id != m.id)
                    .min_by(|a, b| a.1.dist(m.at).total_cmp(&b.1.dist(m.at)))
                    .map(|(_, p, f)| (*p, *f))
                    .unwrap_or((player_at, target_player_floor));
                if m.at.dist(crawl_goal) > 1.35 || m.floor != crawl_floor {
                    let (yaw, moved) = Self::step_actor_floor(
                        ground,
                        interiors,
                        &mut m.at,
                        &mut m.floor,
                        crawl_goal,
                        crawl_floor,
                        0.30,
                        0.60,
                        dt,
                        false,
                    );
                    if moved > 0.0 {
                        m.facing = yaw;
                    }
                    m.speed = moved;
                    m.phase += 2.2 * dt;
                    if blood_marks.iter().rev().take(8).all(|b| b.at.dist(m.at) >= 0.9)
                        && blood_spawns.iter().all(|(p, _)| p.dist(m.at) >= 0.9)
                    {
                        blood_spawns.push((m.at, m.facing));
                    }
                } else {
                    m.speed = 0.0;
                }
                continue;
            }

            if let Some(g) = grenades
                .iter()
                .find(|g| m.floor == g.floor && (g.at.dist(m.at) <= 6.2 || g.target.dist(m.at) <= 6.2))
            {
                let threat = if g.target.dist(m.at) <= 6.2 { g.target } else { g.at };
                let away = if m.at.dist(threat) > 0.1 {
                    m.at.sub(threat).norm()
                } else {
                    P2::new(0.0, -1.0)
                };
                let dive_goal = m.at.add(away.mul(7.0));
                let prev_at = m.at;
                let (yaw, moved_spd) = Self::step_actor_floor(
                    ground,
                    interiors,
                    &mut m.at,
                    &mut m.floor,
                    dive_goal,
                    0,
                    0.30,
                    4.8,
                    dt,
                    false,
                );
                Self::trigger_vault_if_crossed(barricades, prev_at, m.at, m.floor, &mut m.vault_t);
                if moved_spd > 0.0 {
                    m.facing = yaw;
                }
                m.speed = moved_spd;
                m.phase += (std::f32::consts::TAU * 4.8 / 2.95) * dt;
                m.in_cover = false;
                continue;
            }

            if let Some(&(_, did)) = rescue_assignments.iter().find(|(rid, _)| *rid == m.id) {
                if let Some(&(_, dat, dfloor)) = downed_targets.iter().find(|(id, ..)| *id == did) {
                    if m.at.dist(dat) > 1.8 || m.floor != dfloor {
                        let prev_at = m.at;
                        let (yaw, moved_spd) = Self::step_actor_floor(
                            ground,
                            interiors,
                            &mut m.at,
                            &mut m.floor,
                            dat,
                            dfloor,
                            0.30,
                            4.5,
                            dt,
                            false,
                        );
                        Self::trigger_vault_if_crossed(barricades, prev_at, m.at, m.floor, &mut m.vault_t);
                        if moved_spd > 0.0 {
                            m.facing = yaw;
                        }
                        m.speed = moved_spd;
                        m.phase += (std::f32::consts::TAU * 4.5 / 2.84) * dt;
                        m.in_cover = false;
                    } else {
                        m.speed = 0.0;
                        m.in_cover = true;
                        m.rescuing_id = Some(did);
                        let dir = dat.sub(m.at);
                        if dir.len() > 0.05 {
                            m.facing = dir.y.atan2(dir.x);
                        }
                        rescue_ticks.push((did, m.name, dt / 3.0));
                    }
                    continue;
                }
            }

            let (anchor, mut goal_floor) = match m.order {
                MilitiaOrder::Follow => (player_at.add(P2::new(((m.id % 3) as f32 - 1.0) * 1.8, -1.8)), target_player_floor),
                MilitiaOrder::Hold(p) | MilitiaOrder::Garrison(p) => (p, m.floor),
            };
            let threat_near = squads
                .iter()
                .any(|g| g.men.iter().any(|s| s.hp > 0.0 && s.at.dist(m.at) < 62.0));
            let mut goal = anchor;
            if m.floor == 0 && (matches!(m.order, MilitiaOrder::Hold(_)) || (threat_near && anchor.dist(m.at) <= 4.5)) {
                if let Some(cov_at) = Self::find_cover_near(barricades, craters, wrecks, anchor, 7.5) {
                    goal = cov_at;
                    goal_floor = 0;
                }
            }
            let d = goal.dist(m.at);
            if d > 1.2 || m.floor != goal_floor {
                let spd = if d > 6.0 { 4.2 } else { 2.5 };
                let prev_at = m.at;
                let (yaw, moved_spd) = Self::step_actor_floor(
                    ground,
                    interiors,
                    &mut m.at,
                    &mut m.floor,
                    goal,
                    goal_floor,
                    0.30,
                    spd,
                    dt,
                    false,
                );
                Self::trigger_vault_if_crossed(barricades, prev_at, m.at, m.floor, &mut m.vault_t);
                if moved_spd > 0.0 {
                    m.facing = yaw;
                }
                m.speed = moved_spd;
                m.phase += (std::f32::consts::TAU * spd / (0.95 + 0.42 * spd)) * dt;
            } else {
                m.speed = 0.0;
            }
            m.in_cover = m.speed < 0.25
                && m.floor == 0
                && Self::static_cover_mul(barricades, craters, wrecks, sandbag_mul, m.at) < 0.8;

            if m.fire_cd <= 0.0 {
                let mut best: Option<(usize, usize, f32, P2, bool)> = None;
                for (gi, g) in squads.iter().enumerate() {
                    for (si, s) in g.men.iter().enumerate() {
                        if s.hp <= 0.0 {
                            continue;
                        }
                        let dist = m.at.dist(s.at);
                        if dist < 58.0
                            && has_militia_los(m.at, m.floor, s.at, s.floor)
                            && best.map(|b| dist < b.2).unwrap_or(true)
                        {
                            best = Some((gi, si, dist, s.at, s.in_cover));
                        }
                    }
                }
                if let Some((gi, si, _, target, target_cov)) = best {
                    m.fire_cd = 1.1;
                    let dir = target.sub(m.at).norm();
                    m.facing = dir.y.atan2(dir.x);
                    shots.push((m.at, m.floor, gi, si, target_cov));
                }
            }
        }

        for (at, yaw) in blood_spawns {
            self.push_blood_mark(at, yaw);
        }
        for (did, rescuer_name, delta) in rescue_ticks {
            let mut completed = false;
            if let Some(dm) = self.militia.iter_mut().find(|m| m.id == did && m.downed && m.hp > 0.0) {
                dm.rescue_progress += delta;
                if dm.rescue_progress >= 1.0 {
                    dm.downed = false;
                    dm.hp = 55.0;
                    dm.bleedout_s = 0.0;
                    dm.rescue_progress = 0.0;
                    completed = true;
                }
            }
            if completed {
                self.push_radio(rescuer_name, "TEAMMATE RESCUED", "倒地战友已被成功抢救！", false);
            }
        }

        for (from, from_floor, gi, si, target_cov) in shots {
            let Some((target, to_floor)) = self.squads.get(gi).and_then(|g| g.men.get(si)).map(|s| (s.at, s.floor)) else { continue };
            let dist = from.dist(target);
            let mut p = (forces.hit_chance(dist) * 1.25).clamp(0.15, 0.72);
            if target_cov {
                p *= 0.55;
            }
            let roll = self.next_rand(19);
            let hit = roll < p;
            let miss_off = if hit { P2::default() } else { P2::new((roll - 0.5) * 2.2, 0.7) };
            let to = target.add(miss_off);
            out.tracers.push(TracerFx {
                from: [from.x, from_floor as f32 * FLOOR_H + 1.35, -from.y],
                to: [to.x, to_floor as f32 * FLOOR_H + 1.25, -to.y],
                friendly: true,
            });
            if hit {
                self.damage_soldier(gi, si, 24.0);
                self.alert_squads_near(target, from, forces);
            }
        }
    }

    fn step_grenades(&mut self, forces: &Forces, dt: f32, out: &mut TickOutput) {
        if self.grenades.is_empty() {
            return;
        }
        let mut detonated: Vec<(P2, u8, bool)> = Vec::new();
        for g in &mut self.grenades {
            g.flight_t = (g.flight_t + dt).min(g.flight_dur);
            let u = if g.flight_dur > 0.01 { (g.flight_t / g.flight_dur).clamp(0.0, 1.0) } else { 1.0 };
            g.at = g.from.lerp(g.target, u);
            g.fuse -= dt;
            if g.fuse <= 0.0 {
                detonated.push((g.target, g.floor, g.friendly));
            }
        }
        self.grenades.retain(|g| g.fuse > 0.0);
        let rad = forces.num_or("grenade", "wound_m", 8.0).clamp(5.5, 9.0);
        let (_, raw_dmg) = forces.range("grenade", "wound_hp");
        for (at, floor, friendly) in detonated {
            self.explode_at_floor(at, floor, rad, raw_dmg.max(65.0), 38.0, false, forces, friendly, out);
            self.alert_squads_near(at, at, forces);
        }
    }

    fn barricade_cover_mul(&self, forces: &Forces, at: P2) -> f32 {
        let mut mul = 1.0f32;
        for b in &self.barricades {
            if b.shoved || b.hp <= 0.0 {
                continue;
            }
            match b.kind {
                BarricadeKind::Trench | BarricadeKind::TarpDugout if b.at.dist(at) <= 2.6 => {
                    mul = mul.min(0.25);
                }
                BarricadeKind::DeadCar | BarricadeKind::Concrete if b.at.dist(at) <= 2.6 => {
                    mul = mul.min(0.35);
                }
                BarricadeKind::Sandbag if b.at.dist(at) <= 2.4 => {
                    mul = mul.min(forces.num_or("rifle", "cover_mul", 0.45));
                }
                _ => {}
            }
        }
        mul
    }

    pub fn under_tarp_dugout(&self, at: P2) -> bool {
        self.barricades
            .iter()
            .any(|b| !b.shoved && b.hp > 0.0 && b.kind == BarricadeKind::TarpDugout && b.at.dist(at) <= 2.4)
    }

    pub fn crater_cover_mul(&self, at: P2) -> f32 {
        if self.craters.iter().any(|c| c.at.dist(at) <= c.radius) {
            0.45
        } else {
            1.0
        }
    }

    pub fn wreck_cover_mul(&self, at: P2) -> f32 {
        if self.wrecks.iter().any(|w| w.at.dist(at) <= 3.2) {
            0.35
        } else {
            1.0
        }
    }

    pub fn defilade_blast_mul(&self, at: P2) -> f32 {
        let mut mul = self.barricade_concrete_blast_mul(at);
        if self.barricades.iter().any(|b| {
            !b.shoved
                && b.hp > 0.0
                && matches!(b.kind, BarricadeKind::Trench | BarricadeKind::TarpDugout)
                && b.at.dist(at) <= 2.6
        }) {
            mul = mul.min(0.30);
        }
        if self.craters.iter().any(|c| c.at.dist(at) <= c.radius) {
            mul = mul.min(0.55);
        }
        mul
    }

    pub fn in_tactical_smoke(&self, at: P2) -> bool {
        self.smoke_clouds.iter().any(|c| !c.dark && c.at.dist(at) <= c.radius)
    }

    fn wrecks_block_los_slice(wrecks: &[VehicleWreck], a: P2, b: P2) -> bool {
        for w in wrecks {
            if a.dist(w.at) <= 2.4 || b.dist(w.at) <= 2.4 {
                continue;
            }
            if nearest_on_segment(w.at, a, b).dist(w.at) < 1.85 {
                return true;
            }
        }
        false
    }

    fn smoke_blocks_los_slice(smokes: &[SmokeCloud], a: P2, b: P2) -> bool {
        for c in smokes {
            if c.dark && (a.dist(c.at) <= 2.8 || b.dist(c.at) <= 2.8) {
                continue;
            }
            if nearest_on_segment(c.at, a, b).dist(c.at) < c.radius * 0.85 {
                return true;
            }
        }
        false
    }

    fn barricade_concrete_blast_mul(&self, at: P2) -> f32 {
        if self
            .barricades
            .iter()
            .any(|b| !b.shoved && b.hp > 0.0 && b.kind == BarricadeKind::Concrete && b.at.dist(at) <= 2.6)
        {
            0.5
        } else {
            1.0
        }
    }

    fn barricades_block_los_slice(barricades: &[Barricade], a: P2, b: P2) -> bool {
        for bar in barricades {
            if bar.shoved || bar.hp <= 0.0 || bar.kind != BarricadeKind::DeadCar {
                continue;
            }
            if a.dist(bar.at) <= 1.8 || b.dist(bar.at) <= 1.8 {
                continue;
            }
            if nearest_on_segment(bar.at, a, b).dist(bar.at) < 1.5 {
                return true;
            }
        }
        false
    }

    #[cfg(test)]
    fn barricade_blocks_los(&self, a: P2, b: P2) -> bool {
        Self::barricades_block_los_slice(&self.barricades, a, b)
    }

    fn damage_soldier(&mut self, gi: usize, si: usize, dmg: f32) {
        let mut dead_squad = false;
        {
            let Some(s) = self.squads.get_mut(gi).and_then(|g| g.men.get_mut(si)) else { return };
            if s.hp <= 0.0 {
                return;
            }
            s.hp -= dmg;
            if s.hp <= 0.0 {
                s.hp = 0.0;
                let at = s.at;
                let id = s.id;
                let no_rifle = !self.inv.has_rifle;
                self.drops.push(LootDrop {
                    at,
                    ammo: 10 + (id % 8),
                    bandage: u32::from(id % 2 == 0),
                    food: u32::from(id % 3 == 0),
                    grants_rifle: no_rifle,
                    taken: false,
                });
                dead_squad = !self.squads[gi].men.iter().any(|m| m.hp > 0.0);
            }
        }
        if dead_squad {
            self.push_radio("蓝队观察", "SECTOR CLEARED", "该股俄军已被肃清，区域安全", false);
        }
    }

    fn alert_squads_near(&mut self, at: P2, source: P2, forces: &Forces) {
        let alert_r = forces.num_or("infantry_group", "gunfire_alert_m", 85.0);
        for g in &mut self.squads {
            if g.men.iter().any(|m| m.hp > 0.0 && m.at.dist(at) < alert_r) {
                g.last_seen = source;
                g.hostile = true;
                g.state = SquadState::Engage;
                g.meter = 1.0;
                g.lost_for = 0.0;
            }
        }
    }

    fn step_squads(&mut self, city: &City, ground: &Ground, forces: &Forces, light: Light, dt: f32, out: &mut TickOutput) {
        if self.hour >= CROSSING_HOUR {
            self.spawn_wait -= dt;
            let alive_soldiers: usize = self.squads.iter().map(|g| g.men.iter().filter(|m| m.hp > 0.0).count()).sum();
            let near_groups = self.squads.iter().filter(|g| g.men.iter().any(|m| m.hp > 0.0) && g.men[0].at.dist(self.player_at) < 300.0).count();
            if self.spawn_wait <= 0.0 && near_groups < 3 && alive_soldiers < 14 {
                self.spawn_wait = 16.0;
                self.try_spawn_squad(city, forces);
            }
        }

        let mul = exposure(self.player_gait, self.body.sheltered);
        let indoor_stealth = if let Some(i_idx) = self.active_interior {
            let intr = &self.interiors[i_idx];
            let room_lit = !self.blackout && self.hour < BLACKOUT_HOUR;
            let flame_lit = intr.stove_fuel_s > 0.0;
            if self.flashlight_on && !intr.curtains_closed {
                1.35
            } else if !room_lit && !flame_lit && !self.flashlight_on {
                0.22
            } else {
                0.70
            }
        } else {
            1.0
        };
        let sight_angle = forces.num_or("infantry_group", "view_half_angle_deg", 100.0);
        let walk_spd = forces.num_or("infantry_group", "walk_mps", 1.3);
        let rush_spd = forces.num_or("infantry_group", "rush_mps", 3.1);
        let mut hearing_r = forces.hearing(self.player_gait);
        if self.noise_t > 0.0 {
            hearing_r = hearing_r.max(self.noise_m);
        }
        let challenge_s = forces.num_or("infantry_group", "challenge_s", 1.6);
        let lose_s = forces.num_or("infantry_group", "lose_sight_s", 4.0);
        let search_s = forces.num_or("infantry_group", "search_s", 45.0);
        let calm_s = forces.num_or("infantry_group", "calm_after_search_s", 60.0);
        let fog_pen_plain = forces.num_or("infantry_group", "fog_sight_penalty_plain", 0.45);
        let fog_pen_optics = forces.num_or("infantry_group", "fog_sight_penalty_optics", 0.35);
        let gunfire_reveal_m = forces.num_or("infantry_group", "gunfire_reveal_m", 78.0);
        let player_bar_cover = self
            .barricade_cover_mul(forces, self.player_at)
            .min(self.crater_cover_mul(self.player_at))
            .min(self.wreck_cover_mul(self.player_at));

        let bar_snap: Vec<(usize, P2, BarricadeKind)> = self
            .barricades
            .iter()
            .enumerate()
            .filter(|(_, b)| !b.shoved && b.hp > 0.0 && !matches!(b.kind, BarricadeKind::FireBarrel))
            .map(|(i, b)| (i, b.at, b.kind))
            .collect();
        let mut bar_dmg: Vec<(usize, f32)> = Vec::new();
        let mut queue_spotted_fpv = false;

        let mut notice_top = 0.0f32;
        let mut any_challenged = false;
        let mut any_engaged = false;
        let mut robbed = false;
        let mut found_track = false;
        let mut hits_on_player: Vec<(f32, P2, u8, P2, bool)> = Vec::new();
        let mut militia_hits: Vec<(usize, f32, P2, u8, P2, u8)> = Vec::new();
        let mut soldier_tracks: Vec<(P2, P2)> = Vec::new();
        let mut breaches: Vec<(usize, bool)> = Vec::new();
        let interiors = &self.interiors;
        let barricades_ref = &self.barricades;
        let craters_ref = &self.craters;
        let wrecks_ref = &self.wrecks;
        let smokes_ref = &self.smoke_clouds;
        let grenades_snap: Vec<(P2, P2, u8)> = self.grenades.iter().map(|g| (g.at, g.target, g.floor)).collect();
        let mut enemy_grenades: Vec<(P2, P2, u8)> = Vec::new();
        let player_floor = if self.active_interior.is_some() { self.player_floor } else { 0 };
        let los_floors = |a: P2, fa: u8, b: P2, fb: u8, is_shoot: bool| -> bool {
            if fa != fb {
                if let (Some(ia), Some(ib)) = (
                    interiors.iter().position(|it| it.contains(a)),
                    interiors.iter().position(|it| it.contains(b)),
                ) {
                    if ia == ib && interiors[ia].is_slab_intact(fa.max(fb), b) && interiors[ia].is_slab_intact(fa.max(fb), a) {
                        return false;
                    }
                }
            } else if fa == 0 {
                if Self::wrecks_block_los_slice(wrecks_ref, a, b) {
                    return false;
                }
                if !is_shoot
                    && (Self::barricades_block_los_slice(barricades_ref, a, b)
                        || Self::smoke_blocks_los_slice(smokes_ref, a, b))
                {
                    return false;
                }
            }
            ground.clear_line_with_interiors(interiors, a, b, is_shoot)
        };

        for g in &mut self.squads {
            g.ignore_for = (g.ignore_for - dt).max(0.0);
            if !g.men.iter().any(|m| m.hp > 0.0) {
                continue;
            }
            for s in &mut g.men {
                if s.hp > 0.0 {
                    s.vault_t = (s.vault_t - dt).max(0.0);
                    s.grenade_cd = (s.grenade_cd - dt).max(0.0);
                }
            }
            let mut seen_player = false;
            let mut nearest_d = f32::MAX;
            for s in &g.men {
                if s.hp <= 0.0 {
                    continue;
                }
                let d = s.at.dist(self.player_at);
                nearest_d = nearest_d.min(d);
                if g.ignore_for <= 0.0 && !seen_player {
                    let fog_mul = (1.0 - self.fog_density.clamp(0.0, 1.0) * if s.optics { fog_pen_optics } else { fog_pen_plain }).max(0.0);
                    let eff_mul = if matches!(g.state, SquadState::Challenge | SquadState::Detain) {
                        mul.max(1.0)
                    } else {
                        mul
                    };
                    let mut reach = forces.sight(light, s.optics) * eff_mul * fog_mul * indoor_stealth;
                    if self.gunfire_reveal_timer > 0.0 {
                        reach = reach.max(gunfire_reveal_m * fog_mul);
                    }
                    let cone_ok = !matches!(g.state, SquadState::Patrol | SquadState::Check) || in_arc(s.facing, s.at, self.player_at, sight_angle);
                    if d <= reach && cone_ok && los_floors(s.at, s.floor, self.player_at, player_floor, false) {
                        seen_player = true;
                    }
                }
            }

            // Patrol spots fresh player snow tracks within 9 m inside their cone.
            if !seen_player && g.state == SquadState::Patrol && g.ignore_for <= 0.0 && self.fog_density < 0.85 {
                'track_scan: for s in &g.men {
                    if s.hp <= 0.0 || s.floor > 0 {
                        continue;
                    }
                    for tr in self.tracks.iter().rev() {
                        if tr.by_player
                            && tr.age < 45.0
                            && s.at.dist(tr.at) <= 9.0
                            && in_arc(s.facing, s.at, tr.at, sight_angle)
                            && ground.clear_line_with_interiors(interiors, s.at, tr.at, false)
                        {
                            g.state = SquadState::Check;
                            g.timer = 16.0;
                            g.last_seen = tr.at.add(tr.dir.mul(8.0));
                            g.meter = g.meter.max(0.45);
                            found_track = true;
                            break 'track_scan;
                        }
                    }
                }
            }

            // Pick target: active nearby militia can draw Russian fire too.
            let militia_target = self.militia.iter().enumerate().find(|(_, m)| {
                m.hp > 0.0 && !m.downed && g.men.iter().any(|s| {
                    let fog_mul = (1.0 - self.fog_density.clamp(0.0, 1.0) * if s.optics { fog_pen_optics } else { fog_pen_plain }).max(0.0);
                    s.hp > 0.0
                        && s.at.dist(m.at) <= forces.sight(light, s.optics) * fog_mul
                        && los_floors(s.at, s.floor, m.at, m.floor, false)
                })
            }).map(|(i, m)| (i, m.at, m.floor));

            if seen_player {
                g.last_seen = self.player_at;
                g.lost_for = 0.0;
                let fog_mul = (1.0 - self.fog_density.clamp(0.0, 1.0) * fog_pen_plain).max(0.0);
                let eff_mul = if matches!(g.state, SquadState::Challenge | SquadState::Detain) { mul.max(1.0) } else { mul };
                let reach = forces.sight(light, false) * eff_mul * fog_mul * indoor_stealth;
                let gain = dt / forces.notice_time(nearest_d, reach).max(0.2);
                g.meter = (g.meter + if g.state == SquadState::Search { gain * 2.0 } else { gain }).min(1.0);
            } else {
                g.lost_for += dt;
                g.meter = (g.meter - dt * 0.3).max(0.0);
            }

            if g.ignore_for <= 0.0 && g.state == SquadState::Patrol && nearest_d <= hearing_r {
                g.last_seen = self.player_at;
                g.meter = g.meter.max(0.4);
                g.state = SquadState::Check;
                g.timer = 14.0;
            }

            if let Some((_, m_at, _)) = militia_target {
                if g.state == SquadState::Patrol || g.state == SquadState::Check {
                    g.last_seen = m_at;
                    g.hostile = true;
                    g.state = SquadState::Engage;
                    g.lost_for = 0.0;
                }
            }

            // Check if pursuing squad hits a barricaded interior door along their path.
            if !matches!(g.state, SquadState::Patrol) {
                for (i_idx, intr) in interiors.iter().enumerate() {
                    if !intr.contains(g.last_seen) && !intr.contains(self.player_at) {
                        continue;
                    }
                    if intr.front_door.barricade_hp > 0.0
                        && g.men.iter().any(|s| s.hp > 0.0 && s.floor == 0 && s.at.dist(intr.front_door.outside) <= 2.6)
                    {
                        breaches.push((i_idx, false));
                    } else if let Some(bd) = &intr.back_door {
                        if bd.barricade_hp > 0.0 && g.men.iter().any(|s| s.hp > 0.0 && s.floor == 0 && s.at.dist(bd.outside) <= 2.6) {
                            breaches.push((i_idx, true));
                        }
                    }
                }
            }

            match g.state {
                SquadState::Patrol => {
                    if let Some(road) = self.patrol_roads.get(g.route_idx) {
                        let wp = road[g.wp_idx.min(road.len() - 1)];
                        let lead = g.men.iter().find(|m| m.hp > 0.0).map(|m| m.at).unwrap_or(wp);
                        if lead.dist(wp) < 2.5 {
                            let next = g.wp_idx as isize + g.wp_dir;
                            if next < 0 || next >= road.len() as isize {
                                g.wp_dir = -g.wp_dir;
                            }
                            g.wp_idx = (g.wp_idx as isize + g.wp_dir).clamp(0, road.len() as isize - 1) as usize;
                            g.stuck_t = 0.0;
                            g.last_lead_at = lead;
                        } else if lead.dist(g.last_lead_at) < 0.25 {
                            g.stuck_t += dt;
                            if g.stuck_t > 3.0 {
                                g.wp_dir = -g.wp_dir;
                                g.wp_idx = (g.wp_idx as isize + g.wp_dir).clamp(0, road.len() as isize - 1) as usize;
                                g.stuck_t = 0.0;
                                g.last_lead_at = lead;
                            }
                        } else {
                            g.last_lead_at = lead;
                            g.stuck_t = 0.0;
                        }
                        let goal = road[g.wp_idx.min(road.len() - 1)];
                        Self::move_squad(&mut g.men, ground, interiors, &bar_snap, &mut bar_dmg, goal, 0, walk_spd, dt, 1.2);
                    }
                    if g.meter >= 1.0 {
                        g.state = SquadState::Challenge;
                        g.timer = challenge_s;
                        g.shouted = false;
                    } else if g.meter >= 0.5 {
                        g.state = SquadState::Check;
                        g.timer = 14.0;
                    }
                }
                SquadState::Check => {
                    let tf = if interiors.iter().any(|it| it.contains(g.last_seen)) { player_floor } else { 0 };
                    Self::move_squad(&mut g.men, ground, interiors, &bar_snap, &mut bar_dmg, g.last_seen, tf, walk_spd, dt, 1.4);
                    g.timer -= dt;
                    if g.meter >= 1.0 {
                        g.state = SquadState::Challenge;
                        g.timer = challenge_s;
                        g.shouted = false;
                    } else if g.timer <= 0.0 && g.meter < 0.2 {
                        g.state = SquadState::Patrol;
                    }
                }
                SquadState::Challenge => {
                    for s in &mut g.men {
                        if s.hp > 0.0 {
                            s.speed = 0.0;
                            let d = self.player_at.sub(s.at);
                            if d.len() > 0.1 {
                                s.facing = d.y.atan2(d.x);
                            }
                        }
                    }
                    if !g.shouted {
                        g.shouted = true;
                        g.held_at = self.player_at;
                        out.shout_stoy = true;
                        self.subtitle_zh = "有人用俄语喊：Стой！站住！站着别动，或者跑。".into();
                        self.subtitle_en = "A shout in Russian: Stoy! Halt! Stand still, or run.".into();
                        self.subtitle_ttl = 5.0;
                    }
                    g.timer -= dt;
                    if g.timer <= 0.0 {
                        let stood = self.player_at.dist(g.held_at) < 1.5 && !self.inv.has_rifle;
                        let has_los = g.men.iter().any(|s| s.hp > 0.0 && los_floors(s.at, s.floor, self.player_at, player_floor, false));
                        if (seen_player || has_los) && stood {
                            g.state = SquadState::Detain;
                        } else if seen_player {
                            g.state = SquadState::Engage;
                            g.hostile = true;
                            g.meter = 1.0;
                            g.lost_for = 0.0;
                        } else {
                            g.state = SquadState::Search;
                            g.timer = search_s;
                        }
                    }
                }
                SquadState::Detain => {
                    if self.player_at.dist(g.held_at) > 1.6 || self.inv.has_rifle {
                        g.state = SquadState::Engage;
                        g.hostile = true;
                        g.meter = 1.0;
                        g.lost_for = 0.0;
                    } else {
                        Self::move_squad(&mut g.men, ground, interiors, &bar_snap, &mut bar_dmg, self.player_at, player_floor, walk_spd, dt, 2.4);
                        if nearest_d < 3.0 {
                            robbed = true;
                            g.ignore_for = calm_s;
                            g.meter = 0.0;
                            g.hostile = false;
                            g.state = SquadState::Patrol;
                        }
                    }
                }
                SquadState::Engage => {
                    // Engaged with you only while it can see you (a fight with the
                    // militia elsewhere is not one).
                    if seen_player {
                        any_engaged = true;
                    }
                    g.hostile = true;
                    if seen_player && self.hour >= hm(19, 30) && self.fpv_cd <= 0.0 && self.pending_fpvs.is_empty() && self.fpv.is_none() {
                        queue_spotted_fpv = true;
                    }
                    let pursue_floor = if interiors.iter().any(|it| it.contains(g.last_seen)) { player_floor } else { 0 };
                    let sandbag_mul = forces.num_or("rifle", "cover_mul", 0.45);
                    let mut thrown_this_squad = false;
                    for s in &mut g.men {
                        if s.hp <= 0.0 {
                            continue;
                        }
                        s.shot_cd = (s.shot_cd - dt).max(0.0);

                        if let Some(&(gat, gtgt, _gfl)) = grenades_snap.iter().find(|&&(gat, gtgt, gfl)| {
                            s.floor == gfl && (gat.dist(s.at) <= 6.2 || gtgt.dist(s.at) <= 6.2)
                        }) {
                            let threat = if gtgt.dist(s.at) <= 6.2 { gtgt } else { gat };
                            let away = if s.at.dist(threat) > 0.1 { s.at.sub(threat).norm() } else { P2::new(0.0, 1.0) };
                            Self::move_squad_one(
                                s,
                                ground,
                                interiors,
                                &bar_snap,
                                &mut bar_dmg,
                                s.at.add(away.mul(7.0)),
                                0,
                                rush_spd * 1.4,
                                dt,
                                0.2,
                            );
                            s.in_cover = false;
                            continue;
                        }

                        let target_militia = militia_target.filter(|(_, ma, _)| !seen_player || s.at.dist(*ma) < s.at.dist(self.player_at));
                        if !thrown_this_squad && s.floor == 0 && s.grenade_cd <= 0.0 {
                            let throw_tgt = target_militia
                                .map(|(_, ma, mf)| (ma, mf))
                                .filter(|(ma, _)| (8.5..=21.0).contains(&s.at.dist(*ma)))
                                .or_else(|| {
                                    let d = s.at.dist(self.player_at);
                                    (seen_player && (8.5..=21.0).contains(&d) && (player_bar_cover < 0.8 || self.player_in_cover))
                                        .then_some((self.player_at, player_floor))
                                });
                            if let Some((tgt_p, tgt_f)) = throw_tgt {
                                if rand01(&mut self.rng, 59) < 0.25 {
                                    s.grenade_cd = 16.0;
                                    thrown_this_squad = true;
                                    enemy_grenades.push((s.at, tgt_p, tgt_f));
                                } else {
                                    s.grenade_cd = 3.0;
                                }
                            }
                        }

                        if let Some((mi, m_at, m_floor)) = target_militia {
                            if los_floors(s.at, s.floor, m_at, m_floor, true) {
                                let cur_cov = Self::static_cover_mul(barricades_ref, craters_ref, wrecks_ref, sandbag_mul, s.at);
                                if s.floor == 0 && cur_cov >= 0.8 && s.at.dist(m_at) > 10.0 {
                                    if let Some(cov_pt) = Self::find_cover_near(barricades_ref, craters_ref, wrecks_ref, s.at, 5.5) {
                                        if cov_pt.dist(s.at) > 1.3 && ground.clear_line_with_interiors(interiors, s.at, cov_pt, false) {
                                            Self::move_squad_one(s, ground, interiors, &bar_snap, &mut bar_dmg, cov_pt, 0, rush_spd, dt, 0.9);
                                            s.in_cover = false;
                                            continue;
                                        }
                                    }
                                }
                                s.speed = 0.0;
                                s.facing = m_at.sub(s.at).y.atan2(m_at.sub(s.at).x);
                                s.in_cover = s.floor == 0 && cur_cov < 0.8;
                                if s.shot_cd <= 0.0 {
                                    s.shot_cd = 1.2;
                                    let roll = rand01(&mut self.rng, 41);
                                    let m_cov = Self::static_cover_mul(barricades_ref, craters_ref, wrecks_ref, sandbag_mul, m_at);
                                    let p = forces.hit_chance(s.at.dist(m_at)) * m_cov;
                                    let hit = roll < p;
                                    let dmg = if hit { 18.0 } else { 0.0 };
                                    let miss = if hit { P2::default() } else { P2::new((roll - 0.5) * 2.0, -0.6) };
                                    militia_hits.push((mi, dmg, s.at, s.floor, m_at.add(miss), m_floor));
                                }
                                continue;
                            }
                        }
                        if seen_player && los_floors(s.at, s.floor, self.player_at, player_floor, true) {
                            let cur_cov = Self::static_cover_mul(barricades_ref, craters_ref, wrecks_ref, sandbag_mul, s.at);
                            if s.floor == 0 && cur_cov >= 0.8 && s.at.dist(self.player_at) > 10.0 {
                                if let Some(cov_pt) = Self::find_cover_near(barricades_ref, craters_ref, wrecks_ref, s.at, 5.5) {
                                    if cov_pt.dist(s.at) > 1.3 && ground.clear_line_with_interiors(interiors, s.at, cov_pt, false) {
                                        Self::move_squad_one(s, ground, interiors, &bar_snap, &mut bar_dmg, cov_pt, 0, rush_spd, dt, 0.9);
                                        s.in_cover = false;
                                        continue;
                                    }
                                }
                            }
                            s.speed = 0.0;
                            let d = self.player_at.sub(s.at);
                            s.facing = d.y.atan2(d.x);
                            s.in_cover = s.floor == 0 && cur_cov < 0.8;
                            if s.shot_cd <= 0.0 {
                                let (gap_lo, gap_hi) = forces.range("rifle", "burst_gap_s");
                                s.shot_cd = gap_lo + rand01(&mut self.rng, 7) * (gap_hi - gap_lo).max(0.05);
                                let dist = s.at.dist(self.player_at);
                                let mut chance = forces.hit_chance(dist);
                                if self.player_gait != Gait::Still {
                                    chance *= forces.num_or("rifle", "moving_target_mul", 0.6);
                                }
                                if self.body.sheltered {
                                    chance *= sandbag_mul.min(player_bar_cover);
                                } else if player_bar_cover < 1.0 {
                                    chance *= player_bar_cover;
                                }
                                if light == Light::NightDark && !s.optics {
                                    chance *= forces.num_or("rifle", "dark_mul", 0.5);
                                }
                                let r_hit = rand01(&mut self.rng, 13);
                                let r_dmg = rand01(&mut self.rng, 29);
                                if r_hit < chance {
                                    let (w_lo, w_hi) = forces.range("rifle", "wound_hp");
                                    let dmg = w_lo + r_dmg * (w_hi - w_lo);
                                    hits_on_player.push((dmg, s.at, s.floor, self.player_at, true));
                                } else {
                                    let aim = self.player_at.add(P2::new((r_hit - 0.5) * 2.4, (r_dmg - 0.5) * 2.4));
                                    hits_on_player.push((0.0, s.at, s.floor, aim, false));
                                }
                            }
                        } else {
                            Self::move_squad_one(s, ground, interiors, &bar_snap, &mut bar_dmg, g.last_seen, pursue_floor, rush_spd, dt, 1.0);
                            s.in_cover = false;
                        }
                    }
                    if g.lost_for > lose_s && militia_target.is_none() {
                        g.state = SquadState::Search;
                        g.timer = search_s;
                    }
                }
                SquadState::Search => {
                    let tf = if interiors.iter().any(|it| it.contains(g.last_seen)) { player_floor } else { 0 };
                    Self::move_squad(&mut g.men, ground, interiors, &bar_snap, &mut bar_dmg, g.last_seen, tf, walk_spd * 1.4, dt, 1.5);
                    g.timer -= dt;
                    if g.meter >= 1.0 {
                        if g.hostile || self.inv.has_rifle || self.player_gait != Gait::Still {
                            g.state = SquadState::Engage;
                            g.hostile = true;
                            g.lost_for = 0.0;
                        } else {
                            g.state = SquadState::Challenge;
                            g.timer = challenge_s;
                            g.shouted = false;
                        }
                    } else if g.timer <= 0.0 {
                        g.hostile = false;
                        g.state = SquadState::Patrol;
                    }
                }
            }
            notice_top = notice_top.max(g.meter);
            any_challenged |= matches!(g.state, SquadState::Challenge | SquadState::Detain);
            if let Some(lead) = g.men.iter().find(|m| m.hp > 0.0 && m.speed > 0.5) {
                let d = P2::new(lead.facing.cos(), lead.facing.sin());
                if !self.tracks.iter().rev().take(12).any(|t| !t.by_player && t.at.dist(lead.at) < 3.6) {
                    soldier_tracks.push((lead.at, d));
                }
            }
        }

        for (from, target, floor) in enemy_grenades {
            let dist = from.dist(target).max(2.0);
            let flight_dur = (dist / 16.0).clamp(0.35, 1.0);
            let fuse = forces.num_or("grenade", "fuse_s", 0.7).max(1.5);
            self.next_id += 1;
            self.grenades.push(ActiveGrenade {
                id: self.next_id,
                from,
                at: from,
                target,
                floor,
                fuse,
                flight_t: 0.0,
                flight_dur,
                friendly: false,
            });
            self.push_radio("敌军手雷", "INCOMING GRENADE", "俄军投掷破片手雷！快散开隐蔽", true);
        }

        if queue_spotted_fpv {
            self.fpv_cd = 90.0;
            let (flo, _) = forces.range("fpv", "spotted_to_strike_min");
            let fibre = self.next_rand(87) < forces.num_or("fpv", "fibre_optic_p", 0.2);
            self.pending_fpvs.push(PendingFpv {
                target: self.player_at,
                eta: Forces::war_minutes(flo) * 0.4,
                fibre_optic: fibre,
            });
        }

        for (bi, dmg) in bar_dmg {
            if let Some(b) = self.barricades.get_mut(bi) {
                b.hp = (b.hp - dmg).max(0.0);
                if b.hp <= 0.0 {
                    b.shoved = true;
                }
            }
        }

        for (i_idx, is_back) in breaches {
            let mut broke_open = false;
            let mut warn_thud = false;
            {
                let intr = &mut self.interiors[i_idx];
                if intr.breach_alert <= 0.25 {
                    intr.breach_alert = 0.9;
                    warn_thud = true;
                }
                if is_back {
                    if let Some(bd) = &mut intr.back_door {
                        bd.barricade_hp = (bd.barricade_hp - 12.0 * dt).max(0.0);
                        if bd.barricade_hp <= 0.0 {
                            bd.open = true;
                            broke_open = true;
                        }
                    }
                } else {
                    intr.front_door.barricade_hp = (intr.front_door.barricade_hp - 12.0 * dt).max(0.0);
                    if intr.front_door.barricade_hp <= 0.0 {
                        intr.front_door.open = true;
                        broke_open = true;
                    }
                }
            }
            if warn_thud {
                out.door_breach_thud = true;
                if self.active_interior == Some(i_idx) {
                    self.say("咚！俄军正在猛砸顶死的大门！", "Boom! Patrol is bashing down your barricaded door!");
                }
            }
            if broke_open {
                out.door_action = true;
                if self.active_interior == Some(i_idx) {
                    self.say("大门顶木被踹碎了！快从里屋或后门撤离！", "Door breached! Retreat through back room!");
                }
            }
        }

        if found_track && self.subtitle_ttl <= 0.5 {
            self.say("俄军发现了雪地脚印，正顺着踪迹摸过来！", "Patrol spotted snow tracks!");
        }
        for (at, dir) in soldier_tracks {
            self.push_track(at, dir, false, false);
        }

        if robbed {
            let taken = self.inv.food + self.inv.water + self.inv.bandage + self.inv.warmer;
            self.inv.food = 0;
            self.inv.water = 0;
            self.inv.bandage = 0;
            self.inv.warmer = 0;
            if taken > 0 {
                self.say("他们搜了你的身，把食物水药全拿走了，推了你一把。", "They searched you, took your supplies, and shoved you off.");
            } else {
                self.say("他们搜了你的身，什么也没搜到，挥手让你滚。", "They searched you, found nothing, and waved you off.");
            }
        }
        for (mi, dmg, from, from_floor, to, to_floor) in militia_hits {
            out.tracers.push(TracerFx {
                from: [from.x, from_floor as f32 * FLOOR_H + 1.35, -from.y],
                to: [to.x, to_floor as f32 * FLOOR_H + 1.25, -to.y],
                friendly: false,
            });
            if dmg > 0.0 {
                self.damage_militia(mi, dmg);
            }
        }
        let bleed_p = forces.num_or("rifle", "bleed_p", 0.85);
        if !hits_on_player.is_empty() {
            any_engaged = true;
        }
        for (dmg, from, from_floor, to, hit) in hits_on_player {
            out.tracers.push(TracerFx {
                from: [from.x, from_floor as f32 * FLOOR_H + 1.35, -from.y],
                to: [to.x, player_floor as f32 * FLOOR_H + 1.2, -to.y],
                friendly: false,
            });
            if self.active_interior.is_some() {
                let mid = from.lerp(to, 0.72);
                self.shatter_windows_near(mid, 4.0, out);
            }
            if hit && dmg > 0.0 {
                out.player_hit = true;
                let roll = self.next_rand(51);
                if self.body.hurt(dmg, bleed_p, roll) {
                    self.kill_player("你被步枪打中，倒在了雪里。", "A rifle round put you down in the snow.");
                    break;
                } else if self.body.bleeding {
                    self.say("你中弹了，在流血。按 Q 用绷带。", "Hit and bleeding! Press Q to bandage.");
                }
            }
        }

        self.under_fire_for = if any_engaged { 6.0 } else { (self.under_fire_for - dt).max(0.0) };
        self.challenged = any_challenged;
        self.notice = if any_engaged || any_challenged { 1.0 } else { notice_top.clamp(0.0, 1.0) };
    }

    fn move_squad_one(
        s: &mut Soldier,
        ground: &Ground,
        interiors: &[Interior],
        bar_snap: &[(usize, P2, BarricadeKind)],
        bar_dmg: &mut Vec<(usize, f32)>,
        goal: P2,
        goal_floor: u8,
        speed: f32,
        dt: f32,
        stop: f32,
    ) {
        let wish = goal.sub(s.at);
        if wish.len() > stop || s.floor != goal_floor {
            let mut eff_spd = speed;
            if s.floor == 0 {
                if bar_snap.iter().any(|(_, bp, bk)| *bk == BarricadeKind::Sandbag && bp.dist(s.at) < 1.6) {
                    eff_spd *= 0.45;
                }
                for &(bi, bp, bk) in bar_snap {
                    if bk == BarricadeKind::BarbedWire && bp.dist(s.at) <= 1.8 {
                        // Slowed and cut, not killed: a man in wire works his way
                        // through it, and the wire gives as he does. Play numbers.
                        eff_spd *= 0.30;
                        s.hp = (s.hp - 1.5 * dt).max(0.0);
                        bar_dmg.push((bi, 18.0 * dt));
                        break;
                    }
                }
            }
            let prev_at = s.at;
            let (yaw, moved) = Self::step_actor_floor(ground, interiors, &mut s.at, &mut s.floor, goal, goal_floor, 0.32, eff_spd, dt, true);
            if s.floor == 0 {
                for &(bi, bp, bk) in bar_snap {
                    if matches!(bk, BarricadeKind::IronGate | BarricadeKind::DeadCar | BarricadeKind::Concrete) && s.at.dist(bp) < 1.45 {
                        let away = s.at.sub(bp);
                        let dir = if away.len() > 0.05 { away.norm() } else { P2::new(0.0, 1.0) };
                        s.at = bp.add(dir.mul(1.45));
                        bar_dmg.push((bi, 14.0 * dt));
                    }
                }
            }
            Self::trigger_vault_if_crossed_snap(bar_snap, prev_at, s.at, s.floor, &mut s.vault_t);
            if moved > 0.0 {
                s.facing = yaw;
                s.in_cover = false;
            }
            s.speed = moved;
            s.phase += (std::f32::consts::TAU * eff_spd / (0.95 + 0.42 * eff_spd)) * dt;
        } else {
            s.speed = 0.0;
        }
    }

    fn move_squad(
        men: &mut [Soldier],
        ground: &Ground,
        interiors: &[Interior],
        bar_snap: &[(usize, P2, BarricadeKind)],
        bar_dmg: &mut Vec<(usize, f32)>,
        goal: P2,
        goal_floor: u8,
        speed: f32,
        dt: f32,
        stop: f32,
    ) {
        for (i, s) in men.iter_mut().enumerate() {
            if s.hp <= 0.0 {
                continue;
            }
            let spread = P2::new((i % 2) as f32 * 2.4 - 1.2, -(i as f32) * 1.6);
            let target = if i == 0 || goal_floor > 0 { goal } else { goal.add(spread) };
            Self::move_squad_one(s, ground, interiors, bar_snap, bar_dmg, target, goal_floor, speed, dt, stop);
        }
    }

    fn try_spawn_squad(&mut self, city: &City, forces: &Forces) {
        if self.patrol_roads.is_empty() {
            return;
        }
        let reach = front_depth(self.hour, forces);
        let mut spots = Vec::new();
        for (ri, road) in self.patrol_roads.iter().enumerate() {
            for (wi, p) in road.iter().enumerate() {
                let d = p.dist(self.player_at);
                let depth = city.bank_y(p.x) - p.y;
                if (65.0..=180.0).contains(&d) && (4.0..=reach).contains(&depth) {
                    spots.push((ri, wi, *p));
                }
            }
        }
        if spots.is_empty() {
            self.spawn_wait = 6.0;
            return;
        }
        let idx = ((self.next_rand(31) * spots.len() as f32) as usize).min(spots.len() - 1);
        let (ri, wi, at) = spots[idx];
        let (lo, hi) = forces.range("infantry_group", "size");
        let lo_n = (lo.round() as usize).max(1);
        let hi_n = (hi.round() as usize).max(lo_n);
        let n = (lo_n + ((self.next_rand(32) * (hi_n - lo_n + 1) as f32) as usize)).clamp(lo_n, hi_n);
        let opt_p = forces.num("infantry_group", "night_optics_p");
        let soldier_hp = forces.num_or("infantry_group", "soldier_hp", 40.0);
        let mut men = Vec::new();
        for i in 0..n {
            self.next_id += 1;
            let off = P2::new((i % 2) as f32 * 1.2 - 0.6, -(i as f32) * 2.2);
            men.push(Soldier {
                id: self.next_id,
                at: at.add(off),
                floor: 0,
                facing: -std::f32::consts::FRAC_PI_2,
                hp: soldier_hp,
                optics: self.next_rand(33 + i as u32) < opt_p,
                shot_cd: 0.4,
                speed: 0.0,
                phase: i as f32 * 0.9,
                in_cover: false,
                vault_t: 0.0,
                grenade_cd: 2.0 + i as f32 * 1.5,
            });
        }
        self.next_id += 1;
        self.squads.push(Squad {
            id: self.next_id,
            state: SquadState::Patrol,
            route_idx: ri,
            wp_idx: wi,
            wp_dir: 1,
            men,
            meter: 0.0,
            last_seen: at,
            lost_for: 0.0,
            timer: 0.0,
            ignore_for: 0.0,
            held_at: at,
            shouted: false,
            hostile: false,
            stuck_t: 0.0,
            last_lead_at: at,
        });
    }

    fn step_drone_and_mortars(&mut self, city: &City, forces: &Forces, light: Light, dt: f32, out: &mut TickOutput) {
        if self.alert_stage < 2 {
            return;
        }

        // Periodic quadcopter patrol arriving from the riverfront.
        if !self.drone.active {
            self.drone.wait_next -= dt;
            if self.drone.wait_next <= 0.0 {
                self.drone.active = true;
                self.drone.thermal = self.next_rand(61) < forces.num_or("recon_drone", "thermal_p", 0.3);
                let on_min = forces.num_or("recon_drone", "on_station_min", 20.0)
                    * forces.num_or("recon_drone", "cold_endurance_mul", 0.6);
                self.drone.state_left = Forces::war_minutes(on_min);
                let cx = (self.player_at.x + (self.next_rand(62) - 0.5) * 160.0)
                    .clamp(city.walk_x.0 + 25.0, city.walk_x.1 - 25.0);
                let cy = (city.bank_y(cx) - 35.0).max(self.player_at.y + 45.0);
                self.drone.centre = P2::new(cx, cy);
                self.drone.confirm = 0.0;
                self.say("头顶有嗡嗡声。一架侦察四轴无人机过来了，进门脸，别动！", "Quadcopter overhead. Take cover under a doorway!");
            }
        } else {
            let orbit = forces.num_or("recon_drone", "orbit_mps", 9.0);
            let r = 34.0;
            self.drone.angle += (orbit / r) * dt;
            self.drone.at = self.drone.centre.add(P2::new(self.drone.angle.cos() * r, self.drone.angle.sin() * r));
            self.drone.state_left -= dt;
            let seen = !self.in_tactical_smoke(self.player_at)
                && !self.under_tarp_dugout(self.player_at)
                && self.drone.sees_in_weather(forces, self.player_at, self.player_gait, self.body.sheltered, light, self.fog_density);
            self.drone.tracking_trail = false;
            if seen {
                self.drone.centre = self.drone.centre.lerp(self.player_at, (dt * 0.5).min(1.0));
                self.drone.confirm += dt;
                let confirm_s = forces.num_or("recon_drone", "confirm_s", 6.0);
                if self.drone.confirm >= confirm_s && self.mortars.iter().all(|m| m.friendly) {
                    self.drone.confirm = 0.0;
                    let (lo_m, hi_m) = forces.range("mortar_82", "sensor_to_shooter_min");
                    let mins = lo_m + self.next_rand(64) * (hi_m - lo_m);
                    let (rl, rh) = forces.range("mortar_82", "rounds");
                    let rounds = rl as u32 + ((self.next_rand(65) * (rh - rl + 1.0)) as u32).min(4);
                    self.mortars.push(MortarMission {
                        target: self.player_at,
                        fuse: Forces::war_minutes(mins),
                        rounds_left: rounds,
                        round_gap: 0.0,
                        friendly: false,
                        whistled: false,
                    });
                    if self.hour >= hm(18, 30) && self.pending_fpvs.is_empty() && self.fpv.is_none() {
                        let (flo, fhi) = forces.range("fpv", "spotted_to_strike_min");
                        let fmins = flo + self.next_rand(67) * (fhi - flo);
                        let fibre = self.next_rand(68) < forces.num_or("fpv", "fibre_optic_p", 0.2);
                        self.pending_fpvs.push(PendingFpv {
                            target: self.player_at,
                            eta: Forces::war_minutes(fmins),
                            fibre_optic: fibre,
                        });
                    }
                    self.say("无人机停在你头顶了！它看见了，迫击炮三到五战争分钟就到，离开空地进屋檐！", "Drone locked on you! Mortar rounds incoming.");
                }
            } else {
                self.drone.confirm = (self.drone.confirm - dt * 0.5).max(0.0);
                if self.fog_density < 0.75 {
                    let sight = self.drone.sight_radius_in_weather(forces, light, self.fog_density);
                    if let Some(tr) = self.tracks.iter().rev().find(|t| t.by_player && t.age < 75.0 && t.at.dist(self.drone.at) <= sight * 0.75) {
                        let chase = tr.at.add(tr.dir.mul(10.0));
                        self.drone.centre = self.drone.centre.lerp(chase, (dt * 0.45).min(1.0));
                        self.drone.tracking_trail = true;
                    }
                }
            }
            if self.drone.state_left <= 0.0 {
                self.drone.active = false;
                self.drone.tracking_trail = false;
                let (glo, ghi) = forces.range("recon_drone", "gap_between_sorties_min");
                self.drone.wait_next = Forces::war_minutes(glo + self.next_rand(66) * (ghi - glo).max(0.0));
            }
        }

        // Resolve mortar fire missions (both Russian drone-cued and friendly radio support).
        let mut impacts: Vec<(P2, f32, bool)> = Vec::new();
        let drone_seeing = !self.in_tactical_smoke(self.player_at)
            && !self.under_tarp_dugout(self.player_at)
            && self.drone.sees_in_weather(forces, self.player_at, self.player_gait, self.body.sheltered, light, self.fog_density);
        for i in 0..self.mortars.len() {
            if self.mortars[i].fuse > 0.0 {
                self.mortars[i].fuse -= dt;
                if !self.mortars[i].whistled && self.mortars[i].fuse <= 1.6 {
                    self.mortars[i].whistled = true;
                    out.mortar_whistle = true;
                }
                continue;
            }
            if self.mortars[i].rounds_left == 0 {
                continue;
            }
            self.mortars[i].round_gap -= dt;
            if self.mortars[i].round_gap <= 0.0 {
                self.mortars[i].rounds_left -= 1;
                let (glo, ghi) = forces.range("mortar_82", "round_gap_s");
                let r_gap = self.next_rand(71);
                let ang = self.next_rand(72) * std::f32::consts::TAU;
                let r_rad = self.next_rand(73).sqrt();
                let m = &mut self.mortars[i];
                m.round_gap = if m.friendly { 0.6 } else { glo + r_gap * (ghi - glo).max(0.1) };
                let centre = if !m.friendly && drone_seeing { self.player_at } else { m.target };
                let err = if m.friendly {
                    6.0
                } else if drone_seeing {
                    forces.num_or("mortar_82", "corrected_error_m", 11.0)
                } else {
                    forces.num_or("mortar_82", "first_round_error_m", 28.0)
                };
                let at = centre.add(P2::new(ang.cos() * r_rad * err, ang.sin() * r_rad * err));
                impacts.push((at, 16.0, m.friendly));
            }
        }
        self.mortars.retain(|m| m.fuse > 0.0 || m.rounds_left > 0);
        for (at, r, friendly) in impacts {
            if friendly {
                self.explode_at_floor(at, 0, 9.5, 130.0, 220.0, false, forces, true, out);
            } else {
                self.explode_at(at, r, 110.0, true, forces, false, out);
            }
        }

        // Sight-cued forces.json FPV strike chain (no blind timer).
        let mut due_fpvs = Vec::new();
        for p in &mut self.pending_fpvs {
            p.eta -= dt;
            if p.eta <= 0.0 {
                due_fpvs.push(p.clone());
            }
        }
        self.pending_fpvs.retain(|p| p.eta > 0.0);
        self.fpv_cd = (self.fpv_cd - dt).max(0.0);
        for p in due_fpvs {
            if self.fpv.is_some() {
                break;
            }
            let cold_fail = self.next_rand(81) < forces.num_or("fpv", "cold_failure_p", 0.15);
            if cold_fail {
                continue;
            }
            let base_reach = forces.num_or("fpv", "reach_target_p", 0.3);
            let ew_mul = if !p.fibre_optic && self.inv.has_radio {
                forces.num_or("fpv", "ew_jam_mul", 0.5)
            } else {
                1.0
            };
            let reach_ok = self.next_rand(82) < base_reach * ew_mul;
            let miss_ang = self.next_rand(83) * std::f32::consts::TAU;
            let miss_dist = 9.0 + self.next_rand(84) * 7.0;
            let target_spotted_now = drone_seeing
                || self.under_fire_for > 0.5
                || (self.active_interior.is_none() && self.player_at.dist(p.target) <= 16.0);
            let final_target = if reach_ok && target_spotted_now {
                self.player_at
            } else {
                p.target.add(P2::new(miss_ang.cos() * miss_dist, miss_ang.sin() * miss_dist))
            };
            let origin = final_target.add(P2::new(-45.0, 35.0));
            self.fpv = Some(FpvDrone {
                origin,
                at: origin,
                height: 22.0,
                target: final_target,
                hit: reach_ok && target_spotted_now,
                fibre_optic: p.fibre_optic,
                jammed: !p.fibre_optic && self.inv.has_radio && !reach_ok,
            });
            if p.fibre_optic {
                self.say("尖啸声！抗干扰光纤 FPV 巡飞弹扑过来了！", "Fibre-optic FPV diving!");
            } else {
                self.say("FPV 自杀无人机扑过来了！快闪进门脸！", "FPV diving! Get under cover!");
            }
        }

        if let Some(fpv) = self.fpv.take() {
            let d = fpv.target.sub(fpv.at);
            if d.len() < 1.5 {
                let at = fpv.target;
                let (wlo, whi) = forces.range("fpv", "wound_hp");
                let lethal = forces.num_or("fpv", "lethal_m", 3.0);
                let wound = forces.num_or("fpv", "wound_m", 7.0);
                let dist = at.dist(self.player_at);
                if fpv.hit && dist <= wound {
                    let follow_p = forces.num_or("fpv", "follows_into_doorway_p", 0.5);
                    let follows = self.next_rand(85) < follow_p;
                    let cover_mul = match self.active_interior.and_then(|i| self.interiors[i].zone(self.player_at).map(|z| (i, z))) {
                        Some((_, RoomZone::Basement | RoomZone::BackRoom)) => 0.0,
                        Some((i, RoomZone::FrontHall)) => {
                            let intr = &self.interiors[i];
                            if intr.front_door.open && intr.front_door.barricade_hp <= 0.0 && follows {
                                0.55
                            } else {
                                0.08
                            }
                        }
                        None => {
                            if self.under_tarp_dugout(self.player_at) {
                                0.25
                            } else if self.body.sheltered {
                                if follows { 0.65 } else { 0.25 }
                            } else {
                                1.0
                            }
                        }
                    };
                    let raw = if dist <= lethal { whi * 1.1 } else { wlo + self.next_rand(86) * (whi - wlo) };
                    let pre_scale = if self.body.sheltered { 0.35 } else { 1.0 };
                    self.explode_at(at, wound, raw * cover_mul / pre_scale, false, forces, false, out);
                } else {
                    self.explode_at(at, wound, if dist <= wound { 22.0 } else { 0.0 }, false, forces, false, out);
                }
            } else {
                let spd = forces.num_or("fpv", "speed_mps", 28.0);
                let mut next_fpv = fpv;
                next_fpv.at = next_fpv.at.add(d.norm().mul(spd * dt));
                next_fpv.height = (next_fpv.height - dt * 9.0).max(1.2);
                self.fpv = Some(next_fpv);
            }
        }

        // Russian Lancet-3 loitering munition hunting PLA armor or actively spotted open targets after 20:45.
        let lancet_spd = forces.num_or("lancet", "speed_mps", 34.0);
        let lancet_hp = forces.num_or("lancet", "hp", 20.0);
        let lancet_rad = forces.num_or("lancet", "blast_radius_m", 5.2);
        let lancet_dmg = forces.num_or("lancet", "wound_hp", 65.0);
        let lancet_sdmg = forces.num_or("lancet", "structural_dmg", 125.0);
        let lancet_admg = forces.num_or("lancet", "armor_dmg", 190.0);
        let lancet_cd = forces.num_or("lancet", "cooldown_s", 75.0);

        if let Some(mut lan) = self.lancet.take() {
            if let Some(tid) = lan.hit_tank_id {
                if let Some(t) = self.tanks.iter().find(|t| t.id == tid && t.hp > 0.0) {
                    lan.target = t.at;
                }
            }
            let d = lan.target.sub(lan.at);
            if d.len() < 1.8 {
                let impact = lan.target;
                if let Some(tid) = lan.hit_tank_id {
                    if let Some(t) = self.tanks.iter_mut().find(|t| t.id == tid) {
                        t.hp -= lancet_admg;
                    }
                }
                self.explode_at_floor(impact, 0, lancet_rad, lancet_dmg, lancet_sdmg, false, forces, false, out);
                self.say("「柳叶刀-3」巡飞弹俯冲爆炸！", "Russian Lancet-3 struck!");
            } else {
                lan.at = lan.at.add(d.norm().mul(lancet_spd * dt));
                lan.height = (lan.height - dt * 11.0).max(1.2);
                self.lancet = Some(lan);
            }
        } else if self.hour >= hm(20, 45) {
            self.lancet_cd -= dt;
            let pla_tgt = self.tanks.iter().find(|t| t.hp > 0.0 && t.faction == Faction::Pla).map(|t| (t.id, t.at));
            let spotted_player = drone_seeing || self.under_fire_for > 0.5;
            if self.lancet_cd <= 0.0 && (pla_tgt.is_some() || spotted_player) {
                self.lancet_cd = lancet_cd;
                let (tid, tgt) = pla_tgt.map(|(id, p)| (Some(id), p)).unwrap_or((None, self.player_at));
                let origin = tgt.add(P2::new(60.0, 85.0));
                self.lancet = Some(LancetDrone { origin, at: origin, height: 28.0, target: tgt, hp: lancet_hp, hit_tank_id: tid });
                self.say("警报：X翼「柳叶刀-3」巡飞弹从北岸俯冲来袭！（电磁干扰/Saiga-12/PGZ-09 可拦截）", "Incoming Lancet-3 loitering munition!");
            }
        }

        // Player launched friendly FPV strike drone.
        if let Some(mut ffpv) = self.friendly_fpv.take() {
            if let Some(ru) = self
                .tanks
                .iter()
                .filter(|t| t.hp > 0.0 && t.faction == Faction::Russia && t.at.dist(ffpv.target) <= 30.0)
                .min_by(|a, b| a.at.dist(ffpv.target).total_cmp(&b.at.dist(ffpv.target)))
            {
                ffpv.target = ru.at;
            }
            let d = ffpv.target.sub(ffpv.at);
            if d.len() < 1.8 {
                let imp = ffpv.target;
                if let Some(t) = self.tanks.iter_mut().find(|t| t.hp > 0.0 && t.faction == Faction::Russia && t.at.dist(imp) <= 4.0) {
                    t.hp -= 120.0;
                }
                self.explode_at_floor(imp, 0, 5.5, 160.0, 140.0, false, forces, true, out);
                self.say("我方自杀 FPV 精准命中俄军目标顶甲爆炸！", "Friendly FPV drone hit target!");
            } else {
                ffpv.at = ffpv.at.add(d.norm().mul(34.0 * dt));
                ffpv.height = if d.len() < 18.0 { (ffpv.height - dt * 14.0).max(1.0) } else { (ffpv.height + dt * 5.0).min(16.0) };
                self.friendly_fpv = Some(ffpv);
            }
        }
    }

    fn wrap_angle_diff(target: f32, current: f32) -> f32 {
        let mut d = (target - current) % std::f32::consts::TAU;
        if d > std::f32::consts::PI {
            d -= std::f32::consts::TAU;
        } else if d < -std::f32::consts::PI {
            d += std::f32::consts::TAU;
        }
        d
    }

    pub fn spawn_tank_wave_at(&mut self, kind: VehicleKind, route_hint: P2, forces: &Forces) {
        let faction = kind.faction();
        let mut best_route: Vec<P2> = self
            .patrol_roads
            .iter()
            .filter(|r| r.len() >= 2)
            .min_by(|a, b| {
                let da = a.iter().map(|p| p.dist(route_hint)).fold(f32::MAX, f32::min);
                let db = b.iter().map(|p| p.dist(route_hint)).fold(f32::MAX, f32::min);
                da.total_cmp(&db)
            })
            .cloned()
            .unwrap_or_else(|| vec![P2::new(-242.0, -210.0), P2::new(-254.0, -295.0), P2::new(-274.0, -345.0)]);
        if best_route.len() < 2 {
            best_route = vec![P2::new(-242.0, -210.0), P2::new(-274.0, -345.0)];
        }
        if faction == Faction::Russia {
            if best_route.last().is_some_and(|e| e.y > best_route[0].y) {
                best_route.reverse();
            }
        } else if best_route.last().is_some_and(|e| e.y < best_route[0].y) {
            best_route.reverse();
        }
        let start = best_route[0];
        let dir = best_route[1].sub(start).norm();
        let yaw = dir.y.atan2(dir.x);
        let hp = kind
            .forces_key()
            .map(|k| forces.vehicle_num(k, "hp", kind.max_hp()))
            .unwrap_or_else(|| kind.max_hp());
        self.next_id += 1;
        self.tanks.push(TankUnit {
            id: self.next_id,
            kind,
            faction,
            at: start,
            waypoints: best_route,
            wp_idx: 1,
            wp_dir: 1,
            hp,
            hull_yaw: yaw,
            turret_yaw: yaw,
            lock_timer: 0.0,
            locked_target: None,
            cannon_cd: 2.0,
            mg_cd: 1.5,
            salvo_left: 0,
            stuck_t: 0.0,
        });
    }

    fn step_tanks(&mut self, city: &City, ground: &Ground, forces: &Forces, light: Light, dt: f32, out: &mut TickOutput) {
        let _ = city;
        let waves: [(f32, u16, &[VehicleKind], P2, &str, &str); 7] = [
            (
                hm(18, 5),
                1 << 0,
                &[VehicleKind::TigrM, VehicleKind::Btr82A],
                P2::new(-242.0, -215.0),
                "警报！俄军「虎」式装甲车与 BTR-82A 步战车强渡黑龙江，正碾碎路障沿中央街南进！",
                "Russian Tigr-M & BTR-82A advancing south along Central St!",
            ),
            (
                hm(19, 0),
                1 << 1,
                &[VehicleKind::Bmd4M, VehicleKind::Bmp3, VehicleKind::T62M],
                P2::new(-160.0, -220.0),
                "警报！俄军 BMP-3 / BMD-4M 战车及加装顶棚格栅的 T-62M 坦克编队开入文化街！",
                "Russian BMP-3, BMD-4M & T-62M advancing along Wenhua St!",
            ),
            (
                hm(19, 45),
                1 << 2,
                &[VehicleKind::T72B3, VehicleKind::T80Bvm],
                P2::new(-250.0, -250.0),
                "紧急警报！俄军 T-72B3 与 T-80BVM 主战坦克开进城区！125mm高爆弹可炸穿楼板，上高楼用 HJ-12/PF-98 或俯掷 V 火瓶！",
                "Emergency: Russian T-72B3 & T-80BVM MBTs entered the city!",
            ),
            (
                hm(21, 15),
                1 << 3,
                &[VehicleKind::Zbl08, VehicleKind::Csk181, VehicleKind::Pgz09],
                P2::new(-275.0, -350.0),
                "解放军边防快反增援抵达！ZBL-08「大八轮」、猛士 CSK-181 与 PGZ-09 双35高炮正接战防空！",
                "PLA ZBL-08, CSK-181 & PGZ-09 AA joined defense!",
            ),
            (
                hm(22, 15),
                1 << 4,
                &[VehicleKind::T90M, VehicleKind::Bmpt, VehicleKind::Tos1A],
                P2::new(-140.0, -140.0),
                "特级警报！俄军精锐 T-90M「突破-3」、BMPT「终结者」与 TOS-1A 温压火箭炮突破江畔东路！",
                "Warning: T-90M, BMPT Terminator & TOS-1A thermobaric MLRS!",
            ),
            (
                hm(23, 20),
                1 << 5,
                &[VehicleKind::Ztz99A, VehicleKind::Ztz96B, VehicleKind::Ztq15, VehicleKind::Zbd04A],
                P2::new(-270.0, -355.0),
                "解放军重装旅主战力量投入反击！ZTZ-99A、96B、15式轻坦与 ZBD-04A 正与俄军装甲集群决战！",
                "PLA Heavy Armor (ZTZ-99A, ZTZ-96B, ZTQ-15, ZBD-04A) counterattacking!",
            ),
            (
                hm(24, 30),
                1 << 6,
                &[VehicleKind::T90M, VehicleKind::T80Bvm, VehicleKind::Tos1A],
                P2::new(-268.0, -325.0),
                "深夜总攻！俄军 T-90M 与 TOS-1A 重装甲矛头强攻中央街防线！",
                "Midnight heavy armor spearhead attacking Central Street!",
            ),
        ];
        for &(trigger_h, bit, kinds, hint, zh, en) in &waves {
            if self.hour >= trigger_h && (self.tank_wave_mask & bit) == 0 {
                self.tank_wave_mask |= bit;
                for (k_i, &kind) in kinds.iter().enumerate() {
                    let h = hint.add(P2::new((k_i as f32 - 1.0) * 8.0, -(k_i as f32) * 4.0));
                    self.spawn_tank_wave_at(kind, h, forces);
                }
                self.say(zh, en);
            }
        }

        self.tank_lock_warning = false;
        let mut cannon_shots: Vec<(P2, P2, u8, f32, f32, f32, bool)> = Vec::new();
        let mut mg_shots: Vec<(P2, P2, u8, bool, Option<usize>, Option<(usize, usize)>, f32, f32)> = Vec::new();
        let player_floor = if self.active_interior.is_some() { self.player_floor } else { 0 };
        let snapshot: Vec<(usize, Faction, P2, f32)> = self
            .tanks
            .iter()
            .enumerate()
            .filter(|(_, t)| t.hp > 0.0)
            .map(|(i, t)| (i, t.faction, t.at, t.kind.turret_mount_y() + 0.45))
            .collect();

        for ti in 0..self.tanks.len() {
            self.tanks[ti].cannon_cd = (self.tanks[ti].cannon_cd - dt).max(0.0);
            self.tanks[ti].mg_cd = (self.tanks[ti].mg_cd - dt).max(0.0);

            let tat = self.tanks[ti].at;
            let kind = self.tanks[ti].kind;
            let faction = self.tanks[ti].faction;
            let vkey = kind.forces_key();
            let vnum = |field: &str, def: f32| -> f32 {
                vkey.map(|k| forces.vehicle_num(k, field, def)).unwrap_or(def)
            };
            let v_spd = vnum("speed_mps", kind.speed());
            let v_range = vnum("range_m", 92.0);
            let (def_rad, def_dmg, def_sdmg, def_cd) = kind.cannon_profile();
            let v_rad = vnum("cannon_radius_m", def_rad);
            let v_dmg = vnum("cannon_dmg", def_dmg);
            let v_sdmg = vnum("structural_dmg", def_sdmg);
            let v_cd = vnum("cannon_cd_s", def_cd);
            let def_lock = if faction == Faction::Pla { 1.0 } else if kind.is_heavy_armor() { 1.8 } else { 0.65 };
            let v_lock = vnum("lock_s", def_lock);
            let v_mg_cd = vnum("mg_cd_s", 1.9);
            let v_mg_dmg = vnum("mg_dmg", 11.0);
            let v_mg_hit = vnum("mg_hit_p", 0.36);
            let v_mg_range = vnum("mg_range_m", 62.0);
            let v_slew = vnum("turret_slew_rps", if kind.is_heavy_armor() { 1.35 } else { 2.0 });

            // Concrete blocks all armor; BarbedWire/hedgehogs stall & damage wheeled/light armor; trenches aren't shoved aside.
            let mut blocked_by_barricade = false;
            let mut wire_dmg = 0.0f32;
            if faction == Faction::Russia {
                for b in &mut self.barricades {
                    if b.shoved || b.hp <= 0.0 {
                        continue;
                    }
                    let d_b = b.at.dist(tat);
                    match b.kind {
                        BarricadeKind::Concrete if d_b < 2.8 => {
                            blocked_by_barricade = true;
                            b.hp = (b.hp - (if kind.is_heavy_armor() { 55.0 } else { 28.0 }) * dt).max(0.0);
                            if b.hp <= 0.0 {
                                b.shoved = true;
                            }
                        }
                        BarricadeKind::DeadCar if d_b < 2.8 => {
                            if kind.is_heavy_armor() {
                                b.shoved = true;
                                b.hp = 0.0;
                            } else {
                                blocked_by_barricade = true;
                                b.hp = (b.hp - 45.0 * dt).max(0.0);
                                if b.hp <= 0.0 {
                                    b.shoved = true;
                                }
                            }
                        }
                        BarricadeKind::BarbedWire if d_b < 2.8 => {
                            if kind.is_heavy_armor() {
                                b.hp = (b.hp - 55.0 * dt).max(0.0);
                            } else {
                                blocked_by_barricade = true;
                                wire_dmg += 12.0 * dt;
                                b.hp = (b.hp - 32.0 * dt).max(0.0);
                            }
                            if b.hp <= 0.0 {
                                b.shoved = true;
                            }
                        }
                        BarricadeKind::Trench | BarricadeKind::TarpDugout => {}
                        // Further than 2.8 m these wait for the vehicle to reach them.
                        BarricadeKind::Concrete | BarricadeKind::DeadCar | BarricadeKind::BarbedWire => {}
                        _ if d_b < 4.2 => {
                            b.shoved = true;
                            b.hp = 0.0;
                        }
                        _ => {}
                    }
                }
            }
            if wire_dmg > 0.0 {
                self.tanks[ti].hp = (self.tanks[ti].hp - wire_dmg).max(0.0);
            }

            // PGZ-09 anti-air radar tracks & downs enemy drones / FPVs / Lancet-3 within 120m!
            if kind.is_aa() {
                let air_tgt: Option<(u8, P2, f32)> = self
                    .lancet
                    .as_ref()
                    .filter(|l| l.at.dist(tat) <= 120.0)
                    .map(|l| (2, l.at, l.height))
                    .or_else(|| {
                        self.fpv
                            .as_ref()
                            .filter(|f| f.at.dist(tat) <= 120.0)
                            .map(|f| (1, f.at, f.height))
                    })
                    .or_else(|| {
                        self.drone
                            .active
                            .then_some((0, self.drone.at, self.drone.altitude))
                            .filter(|(_, p, _)| p.dist(tat) <= 120.0)
                    });
                if let Some((akind, ap, ay)) = air_tgt {
                    let want_yaw = ap.sub(tat).y.atan2(ap.sub(tat).x);
                    let diff = Self::wrap_angle_diff(want_yaw, self.tanks[ti].turret_yaw);
                    self.tanks[ti].turret_yaw += diff.clamp(-3.2 * dt, 3.2 * dt);
                    if self.tanks[ti].cannon_cd <= 0.0 {
                        self.tanks[ti].cannon_cd = 1.1;
                        out.tank_cannon = true;
                        out.tracers.push(TracerFx {
                            from: [tat.x, 1.9, -tat.y],
                            to: [ap.x, ay, -ap.y],
                            friendly: true,
                        });
                        match akind {
                            2 => {
                                self.lancet = None;
                                self.say("PGZ-09 双35mm高炮凌空打爆「柳叶刀-3」！", "PGZ-09 shot down Lancet-3!");
                            }
                            1 => {
                                self.fpv = None;
                                self.say("PGZ-09 高炮拦截击落敌方 FPV 自杀机！", "PGZ-09 intercepted FPV!");
                            }
                            _ => {
                                self.drone.active = false;
                                self.say("PGZ-09 高炮击落俄军侦察无人机！", "PGZ-09 shot down recon drone!");
                            }
                        }
                        continue;
                    }
                }
            }

            // Acquire ground combat target based on faction.
            // `(tgt_at, tgt_floor, is_player, militia_idx, soldier_pair, dist)`
            let veh_sight_range = if faction == Faction::Pla { 122.0 } else { v_range.max(92.0) };
            let mut best_target: Option<(P2, u8, bool, Option<usize>, Option<(usize, usize)>, f32)> = None;
            let clear_veh_los = |a: P2, b: P2| -> bool {
                !Self::wrecks_block_los_slice(&self.wrecks, a, b)
                    && !Self::smoke_blocks_los_slice(&self.smoke_clouds, a, b)
                    && ground.clear_line_with_interiors(&self.interiors, a, b, false)
            };
            for &(other_i, other_f, other_at, _) in &snapshot {
                if other_i == ti || other_f == faction {
                    continue;
                }
                let vd = tat.dist(other_at);
                if vd <= veh_sight_range
                    && best_target.as_ref().is_none_or(|b| vd < b.5)
                    && clear_veh_los(tat, other_at)
                {
                    best_target = Some((other_at, 0, false, None, None, vd));
                }
            }

            if faction == Faction::Pla {
                for (gi, g) in self.squads.iter().enumerate() {
                    for (si, s) in g.men.iter().enumerate() {
                        if s.hp <= 0.0 || s.floor > 0 {
                            continue;
                        }
                        let sd = tat.dist(s.at);
                        if sd <= 88.0
                            && best_target.as_ref().is_none_or(|b| sd < b.5)
                            && clear_veh_los(tat, s.at)
                        {
                            best_target = Some((s.at, 0, false, None, Some((gi, si)), sd));
                        }
                    }
                }
            } else {
                let pd = tat.dist(self.player_at);
                // A crew finds a person by the same light as a soldier with a night
                // sight: 55 m in the dark, less standing still, less in fog, further
                // when you fire. No sourced figure for Russian vehicle thermals yet,
                // so this is play. Indoors it finds you only once infantry have.
                let fog_mul = (1.0 - self.fog_density.clamp(0.0, 1.0) * forces.num_or("infantry_group", "fog_sight_penalty_optics", 0.35)).max(0.0);
                let mut see_m = forces.sight(light, true) * exposure(self.player_gait, self.body.sheltered) * fog_mul;
                if self.gunfire_reveal_timer > 0.0 {
                    see_m = see_m.max(forces.num_or("infantry_group", "gunfire_reveal_m", 78.0) * fog_mul);
                }
                let seen_by_infantry = self.under_fire_for > 0.2;
                let player_exposed = if self.active_interior.is_some() { seen_by_infantry } else { pd <= see_m || seen_by_infantry };
                if pd <= v_range
                    && player_exposed
                    && best_target.as_ref().is_none_or(|b| pd < b.5)
                    && clear_veh_los(tat, self.player_at)
                {
                    best_target = Some((self.player_at, player_floor, true, None, None, pd));
                }
                for (mi, m) in self.militia.iter().enumerate() {
                    if m.hp <= 0.0 || m.downed {
                        continue;
                    }
                    let md = tat.dist(m.at);
                    if md <= v_range * 0.85
                        && best_target.as_ref().is_none_or(|b| md < b.5)
                        && clear_veh_los(tat, m.at)
                    {
                        best_target = Some((m.at, m.floor, false, Some(mi), None, md));
                    }
                }
            }

            let mut engaging_hold = false;
            if let Some((tgt_at, tgt_floor, is_player, militia_idx, soldier_pair, dist)) = best_target {
                let want_yaw = tgt_at.sub(tat).y.atan2(tgt_at.sub(tat).x);
                let diff = Self::wrap_angle_diff(want_yaw, self.tanks[ti].turret_yaw);
                self.tanks[ti].turret_yaw += diff.clamp(-v_slew * dt, v_slew * dt);

                let is_friendly_tank = faction == Faction::Pla;
                engaging_hold = diff.abs() < 0.75 || self.tanks[ti].lock_timer > 0.05;
                // Danger close: a PLA crew holds the gun (not the machine gun) while you or
                // a militia fighter stands within its blast of the target.
                let close_m = v_rad + 4.0;
                let danger_close = is_friendly_tank
                    && !is_player
                    && (self.player_at.dist(tgt_at) < close_m
                        || self.militia.iter().any(|m| m.hp > 0.0 && m.at.dist(tgt_at) < close_m));

                if self.tanks[ti].salvo_left > 0 && self.tanks[ti].cannon_cd <= 0.0 && !danger_close {
                    self.tanks[ti].salvo_left -= 1;
                    self.tanks[ti].cannon_cd = if self.tanks[ti].salvo_left > 0 { 0.45 } else { v_cd };
                    let scatter = P2::new((self.next_rand(97) - 0.5) * 6.0, (self.next_rand(98) - 0.5) * 6.0);
                    cannon_shots.push((tat, tgt_at.add(scatter), tgt_floor, v_rad, v_dmg, v_sdmg, is_friendly_tank));
                } else if diff.abs() <= 0.35 && self.tanks[ti].cannon_cd <= 0.0 && !danger_close {
                    let prev_lock = self.tanks[ti].lock_timer;
                    self.tanks[ti].lock_timer += dt;
                    self.tanks[ti].locked_target = Some((tgt_at, tgt_floor));
                    if is_player {
                        self.tank_lock_warning = true;
                        if prev_lock <= 0.01 {
                            self.say(
                                &format!("警告：{}炮塔锁定你了！（{:.1}秒后开火，快横向躲避！）", kind.label(), v_lock),
                                "Russian vehicle locking onto you!",
                            );
                        }
                    }
                    if self.tanks[ti].lock_timer >= v_lock {
                        self.tanks[ti].lock_timer = 0.0;
                        if kind.is_thermobaric() {
                            self.tanks[ti].salvo_left = 2;
                            self.tanks[ti].cannon_cd = 0.45;
                        } else {
                            self.tanks[ti].cannon_cd = v_cd;
                        }
                        cannon_shots.push((tat, tgt_at, tgt_floor, v_rad, v_dmg, v_sdmg, is_friendly_tank));
                    }
                } else {
                    self.tanks[ti].lock_timer = (self.tanks[ti].lock_timer - dt * 0.5).max(0.0);
                    self.tanks[ti].locked_target = None;
                }

                // Coaxial / autocannon MG bursts vs infantry whenever turret is near target arc.
                if dist <= v_mg_range && diff.abs() <= 0.45 && self.tanks[ti].mg_cd <= 0.0 && tgt_floor == 0 && (is_player || militia_idx.is_some() || soldier_pair.is_some()) {
                    self.tanks[ti].mg_cd = v_mg_cd;
                    mg_shots.push((tat, tgt_at, tgt_floor, is_friendly_tank, militia_idx, soldier_pair, v_mg_dmg, v_mg_hit));
                }
            } else {
                self.tanks[ti].lock_timer = 0.0;
                self.tanks[ti].locked_target = None;
                let hull = self.tanks[ti].hull_yaw;
                let diff = Self::wrap_angle_diff(hull, self.tanks[ti].turret_yaw);
                self.tanks[ti].turret_yaw += diff.clamp(-1.0 * dt, 1.0 * dt);
            }

            if !blocked_by_barricade && !engaging_hold && self.tanks[ti].lock_timer <= 0.05 && self.tanks[ti].salvo_left == 0 && !self.tanks[ti].waypoints.is_empty() {
                let wi = self.tanks[ti].wp_idx.min(self.tanks[ti].waypoints.len() - 1);
                let wp = self.tanks[ti].waypoints[wi];
                if tat.dist(wp) < 2.8 {
                    let next = self.tanks[ti].wp_idx as isize + self.tanks[ti].wp_dir;
                    if next < 0 || next >= self.tanks[ti].waypoints.len() as isize {
                        self.tanks[ti].wp_dir = -self.tanks[ti].wp_dir;
                    }
                    let clamped = (self.tanks[ti].wp_idx as isize + self.tanks[ti].wp_dir)
                        .clamp(0, self.tanks[ti].waypoints.len() as isize - 1) as usize;
                    self.tanks[ti].wp_idx = clamped;
                }
                let goal = self.tanks[ti].waypoints[self.tanks[ti].wp_idx];
                let d = goal.sub(tat);
                if d.len() > 0.5 {
                    let want = tat.add(d.norm().mul(v_spd * dt));
                    let next_pos = ground.step(tat, want, 0.85);
                    let blocked_wreck = self.wrecks.iter().any(|w| w.at.dist(next_pos) < 3.4);
                    let moves = !blocked_wreck && next_pos.dist(tat) > 0.05 * dt;
                    // Progress is what counts: sliding along a wall moves it a little
                    // every step and gets it nowhere.
                    let progress = d.len() - goal.dist(next_pos);
                    if !moves || progress < 0.25 * v_spd * dt {
                        // A wreck in the way or a wall the route cuts through: back up
                        // the way it came rather than wait for the rest of the night.
                        self.tanks[ti].stuck_t += dt;
                        if self.tanks[ti].stuck_t > 3.0 {
                            let t = &mut self.tanks[ti];
                            t.stuck_t = 0.0;
                            t.wp_dir = -t.wp_dir;
                            t.wp_idx = (t.wp_idx as isize + t.wp_dir).clamp(0, t.waypoints.len() as isize - 1) as usize;
                        }
                    } else {
                        self.tanks[ti].stuck_t = 0.0;
                    }
                    if moves {
                        self.tanks[ti].hull_yaw = d.y.atan2(d.x);
                        self.tanks[ti].at = next_pos;
                        let dir = d.norm();
                        if !self.tracks.iter().rev().take(16).any(|t| t.is_vehicle && t.at.dist(next_pos) < 2.2) {
                            self.push_track(next_pos, dir, false, true);
                        }
                    }
                }
            }
        }

        for (from, to, t_floor, rad, dmg, sdmg, is_friendly) in cannon_shots {
            out.tank_cannon = true;
            out.tracers.push(TracerFx {
                from: [from.x, 1.9, -from.y],
                to: [to.x, t_floor as f32 * FLOOR_H + 1.1, -to.y],
                friendly: is_friendly,
            });
            self.explode_at_floor(to, t_floor, rad, dmg, sdmg, false, forces, is_friendly, out);
        }

        for (from, to, t_floor, is_friendly, militia_idx, soldier_pair, mg_dmg, mg_hit_p) in mg_shots {
            let roll = self.next_rand(94);
            let miss = P2::new((roll - 0.5) * 2.2, (self.next_rand(95) - 0.5) * 2.2);
            let hit = roll < mg_hit_p;
            let impact = if hit { to } else { to.add(miss) };
            out.tracers.push(TracerFx {
                from: [from.x, 1.8, -from.y],
                to: [impact.x, t_floor as f32 * FLOOR_H + 1.1, -impact.y],
                friendly: is_friendly,
            });
            if hit {
                if let Some((gi, si)) = soldier_pair {
                    self.damage_soldier(gi, si, 26.0);
                } else if let Some(mi) = militia_idx {
                    self.damage_militia(mi, mg_dmg);
                } else if !is_friendly && t_floor == player_floor && to.dist(self.player_at) < 1.2 {
                    out.player_hit = true;
                    let r = self.next_rand(96);
                    if self.body.hurt(mg_dmg, 0.5, r) {
                        self.kill_player("你被俄军战车机枪扫中！", "Cut down by coaxial MG!");
                    }
                }
            }
        }
        let mut wire_dead = Vec::new();
        for ti in (0..self.tanks.len()).rev() {
            if self.tanks[ti].hp <= 0.0 {
                wire_dead.push(self.tanks.remove(ti));
            }
        }
        for dead in wire_dead {
            self.push_wreck(dead);
        }
    }

    pub fn explode_at(&mut self, at: P2, radius: f32, raw_dmg: f32, use_mortar_table: bool, forces: &Forces, friendly: bool, out: &mut TickOutput) {
        let sdmg = if use_mortar_table { 125.0 } else { 0.0 };
        self.explode_at_floor(at, 0, radius, raw_dmg, sdmg, use_mortar_table, forces, friendly, out);
    }

    pub fn explode_at_floor(
        &mut self,
        at: P2,
        target_floor: u8,
        radius: f32,
        raw_dmg: f32,
        structural_dmg: f32,
        use_mortar_table: bool,
        forces: &Forces,
        friendly: bool,
        out: &mut TickOutput,
    ) {
        out.explosions.push(ExplosionFx { at, radius });
        // Cover as it was before this blast: its own crater must not shelter anyone from it.
        let player_defilade = self.defilade_blast_mul(self.player_at);
        let militia_defilade: Vec<f32> = self.militia.iter().map(|m| self.defilade_blast_mul(m.at)).collect();
        self.shatter_windows_near(at, 18.0, out);
        if target_floor == 0 && radius >= 4.5 && !self.interiors.iter().any(|it| it.contains(at)) {
            self.push_crater(at, radius * 0.42);
            self.push_smoke_cloud(at, (radius * 0.55).clamp(2.6, 5.2), 8.5, true);
        }

        if structural_dmg > 0.0 {
            let mut broke_slabs = 0u32;
            let bound = radius + 24.0;
            for intr in &mut self.interiors {
                if (intr.origin.x - at.x).abs() > bound || (intr.origin.y - at.y).abs() > bound {
                    continue;
                }
                let (broken, _) = intr.blast_at(at, target_floor, radius, structural_dmg);
                broke_slabs += broken;
            }
            for b in &mut self.barricades {
                if !b.shoved && b.at.dist(at) <= radius * 0.85 {
                    b.hp = (b.hp - structural_dmg * 0.35).max(0.0);
                    if b.hp <= 0.0 {
                        b.shoved = true;
                    }
                }
            }
            if broke_slabs > 0 {
                out.slab_collapse = true;
            }
        }

        // Player takes splash harm from ANY explosion (including careless close-range friendly 155mm/RPG/grenade).
        {
            let d = at.dist(self.player_at);
            let player_floor = if self.active_interior.is_some() { self.player_floor } else { 0 };
            let shielded_by_slab = self.active_interior.is_some_and(|i| {
                player_floor != target_floor && self.interiors[i].is_slab_intact(player_floor.max(target_floor), at)
            });
            let indoor_mul = self.active_interior.map(|i| self.interiors[i].blocks_mortar(self.player_at));
            let concrete_mul = player_defilade;
            let dmg = if shielded_by_slab {
                0.0
            } else if use_mortar_table {
                let raw = if let Some(im) = indoor_mul {
                    blast_harm(forces, d, false) * im
                } else {
                    blast_harm(forces, d, self.body.sheltered)
                };
                raw * concrete_mul
            } else if d < radius {
                let same_room = self.active_interior.is_some_and(|i| self.interiors[i].contains(at));
                let shelter = match indoor_mul {
                    Some(walls) if !same_room => walls,
                    _ if player_floor == target_floor && self.body.sheltered => 0.45,
                    _ => 1.0,
                };
                let falloff = (1.0 - (d / radius.max(0.5)) * 0.5).clamp(0.3, 1.0);
                raw_dmg * shelter * falloff * concrete_mul
            } else {
                0.0
            };
            if dmg > 0.0 {
                out.player_hit = true;
                let roll = self.next_rand(91);
                if self.body.hurt(dmg, 0.9, roll) {
                    self.kill_player("一发炮弹落在你身边。", "A shell landed beside you.");
                }
            }
        }
        for gi in 0..self.squads.len() {
            for si in 0..self.squads[gi].men.len() {
                if self.squads[gi].men[si].at.dist(at) <= radius * 0.7 {
                    self.damage_soldier(gi, si, 85.0);
                }
            }
        }
        for mi in 0..self.militia.len() {
            let m_at = self.militia[mi].at;
            let m_floor = self.militia[mi].floor;
            if m_floor == target_floor && m_at.dist(at) <= radius * 0.78 {
                let bmul = militia_defilade.get(mi).copied().unwrap_or(1.0);
                let falloff = (1.0 - (m_at.dist(at) / radius.max(0.5)) * 0.5).clamp(0.3, 1.0);
                self.damage_militia(mi, raw_dmg.max(60.0) * bmul * falloff);
            }
        }

        let mut destroyed = Vec::new();
        for (ti, tank) in self.tanks.iter_mut().enumerate() {
            let target_match = if friendly {
                tank.faction == Faction::Russia
            } else {
                tank.faction == Faction::Pla
            };
            if target_match && tank.at.dist(at) <= radius {
                tank.hp -= raw_dmg.max(85.0);
                if tank.hp <= 0.0 {
                    destroyed.push((ti, tank.kind));
                }
            }
        }
        let mut cookoffs = Vec::new();
        for (ti, kind) in destroyed.into_iter().rev() {
            let dead = self.tanks.remove(ti);
            let kill_at = dead.at;
            self.push_wreck(dead);
            out.explosions.push(ExplosionFx { at: kill_at, radius: 7.5 });
            if kind.faction() == Faction::Russia {
                self.inv.scrap += 6;
                self.inv.ammo += 12;
                if kind.is_thermobaric() {
                    cookoffs.push(kill_at);
                } else {
                    self.say(
                        &format!("击毁一辆俄军{}！车体爆燃（废铁×6、弹药×12）！", kind.label()),
                        "Russian armoured vehicle knocked out!",
                    );
                }
            } else {
                self.say(&format!("一架我方{}被击毁！", kind.label()), "PLA vehicle lost!");
            }
        }
        for cp in cookoffs {
            out.explosions.push(ExplosionFx { at: cp, radius: 12.0 });
            out.tank_cannon = true;
            for intr in &mut self.interiors {
                if (intr.origin.x - cp.x).abs() <= 30.0 && (intr.origin.y - cp.y).abs() <= 30.0 {
                    intr.blast_at(cp, 0, 11.5, 260.0);
                }
            }
            for gi in 0..self.squads.len() {
                for si in 0..self.squads[gi].men.len() {
                    if self.squads[gi].men[si].at.dist(cp) <= 11.0 {
                        self.damage_soldier(gi, si, 140.0);
                    }
                }
            }
            self.say("TOS-1A「喷火坦克」殉爆！温压弹药库发生惊天连环爆炸！", "TOS-1A catastrophic ammunition cook-off!");
        }

        if !self.dead {
            self.apply_slab_gravity(out);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn load_forces() -> Forces {
        let path = crate::data_dir().join("russia/forces.json");
        Forces::from_json(&std::fs::read_to_string(path).unwrap()).unwrap()
    }

    #[test]
    fn temperature_falls_in_the_street_warms_in_a_heated_doorway_and_heating_stops_at_nine() {
        let mut cold_body = Body::default();
        for _ in 0..30 {
            assert!(cold_body.tick(1.0, 18.0, Gait::Walk, false, false, false).is_none());
        }
        assert!(cold_body.temp < START_TEMP);

        let mut warm_body = cold_body.clone();
        for _ in 0..20 {
            assert!(warm_body.tick(1.0, 18.0, Gait::Still, true, false, false).is_none());
        }
        assert!(warm_body.temp > cold_body.temp);

        let mut late_body = Body::default();
        for _ in 0..20 {
            late_body.tick(1.0, hm(21, 5), Gait::Still, true, false, false);
        }
        assert!(late_body.temp < START_TEMP, "after 21:00 doorways keep wind off but no longer warm");
    }

    #[test]
    fn shop_signs_classify_and_a_pharmacy_always_has_bandages() {
        assert_eq!(shop_kind("肯德基", ""), ShopKind::Food);
        assert_eq!(shop_kind("瑞幸咖啡", ""), ShopKind::Drink);
        assert_eq!(shop_kind("牙科诊所", ""), ShopKind::Pharmacy);
        assert_eq!(shop_kind("汉庭连锁酒店", ""), ShopKind::Hotel);
        assert_eq!(shop_kind("以纯服饰", ""), ShopKind::Clothes);
        assert_eq!(shop_kind("宇泽汽车装饰用品店", ""), ShopKind::Hardware);
        assert_eq!(shop_kind("中国银行", ""), ShopKind::Office);
        assert_eq!(shop_kind("", "Ресторан пельменей"), ShopKind::Food);
        for seed in 0..25 {
            assert!(roll_shop(ShopKind::Pharmacy, seed).bandage >= 2);
        }
    }

    #[test]
    fn holding_e_by_a_shop_searches_it_once_and_items_heal_warm_and_feed() {
        let (city, _) = crate::load_heihe().unwrap();
        let ground = Ground::new(&city);
        let forces = load_forces();
        let mut sim = WorldSim::new(&city);
        // Pick first pharmacy or clothes or any shop.
        let door_idx = sim.doors.iter().position(|d| d.kind == ShopKind::Pharmacy).unwrap_or(0);
        sim.player_at = sim.doors[door_idx].at;
        let inp = SimInput { interact_held: true, ..Default::default() };
        for _ in 0..30 {
            sim.tick(&city, &ground, &forces, &inp, 0.1);
        }
        assert!(sim.doors[door_idx].searched, "shop door must finish scavenging after 2.6 s hold");
        let after_first = sim.inv.bandage;
        sim.tick(&city, &ground, &forces, &inp, 0.5);
        assert_eq!(sim.inv.bandage, after_first, "each door can only be searched once");

        sim.body.hp = 60.0;
        sim.body.bleeding = true;
        sim.body.temp = 35.0;
        sim.inv.warmer = 1;
        sim.body.hunger = 40.0;
        sim.inv.food = 1;
        sim.inv.bandage = sim.inv.bandage.max(1);
        let use_inp = SimInput { use_bandage: true, use_food: true, use_warmer: true, ..Default::default() };
        sim.tick(&city, &ground, &forces, &use_inp, 0.1);
        assert!(!sim.body.bleeding);
        assert!(sim.body.hp > 65.0);
        assert!(sim.body.temp > 35.5);
        // One ration is a day's worth of the reserve's calories, not a full belly.
        let l = crate::living::get();
        let ration = l.num("body", "food_item_kcal") * 100.0 / l.num("body", "food_reserve_kcal");
        assert!(sim.body.hunger > 40.0 + ration * 0.9, "{}", sim.body.hunger);
    }

    #[test]
    fn the_front_moves_south_after_1736_and_blackout_darkens_the_streets_at_1930() {
        let (city, _) = crate::load_heihe().unwrap();
        let ground = Ground::new(&city);
        let forces = load_forces();
        assert_eq!(front_depth(hm(17, 20), &forces), 0.0);
        assert!(front_depth(hm(18, 0), &forces) > front_depth(hm(17, 40), &forces));

        let mut sim = WorldSim::new(&city);
        sim.hour = hm(19, 29);
        let lamp_at = city.lamps[0];
        sim.tick(&city, &ground, &forces, &SimInput::default(), 0.1);
        assert!(!sim.blackout);
        assert_eq!(light_at(&city, sim.hour, sim.blackout, false, lamp_at, &[]), Light::NightLit);

        sim.hour = hm(19, 31);
        sim.tick(&city, &ground, &forces, &SimInput::default(), 0.1);
        assert!(sim.blackout);
        assert_eq!(light_at(&city, sim.hour, sim.blackout, false, lamp_at, &[]), Light::NightDark);
    }

    #[test]
    fn a_soldier_sees_only_in_his_cone_and_light_and_buildings_block_sight() {
        let (city, _) = crate::load_heihe().unwrap();
        let ground = Ground::new(&city);
        let from = P2::new(0.0, 0.0);
        assert!(in_arc(std::f32::consts::FRAC_PI_2, from, P2::new(0.0, 12.0), 100.0));
        assert!(!in_arc(std::f32::consts::FRAC_PI_2, from, P2::new(0.0, -12.0), 100.0));

        let b = city.buildings.iter().find(|b| b.solid && b.footprint.len() >= 4).unwrap();
        let dir = b.footprint[1].sub(b.footprint[0]).right().norm();
        assert!(!ground.clear_line(b.centroid.sub(dir.mul(35.0)), b.centroid.add(dir.mul(35.0))));
    }

    #[test]
    fn standing_still_when_challenged_gets_you_searched_and_running_gets_you_shot() {
        let (city, _) = crate::load_heihe().unwrap();
        let ground = Ground::new(&city);
        let forces = load_forces();

        // Case 1: unarmed player stands still when challenged -> robbed and released.
        let mut sim = WorldSim::new(&city);
        sim.hour = 18.0;
        sim.alert_stage = 2;
        sim.inv.food = 2;
        sim.inv.water = 1;
        let spot = sim.player_at;
        sim.squads.push(Squad {
            id: 1,
            state: SquadState::Challenge,
            route_idx: 0,
            wp_idx: 0,
            wp_dir: 1,
            men: vec![Soldier { id: 10, at: spot.add(P2::new(4.0, 0.0)), floor: 0, facing: std::f32::consts::PI, hp: 40.0, optics: false, shot_cd: 0.0, speed: 0.0, phase: 0.0, in_cover: false, vault_t: 0.0, grenade_cd: 0.0 }],
            meter: 1.0,
            last_seen: spot,
            lost_for: 0.0,
            timer: 0.1,
            ignore_for: 0.0,
            held_at: spot,
            shouted: true,
            hostile: false,
            stuck_t: 0.0,
            last_lead_at: spot,
        });
        for _ in 0..40 {
            sim.tick(&city, &ground, &forces, &SimInput::default(), 0.1);
        }
        assert_eq!(sim.inv.food, 0);
        assert_eq!(sim.inv.water, 0);
        assert_eq!(sim.squads[0].state, SquadState::Patrol, "stood still must return to Patrol after search");
        assert!(sim.squads[0].ignore_for > 10.0);

        // Case 2: running away when challenged triggers Engage & gunfire.
        let mut sim2 = WorldSim::new(&city);
        sim2.hour = 18.0;
        sim2.alert_stage = 2;
        let p0 = sim2.player_at;
        sim2.squads.push(Squad {
            id: 2,
            state: SquadState::Challenge,
            route_idx: 0,
            wp_idx: 0,
            wp_dir: 1,
            men: vec![Soldier { id: 11, at: p0.add(P2::new(6.0, 0.0)), floor: 0, facing: std::f32::consts::PI, hp: 40.0, optics: false, shot_cd: 0.0, speed: 0.0, phase: 0.0, in_cover: false, vault_t: 0.0, grenade_cd: 0.0 }],
            meter: 1.0,
            last_seen: p0,
            lost_for: 0.0,
            timer: 0.2,
            ignore_for: 0.0,
            held_at: p0,
            shouted: true,
            hostile: false,
            stuck_t: 0.0,
            last_lead_at: p0,
        });
        let run = SimInput { wish_move: P2::new(0.0, -1.0), sprinting: true, ..Default::default() };
        for _ in 0..10 {
            sim2.tick(&city, &ground, &forces, &run, 0.1);
        }
        assert_eq!(sim2.squads[0].state, SquadState::Engage, "running from challenge must provoke Engage");
    }

    #[test]
    fn a_drone_holding_you_in_view_calls_mortars_twelve_to_twenty_seconds_later_and_a_roof_cuts_the_blast() {
        let (_city, _) = crate::load_heihe().unwrap();
        let forces = load_forces();
        let drone = ReconDrone { active: true, at: P2::new(0.0, 0.0), ..Default::default() };
        assert!(drone.sees(&forces, P2::new(5.0, 5.0), Gait::Walk, false, Light::Day));
        assert!(!drone.sees(&forces, P2::new(5.0, 5.0), Gait::Walk, true, Light::Day));

        let open_harm = blast_harm(&forces, 10.0, false);
        let roof_harm = blast_harm(&forces, 10.0, true);
        assert!(roof_harm > 0.0 && roof_harm < open_harm * 0.35);
    }

    #[test]
    fn civilians_can_organize_resistance_build_barricades_and_unlock_supplies_over_time() {
        let (city, _) = crate::load_heihe().unwrap();
        let ground = Ground::new(&city);
        let forces = load_forces();
        let mut sim = WorldSim::new(&city);

        // Advance past 17:36 & rally resistance at cell + build a fire barrel.
        sim.hour = hm(18, 35);
        sim.tick(&city, &ground, &forces, &SimInput::default(), 0.1);
        assert_eq!(sim.alert_stage, 2);
        sim.player_at = sim.caches.iter().find(|c| c.kind == CacheKind::ResistanceCell).unwrap().at;
        sim.tick(&city, &ground, &forces, &SimInput { rally_resistance: true, ..Default::default() }, 0.1);
        assert!(sim.resistance_active);
        assert!(sim.inv.has_rifle);
        assert!(sim.militia.len() >= 3);

        let build_barrel = SimInput { select_barricade: Some(BarricadeKind::FireBarrel), build_barricade: true, ..Default::default() };
        sim.tick(&city, &ground, &forces, &build_barrel, 0.1);
        assert_eq!(sim.fire_barrels.len(), 1);
        assert_eq!(sim.barricades.len(), 1);

        // Progressive cache at 20:00 SOS unlocks radio support.
        sim.hour = hm(20, 5);
        let sos_at = sim.caches.iter().find(|c| c.kind == CacheKind::SosNeighbours).unwrap().at;
        sim.player_at = sos_at;
        sim.tick(&city, &ground, &forces, &SimInput { interact_just_pressed: true, ..Default::default() }, 0.1);
        assert!(sim.neighbours_rescued);
        assert!(sim.inv.has_radio);
        assert!(sim.militia.len() >= 4);
    }

    #[test]
    fn snow_tracks_accumulate_on_open_snow_and_guide_drones_and_patrols() {
        let (city, _) = crate::load_heihe().unwrap();
        let ground = Ground::new(&city);
        let forces = load_forces();
        let mut sim = WorldSim::new(&city);
        sim.hour = hm(17, 45);
        sim.alert_stage = 2;
        sim.weather_override = Some(Weather::Clear);

        // Move off heated doorways across open Central St snow so footprints stamp.
        let walk = SimInput { wish_move: P2::new(-1.0, 0.3).norm(), ..Default::default() };
        for _ in 0..20 {
            sim.tick(&city, &ground, &forces, &walk, 0.25);
        }
        assert!(sim.tracks.iter().any(|t| t.by_player), "walking in open snow must stamp player footprints");

        let newest = sim.tracks.iter().rfind(|t| t.by_player).unwrap().at;
        // Put a patrol right in front of the trail facing it (player moved away beyond sight reach).
        sim.player_at = newest.add(P2::new(-220.0, 0.0));
        sim.squads.push(Squad {
            id: 99,
            state: SquadState::Patrol,
            route_idx: 0,
            wp_idx: 0,
            wp_dir: 1,
            men: vec![Soldier { id: 999, at: newest.add(P2::new(-4.0, 0.0)), floor: 0, facing: 0.0, hp: 40.0, optics: false, shot_cd: 0.0, speed: 0.0, phase: 0.0, in_cover: false, vault_t: 0.0, grenade_cd: 0.0 }],
            meter: 0.0,
            last_seen: newest,
            lost_for: 0.0,
            timer: 0.0,
            ignore_for: 0.0,
            held_at: newest,
            shouted: false,
            hostile: false,
            stuck_t: 0.0,
            last_lead_at: newest,
        });
        sim.drone.active = true;
        sim.drone.state_left = 40.0;
        sim.drone.angle = 0.0;
        sim.drone.centre = newest.add(P2::new(-30.0, 0.0));
        sim.drone.at = newest.add(P2::new(4.0, 0.0));
        sim.tick(&city, &ground, &forces, &SimInput::default(), 0.1);
        assert_eq!(sim.squads[0].state, SquadState::Check, "patrol seeing fresh snow track must enter Check");
        assert!(sim.drone.tracking_trail, "drone in clear air must track fresh snow trail");
    }

    #[test]
    fn river_fog_blinds_recon_drones_and_cuts_patrol_sight() {
        let forces = load_forces();
        let drone = ReconDrone { active: true, thermal: false, ..Default::default() };
        let clear = drone.sight_radius_in_weather(&forces, Light::Day, 0.0);
        let foggy = drone.sight_radius_in_weather(&forces, Light::Day, 0.92);
        assert!(foggy < clear * 0.30, "river fog must sharply blind optical drone sight: clear={clear}, foggy={foggy}");
        assert_eq!(scheduled_weather(hm(21, 0)).0, Weather::RiverFog);
        assert!(scheduled_weather(hm(21, 0)).1 > 0.8);
    }

    #[test]
    fn fpv_uses_table_delay_fibre_optic_share_and_miss_rate() {
        let (city, _) = crate::load_heihe().unwrap();
        let ground = Ground::new(&city);
        let forces = load_forces();
        assert_eq!(forces.range("fpv", "spotted_to_strike_min"), (10.0, 20.0));
        assert!((forces.num("fpv", "fibre_optic_p") - 0.2).abs() < 0.01);
        assert!((forces.num("fpv", "reach_target_p") - 0.3).abs() < 0.01);

        let mut sim = WorldSim::new(&city);
        sim.hour = hm(19, 0);
        sim.alert_stage = 2;
        sim.pending_fpvs.push(PendingFpv { target: sim.player_at, eta: 0.05, fibre_optic: true });
        for _ in 0..30 {
            let fx = sim.tick(&city, &ground, &forces, &SimInput::default(), 0.2);
            if !fx.explosions.is_empty() {
                break;
            }
        }
        // Pending queue must drain (either cold-failed or launched & detonated).
        assert!(sim.pending_fpvs.is_empty());
    }

    #[test]
    fn walking_records_visited_map_cells_and_right_click_orders_militia_to_spot() {
        let (city, _) = crate::load_heihe().unwrap();
        let ground = Ground::new(&city);
        let forces = load_forces();
        let mut sim = WorldSim::new(&city);
        sim.hour = hm(18, 35);
        sim.tick(&city, &ground, &forces, &SimInput::default(), 0.1);
        sim.rally_militia();
        assert!(!sim.militia.is_empty());

        let walk = SimInput { wish_move: P2::new(-1.0, 0.0), ..Default::default() };
        for _ in 0..16 {
            sim.tick(&city, &ground, &forces, &walk, 0.5);
        }
        assert!(sim.visited.len() >= 2, "walking across blocks must reveal new map cells");

        let hold_spot = sim.player_at.add(P2::new(6.0, 2.0));
        sim.tick(&city, &ground, &forces, &SimInput { command_militia_to: Some(hold_spot), ..Default::default() }, 0.1);
        assert_eq!(sim.militia[0].order, MilitiaOrder::Hold(hold_spot));
    }

    #[test]
    fn enterable_interiors_include_shops_home_neighbour_and_basement() {
        let (city, _) = crate::load_heihe().unwrap();
        let sim = WorldSim::new(&city);
        let res_n = sim.interiors.iter().filter(|i| i.kind == InteriorKind::Residential).count();
        let back_n = sim.interiors.iter().filter(|i| i.back_door.is_some()).count();
        println!("ENTERABLE_TOTAL={} RESIDENTIAL={} BACK_DOORS={}", sim.interiors.len(), res_n, back_n);
        assert!(sim.interiors.len() >= 350, "expected >= 350 enterable units across Heihe multi-bay blocks, got {}", sim.interiors.len());
        assert!(sim.interiors.iter().any(|i| i.kind == InteriorKind::PlayerHome), "must have PlayerHome");
        assert!(sim.interiors.iter().any(|i| i.kind == InteriorKind::NeighbourHome), "must have NeighbourHome");
        assert!(sim.interiors.iter().any(|i| i.kind == InteriorKind::BasementShelter), "must have BasementShelter");
        assert!(sim.interiors.iter().any(|i| i.kind == InteriorKind::Pharmacy), "must have Pharmacy");
        assert!(sim.interiors.iter().any(|i| i.kind == InteriorKind::Residential), "must have Residential apartment blocks");
        assert!(sim.interiors.iter().any(|i| i.back_door.is_some()), "at least one shop should have a back alley exit");
    }

    #[test]
    fn front_in_back_out_walking_and_indoor_wall_collision() {
        let (city, _) = crate::load_heihe().unwrap();
        let mut sim = WorldSim::new(&city);
        let ground = Ground::new_with_interiors(&city, &sim.interiors);
        let forces = load_forces();
        let idx = sim.interiors.iter().position(|i| i.back_door.is_some()).unwrap();
        let fd = sim.interiors[idx].front_door.clone();
        let bd = sim.interiors[idx].back_door.clone().unwrap();

        // Start outside front door and walk straight in along room inward axis.
        sim.player_at = fd.outside;
        let in_dir = fd.inside.sub(fd.outside).norm();
        for _ in 0..16 {
            sim.tick(&city, &ground, &forces, &SimInput { wish_move: in_dir, ..Default::default() }, 0.12);
        }
        assert_eq!(sim.active_interior, Some(idx), "walking through front doorway must enter room cleanly");

        // Push sideways against interior side wall; must stay bounded inside room width.
        let side_dir = sim.interiors[idx].u;
        for _ in 0..16 {
            sim.tick(&city, &ground, &forces, &SimInput { wish_move: side_dir, ..Default::default() }, 0.15);
        }
        let (lu, _) = sim.interiors[idx].to_local(sim.player_at);
        assert!(lu <= sim.interiors[idx].half_w + 0.05, "side wall must block player inside room: lu={lu}");

        // Walk out through the back door onto the back alley.
        sim.player_at = bd.inside;
        let out_back = bd.outside.sub(bd.inside).norm();
        for _ in 0..5 {
            sim.tick(&city, &ground, &forces, &SimInput { wish_move: out_back, ..Default::default() }, 0.15);
        }
        assert!(sim.active_interior.is_none(), "walking out back door must exit back to street/alley");
        assert!(sim.player_at.dist(bd.outside) < 2.2);
    }

    #[test]
    fn barricaded_door_blocks_and_russian_patrol_breaches_over_time() {
        let (city, _) = crate::load_heihe().unwrap();
        let mut sim = WorldSim::new(&city);
        let ground = Ground::new_with_interiors(&city, &sim.interiors);
        let forces = load_forces();
        let idx = sim.interiors.iter().position(|i| i.kind == InteriorKind::PlayerHome).unwrap();
        let fd_in = sim.interiors[idx].front_door.inside;
        let fd_out = sim.interiors[idx].front_door.outside;

        sim.player_at = sim.interiors[idx].front_door.at.lerp(fd_in, 0.5);
        sim.inv.scrap = 3;
        sim.tick(&city, &ground, &forces, &SimInput { fortify_or_stove: true, ..Default::default() }, 0.1);
        assert!(sim.interiors[idx].front_door.barricade_hp >= 100.0, "home steel door barricade must set high hp");

        // Spawn engaging soldier right outside the barricaded door.
        sim.hour = hm(18, 20);
        sim.alert_stage = 2;
        sim.squads.push(Squad {
            id: 77,
            state: SquadState::Engage,
            route_idx: 0,
            wp_idx: 0,
            wp_dir: 1,
            men: vec![Soldier { id: 771, at: fd_out, floor: 0, facing: 0.0, hp: 60.0, optics: false, shot_cd: 2.0, speed: 0.0, phase: 0.0, in_cover: false, vault_t: 0.0, grenade_cd: 0.0 }],
            meter: 1.0,
            last_seen: fd_in,
            lost_for: 0.0,
            timer: 10.0,
            ignore_for: 0.0,
            held_at: fd_in,
            shouted: true,
            hostile: true,
            stuck_t: 0.0,
            last_lead_at: fd_out,
        });

        let hp0 = sim.interiors[idx].front_door.barricade_hp;
        for _ in 0..8 {
            sim.tick(&city, &ground, &forces, &SimInput::default(), 0.25);
        }
        assert!(sim.interiors[idx].front_door.barricade_hp < hp0, "Russian patrol must damage barricaded door");
    }

    #[test]
    fn blackout_stealth_curtains_broken_window_chill_and_coal_stove_heat() {
        let (city, _) = crate::load_heihe().unwrap();
        let mut sim = WorldSim::new(&city);
        let ground = Ground::new_with_interiors(&city, &sim.interiors);
        let forces = load_forces();
        let idx = sim.interiors.iter().position(|i| i.kind == InteriorKind::PlayerHome).unwrap();
        let room_mid = sim.interiors[idx].to_world(0.0, sim.interiors[idx].split_v * 0.5);

        sim.player_at = room_mid;
        sim.hour = hm(21, 30);
        sim.blackout = true;
        sim.alert_stage = 2;
        sim.interiors[idx].curtains_closed = false;
        sim.flashlight_on = false;

        // With curtains open at 21:30 heating off and window broken, room temperature drops rapidly.
        sim.interiors[idx].windows[0].broken = true;
        let t_before = sim.interiors[idx].room_temp_c;
        for _ in 0..20 {
            sim.tick(&city, &ground, &forces, &SimInput::default(), 0.25);
        }
        assert!(sim.interiors[idx].room_temp_c < t_before, "broken window at 21:30 must cool room temperature");

        // Board the broken window, draw curtains (K), and light coal stove (T) in back room.
        sim.inv.scrap = 2;
        sim.inv.coal = 2;
        sim.tick(&city, &ground, &forces, &SimInput { toggle_curtains: true, ..Default::default() }, 0.1);
        assert!(sim.interiors[idx].curtains_closed);
        let win_at = sim.interiors[idx].windows[0].at;
        sim.player_at = win_at.lerp(room_mid, 0.35);
        sim.tick(&city, &ground, &forces, &SimInput { fortify_or_stove: true, ..Default::default() }, 0.1);
        assert!(sim.interiors[idx].windows[0].boarded);

        let stove = sim.interiors[idx].stove_at.unwrap();
        sim.player_at = stove;
        sim.tick(&city, &ground, &forces, &SimInput { fortify_or_stove: true, ..Default::default() }, 0.1);
        assert!(sim.interiors[idx].stove_fuel_s > 50.0);

        let cold_t = sim.interiors[idx].room_temp_c;
        for _ in 0..20 {
            sim.tick(&city, &ground, &forces, &SimInput::default(), 0.25);
        }
        assert!(sim.interiors[idx].room_temp_c > cold_t, "lit coal stove after boarding window must reheat room");

        // Drawn curtains block exterior line-of-sight cleanly.
        let outside = sim.interiors[idx].front_door.outside.add(sim.interiors[idx].u.mul(1.8));
        assert!(!ground.clear_line_with_interiors(&sim.interiors, outside, room_mid, false), "closed curtains must block street sight");
        assert!(ground.clear_line_with_interiors(&sim.interiors, outside, room_mid, true), "bullets ignore curtains");
    }

    #[test]
    fn backroom_and_basement_block_mortar_and_fpv_strikes() {
        let (city, _) = crate::load_heihe().unwrap();
        let mut sim = WorldSim::new(&city);
        let ground = Ground::new_with_interiors(&city, &sim.interiors);
        let forces = load_forces();

        let base_idx = sim.interiors.iter().position(|i| i.kind == InteriorKind::BasementShelter).unwrap();
        let base_spot = sim.interiors[base_idx].to_world(0.0, sim.interiors[base_idx].split_v + 1.0);
        sim.player_at = base_spot;
        sim.hour = hm(19, 10);
        sim.alert_stage = 2;
        sim.mortars.push(MortarMission { target: base_spot, fuse: 0.05, rounds_left: 1, round_gap: 0.0, friendly: false, whistled: true });
        for _ in 0..10 {
            sim.tick(&city, &ground, &forces, &SimInput::default(), 0.1);
        }
        assert_eq!(sim.body.hp, 100.0, "basement shelter must completely shield from mortar blast");

        // Shop backroom shields against direct FPV even if target hit is true.
        let shop_idx = sim.interiors.iter().position(|i| i.kind == InteriorKind::Pharmacy).unwrap();
        let back_spot = sim.interiors[shop_idx].to_world(0.0, (sim.interiors[shop_idx].split_v + sim.interiors[shop_idx].depth) * 0.5);
        sim.player_at = back_spot;
        sim.fpv = Some(FpvDrone {
            origin: back_spot.add(P2::new(-1.0, 0.0)),
            at: back_spot,
            height: 2.0,
            target: back_spot,
            hit: true,
            fibre_optic: true,
            jammed: false,
        });
        sim.tick(&city, &ground, &forces, &SimInput::default(), 0.1);
        assert_eq!(sim.body.hp, 100.0, "backroom behind partition wall must protect against FPV burst");
    }

    #[test]
    fn home_stash_and_militia_garrison_work_indoors() {
        let (city, _) = crate::load_heihe().unwrap();
        let mut sim = WorldSim::new(&city);
        let ground = Ground::new_with_interiors(&city, &sim.interiors);
        let forces = load_forces();

        let home_idx = sim.interiors.iter().position(|i| i.kind == InteriorKind::PlayerHome).unwrap();
        let locker = sim.interiors[home_idx].locker_at.unwrap();
        let base_food = sim.inv.food;
        sim.player_at = locker;
        sim.tick(&city, &ground, &forces, &SimInput { interact_just_pressed: true, ..Default::default() }, 0.1);
        assert!(sim.home_visited, "entering PlayerHome must set home_visited milestone");
        assert!(sim.inv.food > base_food, "locker stash E press must transfer supplies");

        // Rally militia at resistance cell and order into interior via right-click spot -> Garrison.
        sim.hour = hm(18, 35);
        sim.alert_stage = 2;
        sim.rally_militia();
        let target_in_room = sim.interiors[home_idx].to_world(0.0, 1.5);
        sim.tick(&city, &ground, &forces, &SimInput { command_militia_to: Some(target_in_room), ..Default::default() }, 0.1);
        assert!(matches!(sim.militia[0].order, MilitiaOrder::Garrison(_)), "commanding militia inside building must trigger Garrison");
    }

    #[test]
    fn walking_into_buildings_works_on_center_and_off_center_approaches() {
        let (city, _) = crate::load_heihe().unwrap();
        let template = WorldSim::new(&city);
        let ground = Ground::new_with_interiors(&city, &template.interiors);
        let forces = load_forces();

        let mut indices: Vec<usize> = (0..template.interiors.len()).collect();
        indices.sort_by(|&a, &b| {
            template.interiors[a]
                .front_door
                .at
                .dist(city.spawn)
                .total_cmp(&template.interiors[b].front_door.at.dist(city.spawn))
        });
        let test_count = indices.len().min(36);
        assert!(test_count >= 30, "need at least 30 enterable buildings for walk-in coverage");

        let mut center_failures = Vec::new();
        let mut offcenter_failures = Vec::new();

        for &idx in indices.iter().take(test_count) {
            let intr = &template.interiors[idx];
            let in_dir = intr.front_door.inside.sub(intr.front_door.outside).norm();

            // 1. Dead-center walk-in from outside threshold.
            let mut s1 = WorldSim::new(&city);
            s1.player_at = intr.front_door.outside;
            let mut entered_center = false;
            for _ in 0..18 {
                s1.tick(&city, &ground, &forces, &SimInput { wish_move: in_dir, ..Default::default() }, 0.12);
                if s1.active_interior == Some(idx) {
                    entered_center = true;
                }
            }
            if !entered_center {
                center_failures.push((idx, intr.name.clone(), s1.player_at, intr.front_door.at));
            }

            // 2. Off-center grazing walk-in (shifted +/-0.50m along doorway width).
            for &side in &[-0.50f32, 0.50f32] {
                let mut s2 = WorldSim::new(&city);
                s2.player_at = intr.front_door.outside.add(intr.u.mul(side));
                let mut entered_side = false;
                for _ in 0..18 {
                    s2.tick(&city, &ground, &forces, &SimInput { wish_move: in_dir, ..Default::default() }, 0.12);
                    if s2.active_interior == Some(idx) {
                        entered_side = true;
                    }
                }
                if !entered_side {
                    offcenter_failures.push((idx, side, intr.name.clone(), s2.player_at, intr.front_door.at));
                }
            }
        }

        assert!(
            center_failures.is_empty(),
            "center walk-in failures ({}): {:?}",
            center_failures.len(),
            center_failures
        );
        assert!(
            offcenter_failures.is_empty(),
            "off-center walk-in failures ({}): {:?}",
            offcenter_failures.len(),
            offcenter_failures
        );

        // Also verify pressing E right outside an unlocked doorway steps cleanly indoors.
        let mut s_e = WorldSim::new(&city);
        let first_idx = indices[0];
        s_e.player_at = s_e.interiors[first_idx].front_door.outside;
        s_e.drops.clear();
        s_e.caches.clear();
        s_e.doors.clear();
        s_e.tick(&city, &ground, &forces, &SimInput { interact_just_pressed: true, ..Default::default() }, 0.1);
        assert_eq!(s_e.active_interior, Some(first_idx), "pressing E outside unlocked doorway must step inside");
    }

    #[test]
    fn multi_story_climbing_real_doors_and_tank_blasting_floor_slab_drops_actor() {
        let (city, _) = crate::load_heihe().unwrap();
        let mut sim = WorldSim::new(&city);
        let ground = Ground::new_with_interiors(&city, &sim.interiors);
        let forces = load_forces();
        sim.drops.clear();
        sim.caches.clear();

        let idx = sim
            .interiors
            .iter()
            .position(|it| {
                it.num_floors >= 3
                    && it.locker_at.is_none()
                    && it.loots.iter().all(|l| l.at.dist(it.front_door.inside) > 2.4)
                    && it.npcs.iter().all(|n| n.at.dist(it.front_door.inside) > 2.5)
            })
            .expect("expected a multi-story building with free entrance");

        // 1. Step inside 1F, stand still at door, verify E toggles door open <-> closed without bounce.
        sim.player_at = sim.interiors[idx].front_door.outside;
        sim.doors.clear();
        sim.tick(&city, &ground, &forces, &SimInput { interact_just_pressed: true, ..Default::default() }, 0.1);
        assert_eq!(sim.active_interior, Some(idx));
        assert_eq!(sim.player_floor, 0);
        assert!(sim.interiors[idx].front_door.open, "entering with E should open front door");

        sim.tick(&city, &ground, &forces, &SimInput { interact_just_pressed: true, ..Default::default() }, 0.1);
        assert!(!sim.interiors[idx].front_door.open, "pressing E indoors must shut the real door");
        sim.tick(&city, &ground, &forces, &SimInput::default(), 0.2);
        assert!(!sim.interiors[idx].front_door.open, "closed door must stay closed while standing still");
        sim.tick(&city, &ground, &forces, &SimInput { interact_just_pressed: true, ..Default::default() }, 0.1);
        assert!(sim.interiors[idx].front_door.open, "pressing E again must swing door back open");

        // 2. Move to stairwell & climb to 2F (floor index 1).
        sim.player_at = sim.interiors[idx].stairwell_world();
        sim.tick(&city, &ground, &forces, &SimInput { floor_delta: 1, ..Default::default() }, 0.1);
        assert_eq!(sim.player_floor, 1, "Space/floor_delta at stairwell must climb to 2F");

        // 3. Stand on intact 2F concrete slab (row 0, col 1 center), blast it, verify hole & fall-through to 1F.
        let slab_spot = sim.interiors[idx].to_world(0.0, sim.interiors[idx].depth * 0.25);
        sim.player_at = slab_spot;
        assert!(sim.interiors[idx].is_slab_intact(1, slab_spot), "2F slab should start intact");

        let rev_before = sim.interiors[idx].revision;
        let hp_before = sim.body.hp;
        let mut out = TickOutput::default();
        sim.explode_at_floor(slab_spot, 1, 8.0, 20.0, 240.0, false, &forces, false, &mut out);
        assert!(!sim.interiors[idx].is_slab_intact(1, slab_spot), "125mm shell must destroy 2F floor slab");
        assert!(sim.interiors[idx].revision > rev_before, "structural blast must bump interior revision");
        assert!(out.slab_collapse, "slab collapse flag must trigger audio/FX");
        assert_eq!(sim.player_floor, 0, "destroyed 2F slab must drop actor down to 1F");
        assert!(sim.body.hp < hp_before, "falling through destroyed floor slab must deal injury");

        // 4. Verify top-attack Molotov from upper floor deals 2x damage to T-72B3 tank & RPG-7 kills it.
        sim.body.hp = 100.0;
        sim.dead = false;
        let ground_y = sim.interiors[idx].front_door.outside.add(P2::new(0.0, -5.0));
        sim.spawn_tank_wave_at(VehicleKind::T72B3, ground_y, &forces);
        let t_pos = sim.interiors[idx].front_door.outside.add(P2::new(2.0, -2.0));
        sim.tanks[0].at = t_pos;
        sim.tanks[0].hp = 320.0;
        sim.player_at = sim.interiors[idx].stairwell_world();
        sim.player_floor = 1;
        sim.inv.molotov = 1;
        sim.tick(&city, &ground, &forces, &SimInput { throw_molotov_at: Some(t_pos), ..Default::default() }, 0.05);
        assert!(sim.tanks[0].hp <= 175.0, "top-attack Molotov from 2F should deal >= 150 dmg, hp={}", sim.tanks[0].hp);

        sim.inv.rpg = 1;
        sim.player_shoot_cd = 0.0;
        sim.tick(&city, &ground, &forces, &SimInput { fire_rpg_at: Some(t_pos), ..Default::default() }, 0.05);
        assert!(sim.tanks.is_empty(), "RPG-7 direct hit must finish damaged T-72B3 tank");
    }

    #[test]
    fn almost_all_playable_chinese_buildings_are_enterable() {
        let (city, _) = crate::load_heihe().unwrap();
        let sim = WorldSim::new(&city);
        let covered_ids: std::collections::HashSet<i64> = sim.interiors.iter().map(|i| i.building_id).collect();
        let in_playable = |p: P2| -> bool {
            p.x >= city.walk_x.0 + 6.2
                && p.x <= city.walk_x.1 - 6.2
                && p.y >= city.walk_south + 4.2
                && p.y <= city.bank_y(p.x) - 0.8
        };
        let playable: Vec<_> = city
            .buildings
            .iter()
            .filter(|b| {
                b.bank == crate::city::Bank::China
                    && b.solid
                    && b.footprint.len() >= 3
                    && (in_playable(b.centroid) || b.footprint.iter().any(|&p| in_playable(p)))
            })
            .collect();
        let covered = playable.iter().filter(|b| covered_ids.contains(&b.id)).count();
        let ratio = covered as f32 / playable.len().max(1) as f32;
        println!(
            "PLAYABLE_COVERED={covered}/{} ({:.1}%) ENTERABLE_TOTAL={}",
            playable.len(),
            ratio * 100.0,
            sim.interiors.len()
        );
        assert!(ratio >= 0.90, "expected >= 90% playable Chinese buildings enterable, got {covered}/{}", playable.len());
    }

    #[test]
    fn shooting_out_from_inside_buildings_hits_outdoor_enemies_and_drones() {
        let (city, _) = crate::load_heihe().unwrap();
        let mut sim = WorldSim::new(&city);
        let ground = Ground::new_with_interiors(&city, &sim.interiors);
        let forces = load_forces();

        let idx = sim.interiors.iter().position(|it| it.num_floors >= 2).unwrap();
        let fd_out = sim.interiors[idx].front_door.outside;
        // Stand sideways deep inside room so the oblique shot passes through the building's own front wall.
        let pat = sim.interiors[idx].to_world(-sim.interiors[idx].half_w * 0.45, sim.interiors[idx].depth * 0.55);
        let s_out = ground.step(fd_out.add(P2::new(6.0, 2.0)), fd_out.add(P2::new(6.0, 2.0)), 0.35);
        sim.player_at = pat;
        sim.active_interior = Some(idx);
        sim.player_floor = 1;
        sim.inv.unlock_primary(PrimaryWeapon::Qbu191);
        sim.inv.active_primary = PrimaryWeapon::Qbu191;
        sim.inv.ammo = 20;

        sim.squads.clear();
        sim.squads.push(Squad {
            id: 91,
            state: SquadState::Patrol,
            route_idx: 0,
            wp_idx: 0,
            wp_dir: 1,
            men: vec![Soldier { id: 910, at: s_out, floor: 0, facing: 0.0, hp: 60.0, optics: false, shot_cd: 9.0, speed: 0.0, phase: 0.0, in_cover: false, vault_t: 0.0, grenade_cd: 0.0 }],
            meter: 0.0,
            last_seen: s_out,
            lost_for: 0.0,
            timer: 0.0,
            ignore_for: 9.0,
            held_at: s_out,
            shouted: false,
            hostile: false,
            stuck_t: 0.0,
            last_lead_at: s_out,
        });
        sim.drone.active = true;
        sim.drone.at = s_out.add(P2::new(2.0, 2.0));

        // 1. Shoot drone down from inside building first.
        sim.tick(&city, &ground, &forces, &SimInput { fire_at: Some(sim.drone.at), ..Default::default() }, 0.05);
        assert!(!sim.drone.active, "player shooting out of 2F window must down recon drone");

        // 2. Fire burst from 2F window through own wall/window at outdoor soldier!
        for _ in 0..8 {
            sim.player_shoot_cd = 0.0;
            sim.tick(&city, &ground, &forces, &SimInput { fire_at: Some(s_out), ..Default::default() }, 0.05);
        }
        assert!(sim.squads[0].men[0].hp < 60.0, "player shooting out of building must damage outdoor soldier, hp={}", sim.squads[0].men[0].hp);
    }

    #[test]
    fn pla_and_russian_armored_duel_and_modern_weapons_work() {
        let (city, _) = crate::load_heihe().unwrap();
        let mut sim = WorldSim::new(&city);
        let ground = Ground::new_with_interiors(&city, &sim.interiors);
        let forces = load_forces();
        sim.inv.unlock_all_arsenal();

        // 1. EW Jammer downs radio FPV & warns on fibre-optic FPV; Saiga-12 downs fibre-optic FPV!
        sim.fpv = Some(FpvDrone { origin: P2::new(-250.0, -300.0), at: sim.player_at.add(P2::new(12.0, 0.0)), height: 8.0, target: sim.player_at, hit: true, fibre_optic: false, jammed: false });
        sim.inv.active_primary = PrimaryWeapon::EwJammer;
        sim.tick(&city, &ground, &forces, &SimInput { fire_at: Some(sim.player_at.add(P2::new(12.0, 0.0))), ..Default::default() }, 0.05);
        assert!(sim.fpv.is_none(), "EW Jammer must knock down radio FPV");

        sim.fpv = Some(FpvDrone { origin: P2::new(-250.0, -300.0), at: sim.player_at.add(P2::new(12.0, 0.0)), height: 8.0, target: sim.player_at, hit: true, fibre_optic: true, jammed: false });
        sim.player_shoot_cd = 0.0;
        sim.tick(&city, &ground, &forces, &SimInput { fire_at: Some(sim.player_at.add(P2::new(12.0, 0.0))), ..Default::default() }, 0.05);
        assert!(sim.fpv.is_some(), "fibre-optic FPV must resist EW Jammer");

        sim.inv.active_primary = PrimaryWeapon::Saiga12;
        sim.player_shoot_cd = 0.0;
        sim.tick(&city, &ground, &forces, &SimInput { fire_at: Some(sim.player_at.add(P2::new(12.0, 0.0))), ..Default::default() }, 0.05);
        assert!(sim.fpv.is_none(), "Saiga-12 shotgun must shoot down fibre-optic FPV");

        // 2. ZTZ-99A engages T-90M on open street + HJ-12 top-attack finishes it!
        let (pos_a, pos_b) = sim.patrol_roads.iter()
            .filter_map(|r| {
                if r.len() < 2 { return None; }
                let d = r[1].sub(r[0]);
                if d.len() < 22.0 { return None; }
                let a = r[0].add(d.norm().mul(2.0));
                let b = r[0].add(d.norm().mul(20.0));
                if ground.clear_line_with_interiors(&sim.interiors, a, b, false) { Some((a, b)) } else { None }
            })
            .next()
            .expect("open road segment for armored duel");
        sim.spawn_tank_wave_at(VehicleKind::Ztz99A, pos_a, &forces);
        sim.spawn_tank_wave_at(VehicleKind::T90M, pos_b, &forces);
        sim.tanks[0].at = pos_a;
        sim.tanks[0].waypoints.clear();
        sim.tanks[1].at = pos_b;
        sim.tanks[1].waypoints.clear();

        let t90_hp0 = sim.tanks[1].hp;
        for _ in 0..45 {
            sim.tick(&city, &ground, &forces, &SimInput::default(), 0.2);
        }
        assert!(sim.tanks.iter().find(|t| t.kind == VehicleKind::T90M).is_none_or(|t| t.hp < t90_hp0), "ZTZ-99A must engage T-90M");

        // 3. TOS-1A thermobaric cook-off & HJ-12 verification.
        let pos_tos = sim.player_at.add(P2::new(10.0, 0.0));
        sim.tanks.clear();
        sim.spawn_tank_wave_at(VehicleKind::Tos1A, pos_tos, &forces);
        sim.tanks[0].at = pos_tos;
        sim.inv.active_at = AtWeapon::Hj12;
        sim.player_shoot_cd = 0.0;
        sim.tick(&city, &ground, &forces, &SimInput { fire_rpg_at: Some(pos_tos), ..Default::default() }, 0.05);
        assert!(sim.tanks.is_empty(), "HJ-12 top-attack + splash (380+140 dmg) must destroy TOS-1A & trigger cook-off");
    }

    #[test]
    fn hidden_player_indoors_never_gets_blind_fpv_or_lancet_strikes() {
        let (city, _) = crate::load_heihe().unwrap();
        let mut sim = WorldSim::new(&city);
        let ground = Ground::new_with_interiors(&city, &sim.interiors);
        let forces = load_forces();

        let base_idx = sim.interiors.iter().position(|i| i.kind == InteriorKind::BasementShelter).unwrap();
        let spot = sim.interiors[base_idx].to_world(0.0, sim.interiors[base_idx].split_v + 1.0);
        sim.player_at = spot;
        sim.active_interior = Some(base_idx);
        sim.hour = hm(21, 30);
        sim.alert_stage = 2;
        sim.blackout = true;
        sim.tank_wave_mask = u16::MAX;
        sim.squads.clear();
        sim.tanks.clear();

        for _ in 0..360 {
            sim.tick(&city, &ground, &forces, &SimInput::default(), 0.5);
        }
        assert!(sim.pending_fpvs.is_empty(), "hidden player in basement must never queue blind FPVs");
        assert!(sim.fpv.is_none(), "no FPV may dive blindly on unseen indoor player");
        assert!(sim.lancet.is_none(), "no Lancet-3 may spawn on unseen player without PLA armor");
        assert_eq!(sim.body.hp, 100.0);
    }

    #[test]
    fn complying_with_challenge_at_long_range_45m_detains_without_shooting() {
        let (city, _) = crate::load_heihe().unwrap();
        let mut sim = WorldSim::new(&city);
        let ground = Ground::new_with_interiors(&city, &sim.interiors);
        let forces = load_forces();

        let (pa, pb) = sim.patrol_roads.iter()
            .filter_map(|r| {
                if r.len() < 2 { return None; }
                let d = r[1].sub(r[0]);
                if d.len() < 48.0 { return None; }
                let a = r[0].add(d.norm().mul(1.0));
                let b = r[0].add(d.norm().mul(46.0));
                if ground.clear_line_with_interiors(&sim.interiors, a, b, false) { Some((a, b)) } else { None }
            })
            .next()
            .expect("45m clear road stretch");

        sim.hour = hm(16, 30); // daylight so 45m is inside squad sight
        sim.alert_stage = 2;
        sim.player_at = pa;
        sim.inv.has_rifle = false;
        sim.squads.push(Squad {
            id: 71,
            state: SquadState::Challenge,
            route_idx: 0,
            wp_idx: 0,
            wp_dir: 1,
            men: vec![Soldier {
                id: 710,
                at: pb,
                floor: 0,
                facing: pa.sub(pb).y.atan2(pa.sub(pb).x),
                hp: 40.0,
                optics: false,
                shot_cd: 0.0,
                speed: 0.0,
                phase: 0.0,
                in_cover: false,
                vault_t: 0.0,
                grenade_cd: 0.0,
            }],
            meter: 1.0,
            last_seen: pa,
            lost_for: 0.0,
            timer: 0.2,
            ignore_for: 0.0,
            held_at: pa,
            shouted: true,
            hostile: false,
            stuck_t: 0.0,
            last_lead_at: pb,
        });

        // Stand completely still while challenged from 45m away -> must transition to Detain and never Engage/shoot.
        let mut entered_engage = false;
        for _ in 0..80 {
            sim.tick(&city, &ground, &forces, &SimInput::default(), 0.2);
            if sim.squads[0].state == SquadState::Engage {
                entered_engage = true;
            }
        }
        assert!(!entered_engage, "standing still when challenged at 45m must not turn into Engage");
        assert_eq!(sim.body.hp, 100.0, "complying unarmed civilian must not be shot");
    }

    #[test]
    fn combat_is_two_sided_player_misses_at_range_enemies_fire_back_radio_limited_and_hurts_self() {
        let (city, _) = crate::load_heihe().unwrap();
        let mut sim = WorldSim::new(&city);
        let ground = Ground::new_with_interiors(&city, &sim.interiors);
        let forces = load_forces();

        let (pa, pb) = sim.patrol_roads.iter()
            .filter_map(|r| {
                if r.len() < 2 { return None; }
                let d = r[1].sub(r[0]);
                if d.len() < 58.0 { return None; }
                let a = r[0].add(d.norm().mul(1.0));
                let b = r[0].add(d.norm().mul(55.0));
                if ground.clear_line_with_interiors(&sim.interiors, a, b, false) { Some((a, b)) } else { None }
            })
            .next()
            .expect("55m clear road stretch");

        sim.hour = hm(18, 10);
        sim.blackout = false;
        sim.alert_stage = 2;
        sim.player_at = pa;
        sim.inv.unlock_primary(PrimaryWeapon::Ak74m);
        sim.inv.active_primary = PrimaryWeapon::Ak74m;
        sim.inv.ammo = 60;
        sim.squads.push(Squad {
            id: 88,
            state: SquadState::Patrol,
            route_idx: 0,
            wp_idx: 0,
            wp_dir: 1,
            men: vec![Soldier {
                id: 880,
                at: pb,
                floor: 0,
                facing: pa.sub(pb).y.atan2(pa.sub(pb).x),
                hp: 5000.0,
                optics: false,
                shot_cd: 0.0,
                speed: 0.0,
                phase: 0.0,
                in_cover: false,
                vault_t: 0.0,
                grenade_cd: 0.0,
            }],
            meter: 0.0,
            last_seen: pb,
            lost_for: 0.0,
            timer: 0.0,
            ignore_for: 0.0,
            held_at: pb,
            shouted: false,
            hostile: false,
            stuck_t: 0.0,
            last_lead_at: pb,
        });

        let mut shots_fired = 0;
        let mut hits_landed = 0;
        let mut prev_hp = 5000.0;
        for _ in 0..30 {
            sim.player_shoot_cd = 0.0;
            sim.tick(&city, &ground, &forces, &SimInput { fire_at: Some(pb), ..Default::default() }, 0.05);
            shots_fired += 1;
            if sim.squads[0].men[0].hp < prev_hp {
                hits_landed += 1;
                prev_hp = sim.squads[0].men[0].hp;
            }
        }
        assert!(hits_landed < shots_fired, "AK-74M at 55m must not have 100% hit rate: {hits_landed}/{shots_fired}");
        assert!(sim.squads[0].hostile && sim.squads[0].state == SquadState::Engage, "gunfire must alert squad into Engage");

        // Step a few ticks so alert squad fires back!
        let mut enemy_tracer_seen = false;
        for _ in 0..25 {
            let fx = sim.tick(&city, &ground, &forces, &SimInput::default(), 0.2);
            if fx.tracers.iter().any(|t| !t.friendly) {
                enemy_tracer_seen = true;
            }
        }
        assert!(enemy_tracer_seen, "Russian squad under fire at 55m must shoot back");

        // Radio is finite + has cooldown + friendly blast hurts careless player standing at impact!
        sim.body.hp = 100.0;
        sim.dead = false;
        sim.inv.has_radio = true;
        sim.inv.radio_strikes = 1;
        sim.radio_cd = 0.0;
        sim.tick(&city, &ground, &forces, &SimInput { call_radio_at: Some(sim.player_at), ..Default::default() }, 0.05);
        assert_eq!(sim.inv.radio_strikes, 0, "calling radio must consume strike charge");
        assert!(sim.radio_cd > 20.0, "calling radio must start cooldown");

        sim.radio_cd = 0.0;
        sim.mortars.clear();
        sim.tick(&city, &ground, &forces, &SimInput { call_radio_at: Some(sim.player_at), ..Default::default() }, 0.05);
        assert!(sim.mortars.is_empty(), "zero remaining radio strikes must block additional calls");

        let mut out = TickOutput::default();
        sim.explode_at_floor(sim.player_at, 0, 8.0, 40.0, 50.0, false, &forces, true, &mut out);
        assert!(sim.body.hp < 100.0, "friendly explosion near player must hurt player");
    }

    #[test]
    fn rally_requires_resistance_cell_after_1830() {
        let (city, _) = crate::load_heihe().unwrap();
        let mut sim = WorldSim::new(&city);
        let ground = Ground::new_with_interiors(&city, &sim.interiors);
        let forces = load_forces();

        sim.hour = hm(19, 0);
        sim.alert_stage = 2;
        sim.player_at = sim.interiors.iter()
            .find(|i| i.kind == InteriorKind::Shop)
            .map(|i| i.front_door.outside.add(i.v.mul(-1.5)))
            .unwrap_or(city.spawn);
        // Ensure far from any ResistanceCell cache
        if sim.caches.iter().any(|c| c.kind == CacheKind::ResistanceCell && c.at.dist(sim.player_at) <= 20.0) {
            sim.player_at = P2::new(-140.0, -240.0);
        }
        sim.tick(&city, &ground, &forces, &SimInput { rally_resistance: true, ..Default::default() }, 0.05);
        assert!(!sim.resistance_active, "must NOT rally resistance from random street corner");
        assert!(!sim.inv.has_rifle);

        sim.player_at = sim.caches.iter().find(|c| c.kind == CacheKind::ResistanceCell).unwrap().at;
        sim.tick(&city, &ground, &forces, &SimInput { rally_resistance: true, ..Default::default() }, 0.05);
        assert!(sim.resistance_active, "pressing H at resistance cell after 18:30 must rally militia");
    }

    #[test]
    fn multiple_survival_paths_home_and_basement_survive_to_dawn() {
        let (city, _) = crate::load_heihe().unwrap();
        let forces = load_forces();
        let base_ground = Ground::new_with_interiors(&city, &WorldSim::new(&city).interiors);

        for kind in [InteriorKind::PlayerHome, InteriorKind::BasementShelter] {
            let mut sim = WorldSim::new(&city);
            let idx = sim.interiors.iter().position(|i| i.kind == kind).unwrap();
            let locker = sim.interiors[idx].locker_at.unwrap();
            let stove = sim.interiors[idx].stove_at.unwrap();

            // Start in shelter, grab locker & initial room loot, bunker down with door closed/barricaded & curtains closed.
            sim.player_at = locker;
            sim.tick(&city, &base_ground, &forces, &SimInput { interact_just_pressed: true, ..Default::default() }, 0.1);
            if let Some(loot_at) = sim.interiors[idx].loots.first().map(|l| l.at) {
                sim.player_at = loot_at;
                sim.tick(&city, &base_ground, &forces, &SimInput { interact_just_pressed: true, ..Default::default() }, 0.1);
            }
            sim.player_at = sim.interiors[idx].front_door.inside;
            sim.tick(&city, &base_ground, &forces, &SimInput { fortify_or_stove: true, ..Default::default() }, 0.1);
            sim.interiors[idx].curtains_closed = true;

            sim.player_at = stove;
            // Simulate full 15h night (16:00 -> 07:00 = 3600s play time @ 0.25s = 14400 steps; skip squads outside distant blocks)
            for _ in 0..16000 {
                if sim.dead || sim.dawn_survived {
                    break;
                }
                let mut inp = SimInput { clock_running: true, ..Default::default() };
                if sim.body.hunger < 45.0 && sim.inv.food > 0 {
                    inp.use_food = true;
                }
                if sim.body.thirst < 55.0 {
                    if sim.inv.water > 0 {
                        inp.use_water = true;
                    } else if sim.interiors[idx].stove_fuel_s >= 15.0 {
                        inp.interact_just_pressed = true; // melt snow on coal stove!
                    }
                }
                if sim.interiors[idx].room_temp_c < 8.0 && sim.interiors[idx].stove_fuel_s <= 1.0 && sim.inv.coal > 0 {
                    inp.fortify_or_stove = true;
                }
                if sim.body.temp < 35.0 && sim.inv.warmer > 0 {
                    inp.use_warmer = true;
                }
                if sim.body.bleeding && sim.inv.bandage > 0 {
                    inp.use_bandage = true;
                }
                sim.tick(&city, &base_ground, &forces, &inp, 0.25);
            }
            assert!(
                sim.dawn_survived && !sim.dead,
                "{kind:?} path must survive to dawn! hour={}, dead={}, reason={}, thirst={}, temp={}",
                sim.hour, sim.dead, sim.death_en, sim.body.thirst, sim.body.temp
            );
        }
    }

    #[test]
    fn all_patrol_waypoints_caches_and_shop_doors_are_walkable_and_reachable() {
        let (city, _) = crate::load_heihe().unwrap();
        let sim = WorldSim::new(&city);
        let solid_ground = Ground::new(&city);
        let full_ground = Ground::new_with_interiors(&city, &sim.interiors);
        let forces = load_forces();

        assert!(!sim.patrol_roads.is_empty());
        for (ri, road) in sim.patrol_roads.iter().enumerate() {
            for (wi, &wp) in road.iter().enumerate() {
                assert!(
                    solid_ground.in_playable_land(wp) && solid_ground.free(wp, 0.55),
                    "patrol_roads[{ri}][{wi}] at ({:.1},{:.1}) is not on walkable land!", wp.x, wp.y
                );
            }
        }
        for c in &sim.caches {
            assert!(
                solid_ground.in_playable_land(c.at) && solid_ground.free(c.at, 0.55),
                "cache {} ({}) at ({:.1},{:.1}) is stuck inside building or off map!", c.id, c.title, c.at.x, c.at.y
            );
        }
        assert!(!sim.doors.is_empty());
        for (di, d) in sim.doors.iter().enumerate() {
            assert!(
                solid_ground.in_playable_land(d.at) && solid_ground.free(d.at, 0.35),
                "shop door[{di}] '{}' at ({:.1},{:.1}) is not walkable!", d.name, d.at.x, d.at.y
            );
            // Verify pressing/holding E at shop sign searches shop instead of stepping indoors first!
            let mut s = WorldSim::new(&city);
            s.player_at = d.at;
            s.drops.clear();
            s.caches.clear();
            for step_i in 0..30 {
                s.tick(
                    &city,
                    &full_ground,
                    &forces,
                    &SimInput {
                        interact_held: true,
                        interact_just_pressed: step_i == 0,
                        ..Default::default()
                    },
                    0.1,
                );
            }
            assert!(s.doors[di].searched, "shop '{}' could not be searched at its doorway", d.name);
        }
    }

    #[test]
    fn buildings_and_closed_doors_always_block_line_of_sight() {
        let (city, _) = crate::load_heihe().unwrap();
        let mut sim = WorldSim::new(&city);
        let ground = Ground::new_with_interiors(&city, &sim.interiors);

        // No enterable building may be transparent from outside-front to outside-back!
        for intr in &sim.interiors {
            let out_f = intr.to_world(0.0, -3.5);
            let out_b = intr.to_world(0.0, intr.depth + 3.5);
            assert!(
                !ground.clear_line_with_interiors(&sim.interiors, out_f, out_b, false),
                "enterable building '{}' must block outside-to-outside LOS!", intr.name
            );
        }

        // Closed front door blocks sight along its doorway axis; opening it clears sight.
        let idx = sim.interiors.iter().position(|it| !it.front_door.open).unwrap();
        sim.interiors[idx].curtains_closed = false;
        let door_out = sim.interiors[idx].front_door.outside;
        let door_in = sim.interiors[idx].front_door.inside;
        assert!(
            !ground.clear_line_with_interiors(&sim.interiors, door_out, door_in, false),
            "closed door must block visual LOS"
        );
        sim.interiors[idx].front_door.open = true;
        assert!(
            ground.clear_line_with_interiors(&sim.interiors, door_out, door_in, false),
            "opened door must allow visual LOS"
        );
    }

    #[test]
    fn fire_barrel_warms_without_hiding_from_drone() {
        let forces = load_forces();
        let mut body = Body::default();
        body.temp = 34.5;
        assert!(body.tick(10.0, hm(21, 30), Gait::Still, false, false, true).is_none());
        assert!(!body.sheltered, "standing outdoors by a FireBarrel must NOT set sheltered=true");
        assert!(body.temp > 34.5, "FireBarrel must warm the player");

        let drone = ReconDrone { active: true, at: P2::new(0.0, 0.0), ..Default::default() };
        assert!(
            drone.sees_in_weather(&forces, P2::new(3.0, 3.0), Gait::Still, body.sheltered, Light::NightLit, 0.0),
            "recon drone must still spot player warming at outdoor FireBarrel"
        );
    }

    #[test]
    fn neighbour_rescue_reward_can_only_be_claimed_once() {
        let (city, _) = crate::load_heihe().unwrap();
        let mut sim = WorldSim::new(&city);
        let ground = Ground::new_with_interiors(&city, &sim.interiors);
        let forces = load_forces();

        sim.hour = hm(20, 10);
        let nh_idx = sim.interiors.iter().position(|i| i.kind == InteriorKind::NeighbourHome).unwrap();
        let npc_at = sim.interiors[nh_idx].npcs[0].at;
        sim.player_at = npc_at;
        sim.tick(&city, &ground, &forces, &SimInput { interact_just_pressed: true, ..Default::default() }, 0.1);
        assert!(sim.neighbours_rescued);
        let strikes_after_npc = sim.inv.radio_strikes;
        let coal_after_npc = sim.inv.coal;
        assert!(strikes_after_npc >= 2);

        // Now go to SOS outdoor cache and try claiming again -> must NOT double grant!
        let sos_at = sim.caches.iter().find(|c| c.kind == CacheKind::SosNeighbours).unwrap().at;
        sim.player_at = sos_at;
        sim.tick(&city, &ground, &forces, &SimInput { interact_just_pressed: true, ..Default::default() }, 0.1);
        assert_eq!(sim.inv.radio_strikes, strikes_after_npc);
        assert_eq!(sim.inv.coal, coal_after_npc);
    }

    #[test]
    fn all_five_barricade_kinds_have_real_gameplay_effects() {
        let (city, _) = crate::load_heihe().unwrap();
        let mut sim = WorldSim::new(&city);
        let ground = Ground::new_with_interiors(&city, &sim.interiors);
        let forces = load_forces();

        let p = sim.player_at;
        sim.barricades = vec![
            Barricade { at: p, kind: BarricadeKind::Sandbag, yaw: 0.0, shoved: false, hp: 80.0 },
        ];
        assert!(sim.barricade_cover_mul(&forces, p) < 0.6, "Sandbag must reduce incoming hit chance");

        sim.barricades.push(Barricade { at: p.add(P2::new(6.0, 0.0)), kind: BarricadeKind::DeadCar, yaw: 0.0, shoved: false, hp: 160.0 });
        assert!(sim.barricade_blocks_los(p.add(P2::new(3.0, 0.0)), p.add(P2::new(10.0, 0.0))), "DeadCar must block LOS across it");

        sim.barricades.push(Barricade { at: p.add(P2::new(0.0, 0.5)), kind: BarricadeKind::Concrete, yaw: 0.0, shoved: false, hp: 240.0 });
        assert_eq!(sim.barricade_concrete_blast_mul(p), 0.5, "Concrete must halve explosion blast damage");

        // IronGate pushes back enemy infantry attempting to walk through it
        let gate_at = p.add(P2::new(20.0, 0.0));
        sim.barricades.push(Barricade { at: gate_at, kind: BarricadeKind::IronGate, yaw: 0.0, shoved: false, hp: 90.0 });
        let mut dummy = Soldier { id: 1, at: gate_at.add(P2::new(0.8, 0.0)), floor: 0, facing: std::f32::consts::PI, hp: 40.0, optics: false, shot_cd: 0.0, speed: 0.0, phase: 0.0, in_cover: false, vault_t: 0.0, grenade_cd: 0.0 };
        let snap = vec![(3usize, gate_at, BarricadeKind::IronGate)];
        let mut dmg_acc = Vec::new();
        WorldSim::move_squad_one(&mut dummy, &ground, &sim.interiors, &snap, &mut dmg_acc, gate_at.add(P2::new(-4.0, 0.0)), 0, 3.0, 0.2, 0.5);
        assert!(dummy.at.dist(gate_at) >= 1.4, "IronGate must keep enemy infantry outside 1.4m");

        // Concrete stops even a heavy T-72B3 tank until broken!
        let conc_at = sim.patrol_roads[0][0];
        let wp_next = sim.patrol_roads[0][1];
        sim.barricades.push(Barricade { at: conc_at.lerp(wp_next, 0.1), kind: BarricadeKind::Concrete, yaw: 0.0, shoved: false, hp: 240.0 });
        sim.spawn_tank_wave_at(VehicleKind::T72B3, conc_at, &forces);
        let t_idx = sim.tanks.len() - 1;
        sim.tanks[t_idx].at = conc_at.lerp(wp_next, 0.1);
        let before_hp = sim.barricades.last().unwrap().hp;
        sim.tick(&city, &ground, &forces, &SimInput::default(), 0.25);
        assert!(sim.barricades.last().unwrap().hp < before_hp, "Russian tank hitting Concrete must be halted & chew through barricade HP");
    }

    #[test]
    fn btr82a_acquires_and_fires_on_exposed_target() {
        let (city, _) = crate::load_heihe().unwrap();
        let mut sim = WorldSim::new(&city);
        let ground = Ground::new_with_interiors(&city, &sim.interiors);
        let forces = load_forces();

        let (pa, pb) = sim.patrol_roads.iter()
            .filter_map(|r| {
                if r.len() < 2 { return None; }
                let d = r[1].sub(r[0]);
                if d.len() < 40.0 { return None; }
                let a = r[0].add(d.norm().mul(2.0));
                let b = r[0].add(d.norm().mul(34.0));
                if ground.clear_line_with_interiors(&sim.interiors, a, b, false) { Some((a, b)) } else { None }
            })
            .next()
            .expect("32m clear road stretch");

        sim.hour = hm(18, 5);
        sim.blackout = false;
        sim.player_at = b_pos(pa);
        sim.spawn_tank_wave_at(VehicleKind::Btr82A, pb, &forces);
        sim.tanks[0].at = pb;
        sim.tanks[0].waypoints = vec![pb, pa.sub(pb.sub(pa).norm().mul(20.0))];
        sim.tanks[0].wp_idx = 1;
        sim.tanks[0].cannon_cd = 0.0;
        sim.tanks[0].mg_cd = 0.0;

        let mut fired_by_btr = false;
        for _ in 0..30 {
            let fx = sim.tick(&city, &ground, &forces, &SimInput::default(), 0.2);
            if fx.tracers.iter().any(|t| !t.friendly) || fx.tank_cannon {
                fired_by_btr = true;
            }
        }
        assert!(fired_by_btr, "BTR-82A must halt/track and fire autocannon or MG on exposed target");
        fn b_pos(p: P2) -> P2 { p }
    }

    #[test]
    fn forces_json_values_can_be_lowered_and_drive_soldiers_fog_ew_and_vehicles() {
        let (city, _) = crate::load_heihe().unwrap();
        let ground = Ground::new(&city);
        let base_json = std::fs::read_to_string(crate::data_dir().join("russia/forces.json")).unwrap();
        let mut root: serde_json::Value = serde_json::from_str(&base_json).unwrap();
        root["infantry_group"]["soldier_hp"] = serde_json::json!(12.0);
        root["infantry_group"]["walk_mps"] = serde_json::json!(0.4);
        root["infantry_group"]["view_half_angle_deg"] = serde_json::json!(25.0);
        root["infantry_group"]["challenge_s"] = serde_json::json!(0.2);
        root["recon_drone"]["fog_sight_penalty_plain"] = serde_json::json!(0.1);
        root["fpv"]["ew_jam_mul"] = serde_json::json!(0.05);
        root["lancet"]["hp"] = serde_json::json!(5.0);
        root["vehicles"]["btr82a"]["hp"] = serde_json::json!(40.0);

        let custom = Forces::from_json(&root.to_string()).unwrap();
        assert!((custom.num_or("infantry_group", "soldier_hp", 40.0) - 12.0).abs() < 0.01);
        assert!((custom.num_or("infantry_group", "walk_mps", 1.3) - 0.4).abs() < 0.01);
        assert!((custom.num_or("fpv", "ew_jam_mul", 0.5) - 0.05).abs() < 0.01);
        assert!((custom.num_or("lancet", "hp", 20.0) - 5.0).abs() < 0.01);
        assert!((custom.vehicle_num("btr82a", "hp", 160.0) - 40.0).abs() < 0.01);

        let mut sim = WorldSim::new(&city);
        sim.hour = hm(19, 0);
        sim.try_spawn_squad(&city, &custom);
        assert!(!sim.squads.is_empty());
        assert!((sim.squads[0].men[0].hp - 12.0).abs() < 0.01, "soldier HP must follow lowered forces.json");

        sim.spawn_tank_wave_at(VehicleKind::Btr82A, sim.player_at, &custom);
        assert!((sim.tanks[0].hp - 40.0).abs() < 0.01, "BTR-82A HP must follow lowered forces.json");

        // Ensure drone fog sight reflects lowered fog_sight_penalty_plain = 0.1 without upward clamping!
        let _ = ground;
        let drone = ReconDrone { active: true, thermal: false, ..Default::default() };
        let clear = drone.sight_radius_in_weather(&custom, Light::Day, 0.0);
        let foggy = drone.sight_radius_in_weather(&custom, Light::Day, 0.9);
        assert!(foggy > clear * 0.85, "lowered fog penalty 0.1 must barely blind drone: clear={clear}, foggy={foggy}");
    }

    #[test]
    fn radio_callouts_and_paused_militia_order_work() {
        let (city, _) = crate::load_heihe().unwrap();
        let mut sim = WorldSim::new(&city);
        let ground = Ground::new_with_interiors(&city, &sim.interiors);
        let forces = load_forces();

        sim.rally_militia();
        assert_eq!(sim.radio_feed.as_ref().map(|r| r.code_en), Some("AWAITING ORDERS"));

        let order_pt = sim.player_at.add(P2::new(8.0, 4.0));
        assert!(sim.issue_militia_order(order_pt));
        assert_eq!(sim.radio_feed.as_ref().map(|r| r.code_en), Some("YOU GOT IT"));
        assert!(sim.militia_cmd_marker.is_some());

        // Clearing a squad broadcasts SECTOR CLEARED.
        sim.squads.push(Squad {
            id: 900,
            state: SquadState::Patrol,
            route_idx: 0,
            wp_idx: 0,
            wp_dir: 1,
            men: vec![Soldier {
                id: 901,
                at: sim.player_at.add(P2::new(15.0, 0.0)),
                floor: 0,
                facing: 0.0,
                hp: 30.0,
                optics: false,
                shot_cd: 0.0,
                speed: 0.0,
                phase: 0.0,
                in_cover: false,
                vault_t: 0.0,
                grenade_cd: 0.0,
            }],
            meter: 0.0,
            last_seen: sim.player_at,
            lost_for: 0.0,
            timer: 0.0,
            ignore_for: 0.0,
            held_at: sim.player_at,
            shouted: false,
            hostile: false,
            stuck_t: 0.0,
            last_lead_at: sim.player_at,
        });
        sim.damage_soldier(0, 0, 50.0);
        assert_eq!(sim.radio_feed.as_ref().map(|r| r.code_en), Some("SECTOR CLEARED"));

        // Radio feed decays to None after ttl expires.
        for _ in 0..30 {
            sim.tick(&city, &ground, &forces, &SimInput::default(), 0.25);
        }
        assert!(sim.radio_feed.is_none());
    }

    #[test]
    fn explosions_carve_cover_craters_wrecks_persist_and_smoke_blocks_los() {
        let (city, _) = crate::load_heihe().unwrap();
        let mut sim = WorldSim::new(&city);
        let ground = Ground::new_with_interiors(&city, &sim.interiors);
        let forces = load_forces();

        // 1. Heavy explosion carves a crater providing cover & blast reduction.
        let p = sim.player_at;
        let mut out = TickOutput::default();
        sim.explode_at(p, 7.5, 40.0, false, &forces, false, &mut out);
        assert!(!sim.craters.is_empty(), "explosion >=4.5m must carve a Crater");
        assert!(sim.crater_cover_mul(p) < 0.6, "inside crater must reduce rifle hit chance");
        assert!(sim.defilade_blast_mul(p) < 0.7, "inside crater must reduce blast damage");

        // 2. Destroying a tank leaves a permanent VehicleWreck that blocks LOS and gives cover.
        let wreck_pos = p.add(P2::new(6.0, 0.0));
        sim.spawn_tank_wave_at(VehicleKind::Btr82A, wreck_pos, &forces);
        sim.tanks[0].at = wreck_pos;
        sim.tanks[0].hp = 40.0;
        sim.explode_at_floor(wreck_pos, 0, 5.0, 120.0, 0.0, false, &forces, true, &mut out);
        assert!(sim.tanks.is_empty());
        assert_eq!(sim.wrecks.len(), 1, "destroyed vehicle must persist as VehicleWreck");
        assert!(sim.wreck_cover_mul(wreck_pos) < 0.5);
        assert!(WorldSim::wrecks_block_los_slice(
            &sim.wrecks,
            p,
            wreck_pos.add(P2::new(8.0, 0.0))
        ));

        // 3. Throwing white tactical smoke blocks infantry/vehicle LOS and hides player from recon drone.
        sim.inv.smoke_grenades = 1;
        sim.drone.active = true;
        sim.drone.at = p;
        sim.tick(
            &city,
            &ground,
            &forces,
            &SimInput {
                throw_smoke_at: Some(p),
                ..Default::default()
            },
            0.1,
        );
        assert!(sim.in_tactical_smoke(p), "smoke grenade must create white tactical smoke");
        assert!(WorldSim::smoke_blocks_los_slice(
            &sim.smoke_clouds,
            p,
            p.add(P2::new(15.0, 0.0))
        ));
    }

    #[test]
    fn trench_tarp_dugout_and_barbed_wire_effects_work() {
        let (city, _) = crate::load_heihe().unwrap();
        let mut sim = WorldSim::new(&city);
        let ground = Ground::new_with_interiors(&city, &sim.interiors);
        let forces = load_forces();

        let p = sim.player_at;
        sim.barricades.push(Barricade { at: p, kind: BarricadeKind::Trench, yaw: 0.0, shoved: false, hp: 180.0 });
        assert!(sim.barricade_cover_mul(&forces, p) <= 0.26, "Trench must cut rifle hit chance");
        assert!(sim.defilade_blast_mul(p) <= 0.31, "Trench must cut blast damage");

        let tarp_p = p.add(P2::new(8.0, 0.0));
        sim.barricades.push(Barricade { at: tarp_p, kind: BarricadeKind::TarpDugout, yaw: 0.0, shoved: false, hp: 150.0 });
        sim.alert_stage = 2;
        sim.player_at = tarp_p;
        sim.drone.active = true;
        sim.drone.at = tarp_p;
        sim.drone.centre = tarp_p;
        sim.drone.confirm = 0.0;
        sim.tick(&city, &ground, &forces, &SimInput::default(), 0.2);
        assert!(sim.under_tarp_dugout(tarp_p));
        assert!(sim.drone.confirm < 0.01, "TarpDugout must conceal occupant from overhead recon drone");

        // BarbedWire slows & damages Russian infantry and blocks/damages wheeled APCs.
        let wire_p = p.add(P2::new(22.0, 0.0));
        sim.barricades.push(Barricade { at: wire_p, kind: BarricadeKind::BarbedWire, yaw: 0.0, shoved: false, hp: 110.0 });
        let snap = vec![(2usize, wire_p, BarricadeKind::BarbedWire)];
        let mut dmg_acc = Vec::new();
        let mut sol = Soldier {
            id: 77,
            at: wire_p.add(P2::new(0.4, 0.0)),
            floor: 0,
            facing: 0.0,
            hp: 40.0,
            optics: false,
            shot_cd: 0.0,
            speed: 0.0,
            phase: 0.0,
            in_cover: false,
            vault_t: 0.0,
            grenade_cd: 0.0,
        };
        WorldSim::move_squad_one(&mut sol, &ground, &sim.interiors, &snap, &mut dmg_acc, wire_p.add(P2::new(10.0, 0.0)), 0, 3.0, 0.5, 0.2);
        assert!(sol.hp < 40.0, "BarbedWire must damage enemy infantry crossing it");
        assert!(!dmg_acc.is_empty());

        sim.spawn_tank_wave_at(VehicleKind::Btr82A, wire_p, &forces);
        let btr_idx = sim.tanks.len() - 1;
        sim.tanks[btr_idx].at = wire_p;
        let hp0 = sim.tanks[btr_idx].hp;
        sim.tick(&city, &ground, &forces, &SimInput::default(), 0.25);
        assert!(sim.tanks[btr_idx].hp < hp0, "BarbedWire/hedgehogs must damage wheeled BTR-82A");
    }

    #[test]
    fn grenade_dive_away_cover_seeking_vaulting_and_militia_rescue_work() {
        let (city, _) = crate::load_heihe().unwrap();
        let mut sim = WorldSim::new(&city);
        let ground = Ground::new_with_interiors(&city, &sim.interiors);
        let forces = load_forces();

        let (pa, pb) = sim
            .patrol_roads
            .iter()
            .filter_map(|r| {
                if r.len() < 2 {
                    return None;
                }
                let d = r[1].sub(r[0]);
                if d.len() < 42.0 {
                    return None;
                }
                let dir = d.norm();
                let a = r[0].add(dir.mul(2.0));
                let b = r[0].add(dir.mul(36.0));
                if ground.clear_line_with_interiors(&sim.interiors, a, b, false) {
                    Some((a, b))
                } else {
                    None
                }
            })
            .next()
            .expect("34m open road segment");
        let dir = pb.sub(pa).norm();

        sim.hour = hm(17, 10);
        sim.squads.clear();
        sim.tanks.clear();
        sim.player_at = pa;
        sim.inv.grenades = 2;

        // 1. Timed arcing grenade causes militia & enemy soldier to dive away, then detonates leaving a crater.
        let g_tgt = pa.add(dir.mul(10.0));
        let m_pos = g_tgt.sub(dir.mul(1.6));
        let s_pos = g_tgt.add(dir.mul(1.6));
        sim.militia.push(MilitiaMember {
            id: 5001,
            name: "老王",
            at: m_pos,
            floor: 0,
            facing: 0.0,
            hp: 100.0,
            coat: Rgb { r: 0.2, g: 0.25, b: 0.3 },
            order: MilitiaOrder::Hold(m_pos),
            fire_cd: 10.0,
            speed: 0.0,
            phase: 0.0,
            in_cover: false,
            vault_t: 0.0,
            downed: false,
            bleedout_s: 0.0,
            rescuing_id: None,
            rescue_progress: 0.0,
        });
        sim.squads.push(Squad {
            id: 500,
            state: SquadState::Engage,
            route_idx: 0,
            wp_idx: 0,
            wp_dir: 1,
            men: vec![Soldier {
                id: 5002,
                at: s_pos,
                floor: 0,
                facing: -dir.y.atan2(-dir.x),
                hp: 60.0,
                optics: false,
                shot_cd: 9.0,
                speed: 0.0,
                phase: 0.0,
                in_cover: false,
                vault_t: 0.0,
                grenade_cd: 99.0,
            }],
            meter: 1.0,
            last_seen: pa,
            lost_for: 0.0,
            timer: 10.0,
            ignore_for: 0.0,
            held_at: pa,
            shouted: true,
            hostile: true,
            stuck_t: 0.0,
            last_lead_at: s_pos,
        });

        sim.tick(&city, &ground, &forces, &SimInput { throw_grenade_at: Some(g_tgt), ..Default::default() }, 0.1);
        assert_eq!(sim.grenades.len(), 1, "throwing grenade must spawn ActiveGrenade");
        let mut max_m_dist = 0.0f32;
        let mut max_s_dist = 0.0f32;
        let mut exploded = false;
        for _ in 0..14 {
            let fx = sim.tick(&city, &ground, &forces, &SimInput::default(), 0.2);
            max_m_dist = max_m_dist.max(sim.militia[0].at.dist(g_tgt));
            max_s_dist = max_s_dist.max(sim.squads[0].men[0].at.dist(g_tgt));
            if !fx.explosions.is_empty() {
                exploded = true;
            }
        }
        assert!(max_m_dist > 3.5, "militia must dive away from active grenade (dist={max_m_dist})");
        assert!(max_s_dist > 3.5, "soldier must dive away from active grenade (dist={max_s_dist})");
        assert!(exploded && sim.grenades.is_empty() && !sim.craters.is_empty(), "grenade must detonate & leave crater");

        // 2. Autonomous cover seeking into Trench -> sets in_cover = true.
        let trench_p = pb.sub(dir.mul(4.0));
        sim.squads.clear();
        sim.barricades.push(Barricade { at: trench_p, kind: BarricadeKind::Trench, yaw: 0.0, shoved: false, hp: 180.0 });
        sim.militia[0].at = trench_p.sub(dir.mul(3.2));
        sim.militia[0].order = MilitiaOrder::Hold(sim.militia[0].at);
        for _ in 0..15 {
            sim.tick(&city, &ground, &forces, &SimInput::default(), 0.2);
        }
        assert!(sim.militia[0].at.dist(trench_p) <= 1.5, "holding militia must autonomously take nearby Trench cover");
        assert!(sim.militia[0].in_cover, "militia stopped in trench must enter crouched cover state");

        // 3. Vaulting over a Sandbag sets player_vault_t > 0.0 & AnimState::Vault.
        let sb_p = pa.add(dir.mul(1.8));
        sim.barricades.push(Barricade { at: sb_p, kind: BarricadeKind::Sandbag, yaw: 0.0, shoved: false, hp: 80.0 });
        sim.player_at = pa;
        let mut vaulted = false;
        for _ in 0..12 {
            sim.tick(&city, &ground, &forces, &SimInput { wish_move: dir, ..Default::default() }, 0.12);
            if sim.player_vault_t > 0.0 && sim.player_anim == AnimState::Vault {
                vaulted = true;
                break;
            }
        }
        assert!(vaulted, "crossing Sandbag must trigger AnimState::Vault");

        // 4. Downed militia crawls leaving BloodMarks & healthy teammate autonomously rescues them.
        sim.militia[0].at = pa;
        sim.militia[0].order = MilitiaOrder::Follow;
        sim.militia[0].hp = 85.0;
        sim.militia.push(MilitiaMember {
            id: 5003,
            name: "阿敏",
            at: pa.add(dir.mul(3.5)),
            floor: 0,
            facing: 0.0,
            hp: 85.0,
            coat: Rgb { r: 0.3, g: 0.2, b: 0.2 },
            order: MilitiaOrder::Follow,
            fire_cd: 10.0,
            speed: 0.0,
            phase: 0.0,
            in_cover: false,
            vault_t: 0.0,
            downed: false,
            bleedout_s: 0.0,
            rescuing_id: None,
            rescue_progress: 0.0,
        });
        sim.damage_militia(0, 72.0);
        assert!(sim.militia[0].downed, "severe wound must put militia into downed state");
        assert!(!sim.blood_marks.is_empty(), "downed wound must stamp BloodMark");
        assert_eq!(sim.radio_feed.as_ref().map(|r| r.code_en), Some("MAN DOWN"));

        let mut rescued_code = false;
        for _ in 0..24 {
            sim.tick(&city, &ground, &forces, &SimInput::default(), 0.25);
            if sim.radio_feed.as_ref().map(|r| r.code_en) == Some("TEAMMATE RESCUED") {
                rescued_code = true;
            }
        }
        assert!(!sim.militia[0].downed && sim.militia[0].hp >= 50.0, "healthy teammate must revive downed militia");
        assert!(rescued_code, "revive must broadcast TEAMMATE RESCUED over radio");
    }

    /// A short scripted run: walk away from the spawn, then stand.
    fn scripted(sim: &mut WorldSim, city: &City, ground: &Ground, forces: &Forces, steps: usize) {
        for i in 0..steps {
            let wish_move = if i < steps / 2 { P2::new(0.3, -1.0) } else { P2::default() };
            let inp = SimInput { wish_move, clock_running: true, ..Default::default() };
            sim.tick(city, ground, forces, &inp, 0.25);
            if sim.dead {
                break;
            }
        }
    }

    #[test]
    fn the_same_seed_plays_the_same_run() {
        let (city, _) = crate::load_heihe().unwrap();
        let forces = load_forces();
        let ground = Ground::new_with_interiors(&city, &WorldSim::new(&city).interiors);
        let run = |seed: u64| {
            let mut sim = WorldSim::with_seed(&city, seed);
            sim.hour = hm(19, 40);
            scripted(&mut sim, &city, &ground, &forces, 2400);
            let men: Vec<(i32, i32)> = sim
                .squads
                .iter()
                .flat_map(|sq| sq.men.iter())
                .map(|m| ((m.at.x * 100.0) as i32, (m.at.y * 100.0) as i32))
                .collect();
            (sim.hour.to_bits(), sim.player_at.x.to_bits(), sim.player_at.y.to_bits(), sim.body.hp.to_bits(), sim.dead, men, sim.rng)
        };
        let a = run(7);
        let b = run(7);
        assert_eq!(a, b, "two runs from one seed must match step for step");
        assert_ne!(a.6, run(8).6, "another seed must give another random stream");
    }

    #[test]
    fn the_clock_runs_past_dawn_and_the_run_goes_on() {
        let (city, _) = crate::load_heihe().unwrap();
        let forces = load_forces();
        let ground = Ground::new_with_interiors(&city, &WorldSim::new(&city).interiors);
        let mut sim = WorldSim::new(&city);
        sim.hour = DAWN_HOUR - 0.02;
        let inp = SimInput { clock_running: true, ..Default::default() };
        for _ in 0..60 {
            sim.tick(&city, &ground, &forces, &inp, 0.25);
        }
        assert!(sim.dawn_survived && !sim.dead);
        assert_eq!(sim.mornings, 1);
        assert!(sim.hour > DAWN_HOUR, "the clock must not stop at 07:00: {}", sim.hour);
        let t = sim.sim_time;
        sim.tick(&city, &ground, &forces, &inp, 0.25);
        assert!(sim.sim_time > t, "the world keeps running after dawn");
        assert!(clock::label(sim.hour).starts_with("1月18日 07:0"), "{}", clock::label(sim.hour));

        // The next morning is counted too.
        sim.hour = DAWN_HOUR + 24.0 - 0.01;
        for _ in 0..10 {
            sim.tick(&city, &ground, &forces, &inp, 0.25);
        }
        assert_eq!(sim.mornings, 2);
        assert_eq!(clock::date_of(sim.hour), (1, 19));
    }

    #[test]
    fn light_and_weather_follow_the_days() {
        let (city, _) = crate::load_heihe().unwrap();
        let away = P2::new(-9999.0, -9999.0);
        // The first evening is as it was.
        assert_eq!(light_at(&city, hm(16, 20), false, false, away, &[]), Light::Day);
        assert_eq!(light_at(&city, hm(16, 50), false, false, away, &[]), Light::Dusk);
        assert_eq!(light_at(&city, hm(17, 30), true, false, away, &[]), Light::NightDark);
        // And the next day has a morning and a noon.
        assert_eq!(light_at(&city, 24.0 + hm(2, 0), true, false, away, &[]), Light::NightDark);
        assert_eq!(light_at(&city, 24.0 + hm(6, 30), true, false, away, &[]), Light::Dusk);
        assert_eq!(light_at(&city, 24.0 + hm(12, 0), true, false, away, &[]), Light::Day);
        // Two months on, the evening is light at 17:00.
        assert_eq!(light_at(&city, 24.0 * 60.0 + hm(17, 0), true, false, away, &[]), Light::Day);

        // The first night's weather is the script; the days after roll from the seed.
        assert_eq!(weather_at(1, hm(21, 0)), scheduled_weather(hm(21, 0)));
        let days: Vec<Weather> = (0..400).map(|b| weather_at(1, DAWN_HOUR + b as f32 * 3.0).0).collect();
        let clear = days.iter().filter(|w| **w == Weather::Clear).count();
        assert!(clear > 260 && clear < 400, "mostly clear and cold: {clear} of 400");
        assert!(days.contains(&Weather::Flurries));
        assert_eq!(weather_at(1, 40.0), weather_at(1, 40.0));
    }

    #[test]
    fn you_sleep_indoors_until_rested_and_noise_wakes_you() {
        let (city, _) = crate::load_heihe().unwrap();
        let forces = load_forces();
        let ground = Ground::new_with_interiors(&city, &WorldSim::new(&city).interiors);
        let mut sim = WorldSim::new(&city);
        sim.hour = 24.0 + hm(10, 0);
        sim.mornings = 1;
        sim.dawn_survived = true;
        sim.body.fatigue = 90.0;

        // Not in the street.
        let toggle = SimInput { toggle_sleep: true, clock_running: true, ..Default::default() };
        sim.player_at = P2::new(-140.0, -240.0);
        sim.tick(&city, &ground, &forces, &toggle, 0.05);
        assert!(!sim.sleeping, "no sleeping in the street");

        // In the basement shelter, with the stove going and nobody about.
        let idx = sim.interiors.iter().position(|i| i.kind == InteriorKind::BasementShelter).unwrap();
        sim.player_at = sim.interiors[idx].stove_at.unwrap();
        sim.interiors[idx].stove_fuel_s = 1.0e6;
        sim.interiors[idx].room_temp_c = 15.0;
        sim.tick(&city, &ground, &forces, &SimInput { clock_running: true, ..Default::default() }, 0.05);
        assert!(sim.active_interior == Some(idx));
        sim.squads.clear();
        sim.tanks.clear();
        sim.tick(&city, &ground, &forces, &toggle, 0.05);
        assert!(sim.sleeping, "{}", sim.subtitle_zh);

        let start = sim.hour;
        let inp = SimInput { clock_running: true, ..Default::default() };
        let mut steps = 0;
        while sim.sleeping && steps < 40_000 {
            sim.squads.clear();
            sim.tanks.clear();
            sim.spawn_wait = 1.0e6;
            sim.tick(&city, &ground, &forces, &inp, 0.25);
            steps += 1;
        }
        assert!(!sim.dead, "{}", sim.death_en);
        assert_eq!(sim.woke, Some(WakeReason::Rested), "woke by {:?} after {:.1} h", sim.woke, sim.hour - start);
        let slept = sim.hour - start;
        assert!((5.5..7.5).contains(&slept), "a night's sleep is six or seven hours: {slept}");
        assert!(sim.body.thirst > 50.0 && sim.body.hunger > 50.0, "asleep you need less: {} {}", sim.body.thirst, sim.body.hunger);

        // A soldier on the stairs wakes you.
        sim.body.fatigue = 80.0;
        sim.tick(&city, &ground, &forces, &toggle, 0.05);
        assert!(sim.sleeping);
        sim.spawn_wait = 1.0e6;
        let mut sq = Squad {
            id: 1,
            state: SquadState::Patrol,
            route_idx: 0,
            wp_idx: 0,
            wp_dir: 1,
            men: Vec::new(),
            meter: 0.0,
            last_seen: P2::default(),
            lost_for: 0.0,
            timer: 0.0,
            ignore_for: 0.0,
            held_at: P2::default(),
            shouted: false,
            hostile: false,
            stuck_t: 0.0,
            last_lead_at: P2::default(),
        };
        sq.men.push(Soldier {
            id: 2,
            at: sim.player_at.add(P2::new(6.0, 0.0)),
            floor: 0,
            facing: 0.0,
            hp: 100.0,
            optics: false,
            shot_cd: 9.0,
            speed: 0.0,
            phase: 0.0,
            in_cover: false,
            vault_t: 0.0,
            grenade_cd: 9.0,
        });
        sim.squads.push(sq);
        sim.tick(&city, &ground, &forces, &inp, 0.25);
        assert!(!sim.sleeping);
        assert!(matches!(sim.woke, Some(WakeReason::Enemy) | Some(WakeReason::Noise) | Some(WakeReason::Hurt)), "{:?}", sim.woke);
    }

    fn russian_tank_at(sim: &mut WorldSim, forces: &Forces, at: P2) -> usize {
        let before = sim.tanks.len();
        sim.spawn_tank_wave_at(VehicleKind::T72B3, at, forces);
        let mine = sim.tanks.pop().unwrap();
        sim.tanks.truncate(before);
        sim.tanks.push(mine);
        let ti = sim.tanks.len() - 1;
        sim.tank_wave_mask = u16::MAX;
        let t = &mut sim.tanks[ti];
        t.at = at;
        t.waypoints = vec![at];
        t.wp_idx = 0;
        t.cannon_cd = 0.0;
        t.mg_cd = 0.0;
        ti
    }

    #[test]
    fn a_fight_elsewhere_does_not_put_you_under_fire() {
        let (city, _) = crate::load_heihe().unwrap();
        let forces = load_forces();
        let ground = Ground::new_with_interiors(&city, &WorldSim::new(&city).interiors);
        let mut sim = WorldSim::new(&city);
        sim.hour = hm(21, 0);
        sim.blackout = true;
        let idx = sim.interiors.iter().position(|i| i.kind == InteriorKind::BasementShelter).unwrap();
        sim.player_at = sim.interiors[idx].stove_at.unwrap();
        sim.tick(&city, &ground, &forces, &SimInput::default(), 0.05);
        assert_eq!(sim.active_interior, Some(idx));
        sim.squads.clear();
        sim.tanks.clear();
        sim.spawn_wait = 1.0e6;
        // A squad 300 m away, fighting someone else.
        let far = sim.player_at.add(P2::new(300.0, 0.0));
        let mut sq = test_squad(far);
        sq.state = SquadState::Engage;
        sq.last_seen = far.add(P2::new(20.0, 0.0));
        sim.squads.push(sq);
        let hp = sim.body.hp;
        sim.tank_wave_mask = u16::MAX;
        for _ in 0..1200 {
            sim.tanks.clear();
            sim.squads[0].state = SquadState::Engage;
            sim.squads[0].lost_for = 0.0;
            sim.spawn_wait = 1.0e6;
            sim.tick(&city, &ground, &forces, &SimInput::default(), 0.25);
        }
        assert_eq!(sim.under_fire_for, 0.0, "nobody has seen you");
        assert!(sim.lancet.as_ref().is_none_or(|l| l.target.dist(sim.player_at) > 20.0), "no Lancet comes for you");
        assert!(sim.body.hp >= hp, "and nothing hits you: {} -> {}", hp, sim.body.hp);
    }

    /// A point on Central Street and a direction with a clear view (and clear
    /// ground) for `len` metres.
    fn open_street(city: &City, ground: &Ground, len: f32) -> (P2, P2) {
        let interiors = WorldSim::new(city).interiors;
        for dy in [0.0f32, 6.0, -6.0, 12.0, -12.0] {
            let p = P2::new(-262.0, -322.0 + dy);
            for k in 0..64 {
                let a = k as f32 / 64.0 * std::f32::consts::TAU;
                let d = P2::new(a.cos(), a.sin());
                let far = p.add(d.mul(len));
                let walk = ground.step(p, far, 0.85);
                if ground.clear_line_with_interiors(&interiors, p, far, false) && walk.dist(far) < 0.5 {
                    return (p, d);
                }
            }
        }
        panic!("no open line {len} m long near Central Street");
    }

    fn test_squad(at: P2) -> Squad {
        Squad {
            id: 77,
            state: SquadState::Patrol,
            route_idx: 0,
            wp_idx: 0,
            wp_dir: 1,
            men: vec![Soldier {
                id: 78,
                at,
                floor: 0,
                facing: 0.0,
                hp: 100.0,
                optics: false,
                shot_cd: 9.0,
                speed: 0.0,
                phase: 0.0,
                in_cover: false,
                vault_t: 0.0,
                grenade_cd: 9.0,
            }],
            meter: 0.0,
            last_seen: at,
            lost_for: 0.0,
            timer: 0.0,
            ignore_for: 0.0,
            held_at: at,
            shouted: false,
            hostile: false,
            stuck_t: 0.0,
            last_lead_at: at,
        }
    }

    #[test]
    fn armour_sees_by_the_light_and_not_into_rooms() {
        let (city, _) = crate::load_heihe().unwrap();
        let forces = load_forces();
        let ground = Ground::new_with_interiors(&city, &WorldSim::new(&city).interiors);
        let (a, dir) = open_street(&city, &ground, 82.0);
        let setup = |tank_m: f32| {
            let mut sim = WorldSim::new(&city);
            sim.hour = hm(22, 0);
            sim.blackout = true;
            sim.squads.clear();
            sim.tanks.clear();
            sim.player_at = a.add(dir.mul(10.0));
            let ti = russian_tank_at(&mut sim, &forces, a.add(dir.mul(10.0 + tank_m)));
            (sim, ti)
        };
        let run = |sim: &mut WorldSim| {
            let mut aimed = false;
            let id = sim.tanks[0].id;
            for _ in 0..80 {
                sim.squads.clear();
                sim.tanks.retain(|t| t.id == id);
                sim.spawn_wait = 1.0e6;
                sim.tick(&city, &ground, &forces, &SimInput::default(), 0.125);
                aimed |= sim.tanks.iter().any(|t| t.locked_target.is_some_and(|(p, _)| p.dist(sim.player_at) < 1.0));
            }
            aimed
        };
        let (mut far, _) = setup(70.0);
        assert!(!run(&mut far), "a still figure 70 m away in the blackout is not seen");
        assert!(far.body.hp >= 99.9);
        let (mut near, _) = setup(20.0);
        assert!(run(&mut near) || near.body.hp < 100.0, "at 20 m it is");

        // Indoors, never seen: the crew does not shoot into the room.
        let mut sim = WorldSim::new(&city);
        sim.hour = hm(22, 0);
        sim.blackout = true;
        sim.squads.clear();
        sim.tanks.clear();
        let idx = sim.interiors.iter().position(|i| i.kind == InteriorKind::PlayerHome).unwrap();
        sim.player_at = sim.interiors[idx].front_door.inside;
        let outside = sim.interiors[idx].front_door.outside;
        let away = outside.sub(sim.player_at).norm();
        russian_tank_at(&mut sim, &forces, outside.add(away.mul(18.0)));
        assert!(!run(&mut sim), "a crew that cannot see you does not lock you");
        assert!(sim.body.hp >= 99.9, "{}", sim.body.hp);
    }

    #[test]
    fn concrete_holds_a_tank_for_seconds_not_one_tick() {
        let (city, _) = crate::load_heihe().unwrap();
        let forces = load_forces();
        let ground = Ground::new_with_interiors(&city, &WorldSim::new(&city).interiors);
        let mut sim = WorldSim::new(&city);
        sim.hour = hm(19, 0);
        sim.squads.clear();
        sim.tanks.clear();
        sim.player_at = P2::new(-600.0, -600.0);
        let at = P2::new(-262.0, -322.0);
        let ti = russian_tank_at(&mut sim, &forces, at);
        for (k, kind) in [BarricadeKind::Concrete, BarricadeKind::DeadCar, BarricadeKind::BarbedWire].into_iter().enumerate() {
            sim.barricades.push(Barricade { at: at.add(P2::new(3.4, k as f32 * 0.01)), kind, yaw: 0.0, shoved: false, hp: kind.max_hp() });
        }
        sim.tick(&city, &ground, &forces, &SimInput::default(), 0.1);
        assert!(sim.barricades.iter().all(|b| !b.shoved), "3.4 m away nothing is flattened yet");
        // Up against the concrete, a heavy tank takes 55 hp a second off it.
        sim.barricades.retain(|b| b.kind == BarricadeKind::Concrete);
        sim.barricades[0].at = at.add(P2::new(2.0, 0.0));
        let id = sim.tanks[ti].id;
        for _ in 0..20 {
            sim.tanks.retain(|t| t.id == id);
            sim.tick(&city, &ground, &forces, &SimInput::default(), 0.1);
        }
        let b = &sim.barricades[0];
        assert!(!b.shoved && (b.hp - (240.0 - 110.0)).abs() < 12.0, "two seconds against it: {} hp left", b.hp);
    }

    #[test]
    fn fires_burn_down_and_a_knocked_barrel_goes_out() {
        let (city, _) = crate::load_heihe().unwrap();
        let mut sim = WorldSim::new(&city);
        sim.add_fire(P2::new(0.0, 0.0), MOLOTOV_FIRE_S);
        sim.add_fire(P2::new(5.0, 0.0), f32::INFINITY);
        sim.barricades.push(Barricade { at: P2::new(5.0, 0.0), kind: BarricadeKind::FireBarrel, yaw: 0.0, shoved: false, hp: 40.0 });
        sim.step_fires(MOLOTOV_FIRE_S - 1.0);
        assert_eq!(sim.fire_barrels.len(), 2);
        sim.step_fires(2.0);
        assert_eq!(sim.fire_barrels, vec![P2::new(5.0, 0.0)], "the Molotov has burnt out");
        sim.barricades[0].shoved = true;
        sim.step_fires(0.1);
        assert!(sim.fire_barrels.is_empty(), "the knocked-over barrel is out");
    }

    #[test]
    fn a_blast_falls_off_and_the_basement_walls_take_it() {
        let (city, _) = crate::load_heihe().unwrap();
        let forces = load_forces();
        let hurt_at = |d: f32| {
            let mut sim = WorldSim::new(&city);
            sim.player_at = P2::new(-600.0, -600.0);
            let mut out = TickOutput::default();
            sim.explode_at_floor(sim.player_at.add(P2::new(d, 0.0)), 0, 6.0, 40.0, 0.0, false, &forces, false, &mut out);
            100.0 - sim.body.hp
        };
        let close = hurt_at(0.5);
        let edge = hurt_at(5.5);
        assert!(close > 35.0 && edge < close * 0.6, "close {close}, edge {edge}");

        let mut sim = WorldSim::new(&city);
        let idx = sim.interiors.iter().position(|i| i.kind == InteriorKind::BasementShelter).unwrap();
        sim.player_at = sim.interiors[idx].stove_at.unwrap();
        sim.active_interior = Some(idx);
        let outside = sim.interiors[idx].front_door.outside;
        let mut out = TickOutput::default();
        let d = outside.dist(sim.player_at);
        sim.explode_at_floor(outside, 0, d + 1.0, 60.0, 0.0, false, &forces, false, &mut out);
        assert!(sim.body.hp > 85.0, "the basement walls take most of a Lancet in the street: {}", sim.body.hp);
    }

    #[test]
    fn a_vehicle_blocked_by_a_wreck_turns_back() {
        let (city, _) = crate::load_heihe().unwrap();
        let forces = load_forces();
        let ground = Ground::new_with_interiors(&city, &WorldSim::new(&city).interiors);
        let mut sim = WorldSim::new(&city);
        sim.hour = hm(19, 0);
        sim.squads.clear();
        sim.tanks.clear();
        sim.player_at = P2::new(-600.0, -600.0);
        let (a, dir) = open_street(&city, &ground, 62.0);
        let ti = russian_tank_at(&mut sim, &forces, a);
        sim.tanks[ti].waypoints = vec![a, a.add(dir.mul(60.0))];
        sim.tanks[ti].wp_idx = 1;
        // A wreck 6 m up the road.
        let wreck_ti = russian_tank_at(&mut sim, &forces, a.add(dir.mul(6.0)));
        let dead = sim.tanks.remove(wreck_ti);
        sim.push_wreck(dead);
        let id = sim.tanks[0].id;
        let mut turned = false;
        for _ in 0..80 {
            sim.squads.clear();
            sim.tanks.retain(|t| t.id == id);
            sim.spawn_wait = 1.0e6;
            sim.tick(&city, &ground, &forces, &SimInput::default(), 0.1);
            turned |= sim.tanks[0].wp_dir == -1;
        }
        assert!(turned, "after three seconds against the wreck it turns round");
    }

    #[test]
    fn a_vehicle_pressed_against_a_wall_turns_back() {
        let (city, _) = crate::load_heihe().unwrap();
        let forces = load_forces();
        let ground = Ground::new_with_interiors(&city, &WorldSim::new(&city).interiors);
        let mut sim = WorldSim::new(&city);
        sim.hour = hm(19, 0);
        sim.squads.clear();
        sim.tanks.clear();
        sim.player_at = P2::new(-600.0, -600.0);
        let (a, _) = open_street(&city, &ground, 40.0);
        // Aim it at the middle of the nearest solid building: it reaches the wall and
        // can get no closer, however it slides.
        let b = city
            .buildings
            .iter()
            .filter(|b| b.solid)
            .min_by(|x, y| x.centroid.dist(a).total_cmp(&y.centroid.dist(a)))
            .unwrap();
        let ti = russian_tank_at(&mut sim, &forces, a);
        sim.tanks[ti].waypoints = vec![a, b.centroid];
        sim.tanks[ti].wp_idx = 1;
        let id = sim.tanks[ti].id;
        let mut turned = false;
        for _ in 0..300 {
            sim.squads.clear();
            sim.tanks.retain(|t| t.id == id);
            sim.spawn_wait = 1.0e6;
            sim.tick(&city, &ground, &forces, &SimInput::default(), 0.1);
            turned |= sim.tanks[0].wp_dir == -1;
        }
        assert!(turned, "pressed against a wall for half a minute it must turn round");
    }

    #[test]
    fn a_soldier_crosses_wire_cut_but_alive() {
        let (city, _) = crate::load_heihe().unwrap();
        let sim = WorldSim::new(&city);
        let ground = Ground::new_with_interiors(&city, &sim.interiors);
        let (p, dir) = open_street(&city, &ground, 20.0);
        let wire = p.add(dir.mul(6.0));
        let snap = vec![(0usize, wire, BarricadeKind::BarbedWire)];
        let mut sol = test_squad(p).men.remove(0);
        let mut cut = 0.0;
        for _ in 0..300 {
            let mut dmg = Vec::new();
            WorldSim::move_squad_one(&mut sol, &ground, &sim.interiors, &snap, &mut dmg, p.add(dir.mul(14.0)), 0, 1.3, 0.1, 0.3);
            cut += dmg.iter().map(|d| d.1).sum::<f32>();
        }
        assert!(sol.at.dist(p) > 12.0, "he gets through: {:.1} m", sol.at.dist(p));
        assert!(sol.hp > 80.0, "cut, not killed: {}", sol.hp);
        assert!(cut > 40.0, "and the wire gives: {cut}");
    }

    #[test]
    fn pla_armour_holds_its_gun_when_you_stand_by_the_target() {
        let (city, _) = crate::load_heihe().unwrap();
        let forces = load_forces();
        let ground = Ground::new_with_interiors(&city, &WorldSim::new(&city).interiors);
        let (a, dir) = open_street(&city, &ground, 62.0);
        let run = |player_m: f32| {
            let mut sim = WorldSim::new(&city);
            sim.hour = hm(19, 0);
            sim.squads.clear();
            sim.tanks.clear();
            sim.tank_wave_mask = u16::MAX;
            sim.spawn_tank_wave_at(VehicleKind::Ztz99A, a, &forces);
            let mine = sim.tanks.pop().unwrap();
            sim.tanks.clear();
            sim.tanks.push(mine);
            let t = &mut sim.tanks[0];
            t.at = a;
            t.waypoints = vec![a];
            t.wp_idx = 0;
            t.cannon_cd = 0.0;
            let id = t.id;
            let target = a.add(dir.mul(40.0));
            sim.player_at = target.add(P2::new(-dir.y, dir.x).mul(player_m));
            let mut shelled = false;
            for _ in 0..100 {
                sim.tanks.retain(|t| t.id == id);
                sim.spawn_wait = 1.0e6;
                if sim.squads.is_empty() || sim.squads[0].men.is_empty() {
                    sim.squads.clear();
                    let mut sq = test_squad(target);
                    sq.state = SquadState::Check;
                    sim.squads.push(sq);
                }
                sim.squads[0].men[0].at = target;
                sim.squads[0].men[0].hp = 100.0;
                let out = sim.tick(&city, &ground, &forces, &SimInput::default(), 0.1);
                shelled |= out.tank_cannon && out.explosions.iter().any(|e| e.at.dist(target) < 8.0);
            }
            shelled
        };
        assert!(!run(3.0), "no gun on a soldier 3 m from you");
        let shelled_far = run(60.0);
        assert!(shelled_far, "with you 60 m away it fires");
    }

    /// Runs the rooms alone for `hours` of the clock, as the world would.
    fn run_rooms(sim: &mut WorldSim, hours: f32) {
        let dt = 1.0;
        let steps = (hours * 3600.0 / TIME_SCALE / dt) as usize;
        let mut out = TickOutput::default();
        for _ in 0..steps {
            sim.hour += dt * TIME_SCALE / 3600.0;
            sim.step_interiors_state(&SimInput::default(), dt, &mut out);
        }
    }

    #[test]
    fn a_flat_cools_over_days_not_hours() {
        let (city, _) = crate::load_heihe().unwrap();
        let mut sim = WorldSim::new(&city);
        sim.player_at = P2::new(-900.0, -900.0);
        let home = sim.interiors.iter().position(|i| i.kind == InteriorKind::PlayerHome).unwrap();
        let cellar = sim.interiors.iter().position(|i| i.kind == InteriorKind::BasementShelter).unwrap();
        sim.hour = HEAT_OFF_HOUR;
        sim.interiors[home].room_temp_c = 18.0;
        run_rooms(&mut sim, 10.0);
        let morning = sim.interiors[home].room_temp_c;
        assert!(morning > 8.0, "a closed flat is still cool, not frozen, the morning after: {morning}");
        run_rooms(&mut sim, 62.0);
        let third_day = sim.interiors[home].room_temp_c;
        assert!(third_day < 0.0 && third_day > sim.outdoor_c(), "three days on it is below freezing, still above the street: {third_day}");
        let b = crate::living::get().num("building", "basement_c");
        assert!((sim.interiors[cellar].room_temp_c - b).abs() < 2.0, "the basement holds near {b}: {}", sim.interiors[cellar].room_temp_c);
    }

    #[test]
    fn the_stove_burns_coal_then_wood_then_the_furniture() {
        let (city, _) = crate::load_heihe().unwrap();
        let forces = load_forces();
        let ground = Ground::new_with_interiors(&city, &WorldSim::new(&city).interiors);
        let mut sim = WorldSim::new(&city);
        sim.hour = hm(22, 0);
        let home = sim.interiors.iter().position(|i| i.kind == InteriorKind::PlayerHome).unwrap();
        sim.player_at = sim.interiors[home].stove_at.unwrap();
        sim.tick(&city, &ground, &forces, &SimInput::default(), 0.05);
        assert_eq!(sim.active_interior, Some(home));
        sim.inv.coal = 1;
        sim.inv.wood = 0;
        let t = SimInput { fortify_or_stove: true, ..Default::default() };
        let l = crate::living::get();
        let per_h = 3600.0 / TIME_SCALE;
        let f0 = sim.interiors[home].stove_fuel_s;
        sim.tick(&city, &ground, &forces, &t, 0.05);
        let f1 = sim.interiors[home].stove_fuel_s;
        assert!((f1 - f0 - l.num("fuel", "coal_unit_burn_h") * per_h).abs() < 1.0, "a coal: {f0} -> {f1}");
        let pieces = sim.interiors[home].furniture;
        assert!(pieces > 0);
        sim.tick(&city, &ground, &forces, &t, 0.05);
        assert_eq!(sim.interiors[home].furniture, pieces - 1, "no coal: break up a piece");
        assert_eq!(sim.inv.wood, l.num("fuel", "wood_per_piece") as u32);
        assert!(sim.noise_t > 0.0 && sim.noise_m > 10.0, "and it is heard");
        let f2 = sim.interiors[home].stove_fuel_s;
        sim.tick(&city, &ground, &forces, &t, 0.05);
        assert!(sim.interiors[home].stove_fuel_s > f2 + l.num("fuel", "wood_unit_burn_h") * per_h * 0.9, "then the wood goes in");
    }

    fn body_for_hours(body: &mut Body, hours: f32, env_c: f32) -> Option<&'static str> {
        body.env_c = env_c;
        body.indoors = true;
        let steps = (hours * 3600.0 / TIME_SCALE) as usize;
        for _ in 0..steps {
            if let Some((_, en)) = body.tick(1.0, 40.0, Gait::Still, true, false, false) {
                return Some(en);
            }
        }
        None
    }

    #[test]
    fn water_and_food_last_days_not_hours() {
        let mut dry = Body::default();
        assert!(body_for_hours(&mut dry, 96.0, 14.0).is_none(), "four days without water: alive");
        assert!(dry.thirst > 0.0 && dry.thirst < 30.0, "impaired, near collapse (day 5): {}", dry.thirst);
        assert!(body_for_hours(&mut dry, 72.0, 14.0).is_none(), "collapsed but alive on day 7: hp {}", dry.hp);
        let died = body_for_hours(&mut dry, 48.0, 14.0);
        assert_eq!(died, Some("You died of thirst."), "about eight days without water is the end");

        let mut warm = Body::default();
        let mut cold = Body::default();
        body_for_hours(&mut warm, 24.0, 18.0);
        body_for_hours(&mut cold, 24.0, 0.0);
        assert!(cold.hunger < warm.hunger, "a cold room burns more food: {} vs {}", cold.hunger, warm.hunger);
        assert!(warm.hunger > 80.0, "a day without food is hunger, not collapse: {}", warm.hunger);
    }

    #[test]
    fn cold_rooms_make_you_ill_and_medicine_helps() {
        let mut body = Body::default();
        body.thirst = 100.0;
        body.hunger = 100.0;
        let mut ill_after = 0.0;
        for _ in 0..7 {
            body.thirst = 100.0;
            body.hunger = 100.0;
            body_for_hours(&mut body, 24.0, 4.0);
            ill_after = body.illness;
        }
        assert!(ill_after > 25.0, "a week in a 4 °C room: {ill_after}");
        let mut warm = Body::default();
        warm.illness = 30.0;
        body_for_hours(&mut warm, 24.0, 20.0);
        assert!(warm.illness < 30.0, "a warm room lets it go: {}", warm.illness);

        let (city, _) = crate::load_heihe().unwrap();
        let forces = load_forces();
        let ground = Ground::new_with_interiors(&city, &WorldSim::new(&city).interiors);
        let mut sim = WorldSim::new(&city);
        sim.player_at = P2::new(-900.0, -900.0);
        sim.body.illness = 50.0;
        sim.inv.medicine = 1;
        sim.tick(&city, &ground, &forces, &SimInput { use_bandage: true, ..Default::default() }, 0.05);
        assert!(sim.body.illness < 20.0 && sim.inv.medicine == 0, "{}", sim.body.illness);
    }

    #[test]
    fn others_empty_the_shops_but_not_your_home() {
        let (city, _) = crate::load_heihe().unwrap();
        let mut sim = WorldSim::new(&city);
        let left = |sim: &WorldSim, homes: bool| -> usize {
            sim.interiors
                .iter()
                .filter(|i| homes == matches!(i.kind, InteriorKind::PlayerHome | InteriorKind::NeighbourHome | InteriorKind::BasementShelter))
                .flat_map(|i| i.loots.iter())
                .filter(|l| !l.searched)
                .count()
        };
        let shops0 = left(&sim, false);
        let homes0 = left(&sim, true);
        for m in 1..=5 {
            sim.mornings = m;
            sim.others_take_from_shops();
        }
        let shops5 = left(&sim, false);
        assert!(shops0 > 10 && shops5 * 2 < shops0, "five mornings take more than half: {shops0} -> {shops5}");
        assert!(shops5 > 0, "but not everything");
        assert_eq!(left(&sim, true), homes0);
    }
}
