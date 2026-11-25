use bevy::{
    app::AppExit,
    picking::prelude::{Pickable, PointerButton, PointerClick, PointerOut, PointerOver},
    prelude::*,
};
use sim::{
    city::City,
    figure::AnimState,
    forces::Forces,
    geom::P2,
    survival::{hm, Barricade, BarricadeKind, PrimaryWeapon, VehicleKind, WorldSim},
    walk::Ground,
};

use crate::{
    audio::AudioState,
    map_ui::TacticalMap,
    play::{AtlasKey, FigureAtlas, PlayState, SootMark},
    Clock, Player,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReturnScreen {
    Start,
    Paused,
}

#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuScreen {
    Start,
    Playing,
    Paused,
    Controls { return_to: ReturnScreen },
}

#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct StartScenario(pub usize);

const SCENARIO_NAMES: [&str; 4] = [
    "16:00 战前准备（默认开局）",
    "18:10 中央街夜战（已持枪与民兵）",
    "21:05 停电江雾求生（高危夜战）",
    "00:15 解放军重装反击（99A vs T-90M 全装巷战）",
];

#[derive(Clone, Copy, Debug)]
enum MenuAction {
    StartGame,
    CycleScenario,
    OpenControls,
    CloseControls,
    ResumeGame,
    RestartGame,
    ToggleMute,
    ReturnToStart,
    QuitGame,
}

#[derive(Resource, Default)]
struct MenuActionQueue(Vec<MenuAction>);

#[derive(Component)]
struct StartMenuRoot;

#[derive(Component)]
struct PauseMenuRoot;

#[derive(Component)]
struct ControlsMenuRoot;

#[derive(Component)]
struct ScenarioLabel;

#[derive(Component)]
struct MuteLabel;

#[derive(Component)]
struct PauseClockLabel;

#[derive(Clone, Copy)]
struct BtnPalette {
    normal_bg: Color,
    hover_bg: Color,
    normal_border: Color,
    hover_border: Color,
}

const BTN_PRIMARY: BtnPalette = BtnPalette {
    normal_bg: Color::srgba(0.62, 0.17, 0.14, 0.94),
    hover_bg: Color::srgba(0.76, 0.22, 0.18, 0.98),
    normal_border: Color::srgba(0.95, 0.58, 0.45, 0.72),
    hover_border: Color::srgba(1.0, 0.80, 0.62, 0.95),
};

const BTN_SECONDARY: BtnPalette = BtnPalette {
    normal_bg: Color::srgba(0.09, 0.13, 0.20, 0.90),
    hover_bg: Color::srgba(0.16, 0.23, 0.34, 0.96),
    normal_border: Color::srgba(0.50, 0.66, 0.82, 0.34),
    hover_border: Color::srgba(0.84, 0.92, 1.0, 0.78),
};

const BTN_SUBTLE_DANGER: BtnPalette = BtnPalette {
    normal_bg: Color::srgba(0.11, 0.10, 0.14, 0.86),
    hover_bg: Color::srgba(0.32, 0.12, 0.13, 0.94),
    normal_border: Color::srgba(0.62, 0.40, 0.42, 0.36),
    hover_border: Color::srgba(0.95, 0.56, 0.52, 0.82),
};

pub fn install(app: &mut App, enable: bool, start_open: bool) {
    let initial = if enable && start_open {
        MenuScreen::Start
    } else {
        MenuScreen::Playing
    };
    app.insert_resource(initial)
        .init_resource::<StartScenario>()
        .init_resource::<MenuActionQueue>();

    if enable {
        app.add_systems(Startup, setup_menus.after(crate::play::setup_play))
            .add_systems(Update, drive_menus.before(crate::play::step_play));
    }
}

pub fn apply_scenario(
    scenario: usize,
    city: &City,
    _ground: &Ground,
    forces: &Forces,
    sim: &mut WorldSim,
    clock: &mut Clock,
) {
    *sim = WorldSim::new(city);
    match scenario % SCENARIO_NAMES.len() {
        1 => {
            sim.hour = hm(18, 10);
            sim.alert_stage = 2;
            sim.body.coat_bonus = 0.35;
            sim.inv.unlock_primary(PrimaryWeapon::Qbz191);
            sim.inv.unlock_primary(PrimaryWeapon::Saiga12);
            sim.inv.unlock_primary(PrimaryWeapon::EwJammer);
            sim.inv.ammo = 28;
            sim.inv.rpg = 2;
            sim.inv.pf98 = 1;
            sim.inv.molotov = 3;
            sim.inv.grenades = 2;
            sim.inv.smoke_grenades = 2;
            sim.inv.bandage = 2;
            sim.inv.scrap = 12;
            sim.inv.coal = 6;
            sim.rally_militia();
            sim.spawn_tank_wave_at(VehicleKind::T72B3, P2::new(-248.0, -260.0), forces);
        }
        2 => {
            sim.hour = hm(21, 5);
            sim.alert_stage = 2;
            sim.blackout = true;
            sim.body.coat_bonus = 0.35;
            sim.inv.unlock_primary(PrimaryWeapon::Qbz191);
            sim.inv.unlock_primary(PrimaryWeapon::Saiga12);
            sim.inv.unlock_primary(PrimaryWeapon::EwJammer);
            sim.inv.ammo = 24;
            sim.inv.rpg = 2;
            sim.inv.has_radio = true;
            sim.inv.radio_strikes = 2;
            sim.inv.smoke_grenades = 2;
            sim.inv.bandage = 2;
            sim.inv.warmer = 2;
            sim.inv.scrap = 10;
            sim.inv.coal = 8;
            sim.rally_militia();
        }
        3 => {
            sim.hour = hm(24, 15);
            sim.alert_stage = 2;
            sim.blackout = true;
            sim.body.coat_bonus = 0.45;
            sim.inv.unlock_all_arsenal();
            sim.inv.radio_strikes = 3;
            sim.inv.bandage = 4;
            sim.inv.food = 4;
            sim.inv.water = 4;
            sim.inv.scrap = 16;
            sim.inv.coal = 8;
            sim.rally_militia();
            for kind in [
                VehicleKind::Ztz99A,
                VehicleKind::Ztz96B,
                VehicleKind::Ztq15,
                VehicleKind::Zbd04A,
                VehicleKind::Zbl08,
                VehicleKind::Pgz09,
                VehicleKind::Csk181,
            ] {
                sim.spawn_tank_wave_at(kind, P2::new(-272.0, -355.0), forces);
            }
            for kind in [
                VehicleKind::T90M,
                VehicleKind::T80Bvm,
                VehicleKind::T72B3,
                VehicleKind::Tos1A,
                VehicleKind::Bmpt,
                VehicleKind::Bmp3,
                VehicleKind::Btr82A,
            ] {
                sim.spawn_tank_wave_at(kind, P2::new(-244.0, -215.0), forces);
            }
            let c = sim.player_at;
            let yaw = 0.35;
            for (dx, dy, kind) in [
                (-3.5, 4.5, BarricadeKind::Trench),
                (3.5, 5.2, BarricadeKind::Trench),
                (-1.0, 2.5, BarricadeKind::TarpDugout),
                (-4.2, 8.2, BarricadeKind::BarbedWire),
                (4.0, 8.8, BarricadeKind::BarbedWire),
            ] {
                let p = _ground.step(c.add(P2::new(dx, dy)), c.add(P2::new(dx, dy)), 0.4);
                sim.barricades.push(Barricade {
                    at: p,
                    kind,
                    yaw,
                    shoved: false,
                    hp: kind.max_hp(),
                });
            }
        }
        _ => {}
    }
    clock.hour = sim.hour;
    clock.speed = 1.0;
    clock.blackout = sim.blackout;
    clock.fog = sim.fog_density;
}

fn spawn_menu_btn<M: Component>(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    label: &str,
    palette: BtnPalette,
    action: MenuAction,
    centered: bool,
    marker: Option<M>,
) {
    let mut ec = parent.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(44.0),
            padding: UiRect::horizontal(Val::Px(16.0)),
            border: UiRect::all(Val::Px(1.5)),
            align_items: AlignItems::Center,
            justify_content: if centered {
                JustifyContent::Center
            } else {
                JustifyContent::FlexStart
            },
            ..default()
        },
        BackgroundColor(palette.normal_bg),
        BorderColor::all(palette.normal_border),
    ));

    ec.observe(
        move |ev: On<PointerOver>, mut q: Query<(&mut BackgroundColor, &mut BorderColor)>| {
            if let Ok((mut bg, mut bd)) = q.get_mut(ev.entity) {
                bg.0 = palette.hover_bg;
                *bd = BorderColor::all(palette.hover_border);
            }
        },
    )
    .observe(
        move |ev: On<PointerOut>, mut q: Query<(&mut BackgroundColor, &mut BorderColor)>| {
            if let Ok((mut bg, mut bd)) = q.get_mut(ev.entity) {
                bg.0 = palette.normal_bg;
                *bd = BorderColor::all(palette.normal_border);
            }
        },
    )
    .observe(
        move |ev: On<PointerClick>, mut queue: ResMut<MenuActionQueue>| {
            if ev.button == PointerButton::Primary {
                queue.0.push(action);
            }
        },
    );

    ec.with_children(|b| {
        let mut txt = b.spawn((
            Text::new(label),
            TextFont {
                font: font.clone().into(),
                font_size: bevy::text::FontSize::Px(17.0),
                ..default()
            },
            TextColor(Color::WHITE),
            Pickable::IGNORE,
        ));
        if let Some(m) = marker {
            txt.insert(m);
        }
    });
}

#[derive(Component)]
struct NoMarker;

fn spawn_column_section(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    heading: &str,
    items: &[(&str, &str)],
) {
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(7.0),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|sec| {
            sec.spawn((
                Text::new(heading),
                TextFont {
                    font: font.clone().into(),
                    font_size: bevy::text::FontSize::Px(17.0),
                    ..default()
                },
                TextColor(Color::srgb(0.96, 0.76, 0.40)),
                Pickable::IGNORE,
            ));
            for (key, desc) in items {
                sec.spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(10.0),
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|row| {
                    row.spawn((
                        Node {
                            width: Val::Px(126.0),
                            padding: UiRect::axes(Val::Px(8.0), Val::Px(3.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.11, 0.16, 0.24, 0.95)),
                        BorderColor::all(Color::srgba(0.55, 0.72, 0.88, 0.40)),
                        Pickable::IGNORE,
                    ))
                    .with_children(|k| {
                        k.spawn((
                            Text::new(*key),
                            TextFont {
                                font: font.clone().into(),
                                font_size: bevy::text::FontSize::Px(13.5),
                                ..default()
                            },
                            TextColor(Color::srgb(0.95, 0.97, 1.0)),
                            Pickable::IGNORE,
                        ));
                    });
                    row.spawn((
                        Text::new(*desc),
                        TextFont {
                            font: font.clone().into(),
                            font_size: bevy::text::FontSize::Px(14.5),
                            ..default()
                        },
                        TextColor(Color::srgb(0.84, 0.89, 0.94)),
                        Pickable::IGNORE,
                    ));
                });
            }
        });
}

fn setup_menus(
    mut commands: Commands,
    assets: Res<AssetServer>,
    screen: Res<MenuScreen>,
    scenario: Res<StartScenario>,
) {
    let font: Handle<Font> = assets.load("fonts/NotoSansSC-VF.ttf");
    let start_vis = if *screen == MenuScreen::Start {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };

    // 1. Start Menu (left-anchored dark winter panel over live Central Street scene).
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::FlexStart,
                padding: UiRect::left(Val::Percent(6.5)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.015, 0.025, 0.045, 0.50)),
            GlobalZIndex(20),
            start_vis,
            StartMenuRoot,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    width: Val::Px(470.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(26.0)),
                    row_gap: Val::Px(12.0),
                    border: UiRect::all(Val::Px(1.5)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.04, 0.06, 0.10, 0.93)),
                BorderColor::all(Color::srgba(0.56, 0.72, 0.88, 0.36)),
            ))
            .with_children(|card| {
                card.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(3.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.82, 0.22, 0.18)),
                    Pickable::IGNORE,
                ));
                card.spawn((
                    Text::new("2031 · 01 · 17   |   黑龙江省黑河市 · 中央街"),
                    TextFont {
                        font: font.clone().into(),
                        font_size: bevy::text::FontSize::Px(13.5),
                        ..default()
                    },
                    TextColor(Color::srgb(0.68, 0.78, 0.88)),
                    Pickable::IGNORE,
                ));
                card.spawn((
                    Text::new("东北：俄罗斯入侵"),
                    TextFont {
                        font: font.clone().into(),
                        font_size: bevy::text::FontSize::Px(36.0),
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Pickable::IGNORE,
                ));
                card.spawn((
                    Text::new("黑河  ·  一月的严寒里，在占领下活下去"),
                    TextFont {
                        font: font.clone().into(),
                        font_size: bevy::text::FontSize::Px(17.5),
                        ..default()
                    },
                    TextColor(Color::srgb(0.95, 0.75, 0.38)),
                    Pickable::IGNORE,
                ));
                card.spawn((
                    Text::new(
                        "你不是军人，只是刚从店里下班、离家几条街的普通市民。\n17:36 俄军将跨过冰封的黑龙江，19:30 全城停电，21:00 停止供暖。\n备好棉衣饮食与绷带撑到天亮 07:00，或集结民兵修筑工事抗击入侵。",
                    ),
                    TextFont {
                        font: font.clone().into(),
                        font_size: bevy::text::FontSize::Px(14.5),
                        ..default()
                    },
                    TextColor(Color::srgb(0.80, 0.86, 0.92)),
                    Node {
                        margin: UiRect::vertical(Val::Px(4.0)),
                        ..default()
                    },
                    Pickable::IGNORE,
                ));

                spawn_menu_btn::<NoMarker>(card, &font, "开始生存", BTN_PRIMARY, MenuAction::StartGame, false, None);
                spawn_menu_btn(
                    card,
                    &font,
                    &format!("开局时段：{}", SCENARIO_NAMES[scenario.0 % SCENARIO_NAMES.len()]),
                    BTN_SECONDARY,
                    MenuAction::CycleScenario,
                    false,
                    Some(ScenarioLabel),
                );
                spawn_menu_btn::<NoMarker>(card, &font, "操作与按键说明", BTN_SECONDARY, MenuAction::OpenControls, false, None);
                spawn_menu_btn(card, &font, "音效：开启", BTN_SECONDARY, MenuAction::ToggleMute, false, Some(MuteLabel));
                spawn_menu_btn::<NoMarker>(card, &font, "退出游戏", BTN_SUBTLE_DANGER, MenuAction::QuitGame, false, None);

                card.spawn((
                    Text::new("提示：按 Enter 直接进入黑河 · 游戏中按 Esc 随时呼出暂停与操作菜单"),
                    TextFont {
                        font: font.clone().into(),
                        font_size: bevy::text::FontSize::Px(12.5),
                        ..default()
                    },
                    TextColor(Color::srgb(0.58, 0.67, 0.78)),
                    Node {
                        margin: UiRect::top(Val::Px(4.0)),
                        ..default()
                    },
                    Pickable::IGNORE,
                ));
            });
        });

    // 2. Controls & Survival Guide Menu (centered 2-column card).
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.015, 0.02, 0.04, 0.74)),
            GlobalZIndex(30),
            Visibility::Hidden,
            ControlsMenuRoot,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    width: Val::Px(860.0),
                    max_width: Val::Percent(92.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(26.0)),
                    row_gap: Val::Px(16.0),
                    border: UiRect::all(Val::Px(1.5)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.04, 0.06, 0.10, 0.95)),
                BorderColor::all(Color::srgba(0.56, 0.72, 0.88, 0.42)),
            ))
            .with_children(|card| {
                card.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(3.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.82, 0.22, 0.18)),
                    Pickable::IGNORE,
                ));
                card.spawn((
                    Text::new("操作与按键说明"),
                    TextFont {
                        font: font.clone().into(),
                        font_size: bevy::text::FontSize::Px(28.0),
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Pickable::IGNORE,
                ));
                card.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        column_gap: Val::Px(28.0),
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|cols| {
                    cols.spawn((
                        Node {
                            width: Val::Percent(48.0),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(16.0),
                            ..default()
                        },
                        Pickable::IGNORE,
                    ))
                    .with_children(|left| {
                        spawn_column_section(
                            left,
                            &font,
                            "【基础移动与视角】",
                            &[
                                ("WASD / 方向键", "在黑河街巷中行走移动"),
                                ("Shift + 移动", "加速奔跑（声响与暴露度大）"),
                                ("鼠标滚轮", "拉近或拉远俯视视角"),
                                ("Tab / +- / []", "切换时间流速（1x~16x），也可点左上角按钮"),
                            ],
                        );
                        spawn_column_section(
                            left,
                            &font,
                            "【生存、搜刮与室内】",
                            &[
                                ("E", "推门、搜柜、救邻；在家里柜子旁藏起或取出武器和电台"),
                                ("J", "在屋里躺下睡觉，时间快转；有动静会醒，再按 J 起来"),
                                ("Space / Ctrl", "在室内楼梯口上楼 / 下楼"),
                                ("Q", "用绷带止血；没在流血又生病时吃药"),
                                ("Z  /  X", "食用干粮 / 饮水补充体力"),
                                ("C", "捏热暖宝宝快速回暖抗寒"),
                                ("L  /  K", "开关手电筒 / 拉开或合上室内窗帘"),
                            ],
                        );
                    });

                    cols.spawn((
                        Node {
                            width: Val::Percent(48.0),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(16.0),
                            ..default()
                        },
                        Pickable::IGNORE,
                    ))
                    .with_children(|right| {
                        spawn_column_section(
                            right,
                            &font,
                            "【抵抗与作战】",
                            &[
                                ("H", "警报后集结本地市民组成民兵"),
                                ("鼠标左键", "用当前主武器向窗外/街面开火射击"),
                                ("N", "切换主武器（QBZ-191/QBU-191/PKM/Saiga-12/干扰枪）"),
                                ("U  /  F", "切换重武器 / 发射 HJ-12·PF-98·RPG-7·温压筒"),
                                ("Y  /  O  /  I", "投掷定时手雷 / 释放战术烟雾弹 / 放飞FPV"),
                                ("V", "投掷燃烧瓶（高楼俯掷坦克 2× 伤）"),
                                ("R", "呼叫 PLA 155mm 榴弹炮急射 + 装甲增援"),
                            ],
                        );
                        spawn_column_section(
                            right,
                            &font,
                            "【工事、暂停与地图】",
                            &[
                                ("1 – 8 / B", "选沙袋/野战战壕/伪装布掩体/刺猬拒马按 B 建造"),
                                ("P / Space", "战术暂停（自由 WASD 移镜 + 右键指派民兵）"),
                                ("T", "顶门 / 封窗 / 添煤或柴；没燃料时砸家具当柴"),
                                ("G / 右键", "切随行或指派民兵进屋架窗驻防（支持M地图与E急救）"),
                                ("M / 滚轮 / C", "居中开关战术地图（+-/滚轮缩放 · C居中自己 · WASD平移）"),
                            ],
                        );
                    });
                });

                card.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        padding: UiRect::axes(Val::Px(14.0), Val::Px(10.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.08, 0.12, 0.18, 0.88)),
                    BorderColor::all(Color::srgba(0.95, 0.75, 0.38, 0.30)),
                    Pickable::IGNORE,
                ))
                .with_children(|tip| {
                    tip.spawn((
                        Text::new(
                            "战时提示：推开临街蓝绿棉门帘进屋可前后穿堂脱身；黑灯拉窗帘可藏身躲避街面俄军与无人机，后屋里间与北街人防地下室能大幅削弱迫击炮与 FPV 威胁；21:00 停暖后请及时回家或躲入室内封窗生煤炉（T）。",
                        ),
                        TextFont {
                            font: font.clone().into(),
                            font_size: bevy::text::FontSize::Px(13.5),
                            ..default()
                        },
                        TextColor(Color::srgb(0.92, 0.86, 0.70)),
                        Pickable::IGNORE,
                    ));
                });

                cols_footer_btn(card, &font);
            });
        });

    // 3. In-game Pause Menu (centered compact card).
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.015, 0.02, 0.04, 0.66)),
            GlobalZIndex(25),
            Visibility::Hidden,
            PauseMenuRoot,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    width: Val::Px(390.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(24.0)),
                    row_gap: Val::Px(10.0),
                    border: UiRect::all(Val::Px(1.5)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.04, 0.06, 0.10, 0.94)),
                BorderColor::all(Color::srgba(0.56, 0.72, 0.88, 0.40)),
            ))
            .with_children(|card| {
                card.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(3.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.82, 0.22, 0.18)),
                    Pickable::IGNORE,
                ));
                card.spawn((
                    Text::new("游戏已暂停"),
                    TextFont {
                        font: font.clone().into(),
                        font_size: bevy::text::FontSize::Px(28.0),
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Pickable::IGNORE,
                ));
                card.spawn((
                    Text::new("2031年1月17日 16:00  ·  黑河"),
                    TextFont {
                        font: font.clone().into(),
                        font_size: bevy::text::FontSize::Px(14.5),
                        ..default()
                    },
                    TextColor(Color::srgb(0.75, 0.84, 0.94)),
                    Node {
                        margin: UiRect::bottom(Val::Px(4.0)),
                        ..default()
                    },
                    Pickable::IGNORE,
                    PauseClockLabel,
                ));

                spawn_menu_btn::<NoMarker>(card, &font, "继续游戏  [Esc]", BTN_PRIMARY, MenuAction::ResumeGame, false, None);
                spawn_menu_btn::<NoMarker>(card, &font, "操作与按键说明", BTN_SECONDARY, MenuAction::OpenControls, false, None);
                spawn_menu_btn(card, &font, "音效：开启", BTN_SECONDARY, MenuAction::ToggleMute, false, Some(MuteLabel));
                spawn_menu_btn::<NoMarker>(card, &font, "重新开始本局", BTN_SECONDARY, MenuAction::RestartGame, false, None);
                spawn_menu_btn::<NoMarker>(card, &font, "返回开始菜单", BTN_SECONDARY, MenuAction::ReturnToStart, false, None);
                spawn_menu_btn::<NoMarker>(card, &font, "退出游戏", BTN_SUBTLE_DANGER, MenuAction::QuitGame, false, None);
            });
        });
}

fn cols_footer_btn(card: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    card.spawn((
        Node {
            width: Val::Px(240.0),
            align_self: AlignSelf::Center,
            ..default()
        },
        Pickable::IGNORE,
    ))
    .with_children(|wrap| {
        spawn_menu_btn::<NoMarker>(wrap, font, "返回菜单  [Esc]", BTN_PRIMARY, MenuAction::CloseControls, true, None);
    });
}

fn reset_world_and_player(
    commands: &mut Commands,
    scenario: usize,
    heihe: &crate::Heihe,
    play: &mut PlayState,
    clock: &mut Clock,
    atlas: &FigureAtlas,
    player: &mut Single<(&mut Player, &mut Transform, &mut Mesh3d)>,
    soot_q: &Query<Entity, With<SootMark>>,
) {
    apply_scenario(scenario, &heihe.city, &heihe.ground, &play.forces, &mut play.sim, clock);
    play.paused = false;
    play.preview_barricade = false;
    play.cursor_world = None;
    for e in soot_q {
        commands.entity(e).despawn();
    }
    let (who, tf, pmesh) = &mut **player;
    who.at = play.sim.player_at;
    tf.translation = Vec3::new(who.at.x, 0.0, -who.at.y);
    tf.rotation = Quat::from_rotation_y(play.sim.player_facing + std::f32::consts::FRAC_PI_2);
    let variant = u8::from(play.sim.body.coat_bonus > 0.0) | (u8::from(play.sim.inv.has_rifle) << 1);
    if let Some(h) = atlas.meshes.get(&AtlasKey {
        role: sim::figure::Role::Civilian,
        variant,
        anim: AnimState::Idle,
        frame: 0,
    }) {
        pmesh.0 = h.clone();
    }
}

#[allow(clippy::too_many_arguments)]
fn drive_menus(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut screen: ResMut<MenuScreen>,
    mut scenario: ResMut<StartScenario>,
    mut queue: ResMut<MenuActionQueue>,
    mut play: Option<ResMut<PlayState>>,
    mut map: Option<ResMut<TacticalMap>>,
    (heihe, atlas, mut clock, mut audio_state): (
        Res<crate::Heihe>,
        Option<Res<FigureAtlas>>,
        ResMut<Clock>,
        ResMut<AudioState>,
    ),
    mut player: Single<(&mut Player, &mut Transform, &mut Mesh3d)>,
    soot_q: Query<Entity, With<SootMark>>,
    mut exit: MessageWriter<AppExit>,
    mut roots: (
        Query<&mut Visibility, (With<StartMenuRoot>, Without<PauseMenuRoot>, Without<ControlsMenuRoot>)>,
        Query<&mut Visibility, (With<PauseMenuRoot>, Without<StartMenuRoot>, Without<ControlsMenuRoot>)>,
        Query<&mut Visibility, (With<ControlsMenuRoot>, Without<StartMenuRoot>, Without<PauseMenuRoot>)>,
    ),
    mut labels: (
        Query<&mut Text, (With<ScenarioLabel>, Without<MuteLabel>, Without<PauseClockLabel>)>,
        Query<&mut Text, (With<MuteLabel>, Without<ScenarioLabel>, Without<PauseClockLabel>)>,
        Query<&mut Text, (With<PauseClockLabel>, Without<ScenarioLabel>, Without<MuteLabel>)>,
    ),
) {
    // Handle keyboard shortcuts alongside mouse clicks.
    if keys.just_pressed(KeyCode::Enter) && *screen == MenuScreen::Start {
        queue.0.push(MenuAction::StartGame);
    }
    if keys.just_pressed(KeyCode::Escape) {
        match *screen {
            MenuScreen::Start => {}
            MenuScreen::Controls { .. } => queue.0.push(MenuAction::CloseControls),
            MenuScreen::Paused => queue.0.push(MenuAction::ResumeGame),
            MenuScreen::Playing => {
                if let Some(p) = play.as_deref_mut() {
                    if p.preview_barricade {
                        p.preview_barricade = false;
                    } else if map.as_deref().is_some_and(|m| m.open) {
                        if let Some(m) = map.as_deref_mut() {
                            m.open = false;
                        }
                    } else if p.sim.dead {
                        queue.0.push(MenuAction::ReturnToStart);
                    } else {
                        *screen = MenuScreen::Paused;
                    }
                }
            }
        }
    }

    let actions: Vec<MenuAction> = queue.0.drain(..).collect();
    for action in actions {
        match action {
            MenuAction::StartGame => {
                if let (Some(p), Some(atl)) = (play.as_deref_mut(), atlas.as_deref()) {
                    reset_world_and_player(&mut commands, scenario.0, &heihe, p, &mut clock, atl, &mut player, &soot_q);
                    p.started = true;
                    p.click_cooldown = 0.24;
                }
                if let Some(m) = map.as_deref_mut() {
                    m.open = false;
                }
                *screen = MenuScreen::Playing;
            }
            MenuAction::CycleScenario => {
                scenario.0 = (scenario.0 + 1) % SCENARIO_NAMES.len();
                if let (Some(p), Some(atl)) = (play.as_deref_mut(), atlas.as_deref()) {
                    reset_world_and_player(&mut commands, scenario.0, &heihe, p, &mut clock, atl, &mut player, &soot_q);
                }
            }
            MenuAction::OpenControls => {
                let return_to = match *screen {
                    MenuScreen::Paused => ReturnScreen::Paused,
                    _ => ReturnScreen::Start,
                };
                *screen = MenuScreen::Controls { return_to };
            }
            MenuAction::CloseControls => {
                if let MenuScreen::Controls { return_to } = *screen {
                    *screen = match return_to {
                        ReturnScreen::Start => MenuScreen::Start,
                        ReturnScreen::Paused => MenuScreen::Paused,
                    };
                }
            }
            MenuAction::ResumeGame => {
                if let Some(p) = play.as_deref_mut() {
                    p.click_cooldown = 0.22;
                }
                *screen = MenuScreen::Playing;
            }
            MenuAction::RestartGame => {
                if let (Some(p), Some(atl)) = (play.as_deref_mut(), atlas.as_deref()) {
                    reset_world_and_player(&mut commands, scenario.0, &heihe, p, &mut clock, atl, &mut player, &soot_q);
                    p.started = true;
                    p.click_cooldown = 0.24;
                }
                if let Some(m) = map.as_deref_mut() {
                    m.open = false;
                }
                *screen = MenuScreen::Playing;
            }
            MenuAction::ToggleMute => {
                audio_state.muted = !audio_state.muted;
            }
            MenuAction::ReturnToStart => {
                if let (Some(p), Some(atl)) = (play.as_deref_mut(), atlas.as_deref()) {
                    reset_world_and_player(&mut commands, scenario.0, &heihe, p, &mut clock, atl, &mut player, &soot_q);
                    p.started = false;
                }
                if let Some(m) = map.as_deref_mut() {
                    m.open = false;
                }
                *screen = MenuScreen::Start;
            }
            MenuAction::QuitGame => {
                exit.write(AppExit::Success);
            }
        }
    }

    audio_state.menu_open = *screen != MenuScreen::Playing;

    for mut v in &mut roots.0 {
        *v = if *screen == MenuScreen::Start { Visibility::Inherited } else { Visibility::Hidden };
    }
    for mut v in &mut roots.1 {
        *v = if *screen == MenuScreen::Paused { Visibility::Inherited } else { Visibility::Hidden };
    }
    for mut v in &mut roots.2 {
        *v = if matches!(*screen, MenuScreen::Controls { .. }) {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }

    let scen_text = format!("开局时段：{}", SCENARIO_NAMES[scenario.0 % SCENARIO_NAMES.len()]);
    for mut t in &mut labels.0 {
        if t.0 != scen_text {
            t.0 = scen_text.clone();
        }
    }
    let mute_text = if audio_state.muted { "音效：静音" } else { "音效：开启" };
    for mut t in &mut labels.1 {
        if t.0 != mute_text {
            t.0 = mute_text.into();
        }
    }
    if *screen == MenuScreen::Paused {
        let clk = format!("2031年{}  ·  黑河", sim::clock::label(clock.hour));
        for mut t in &mut labels.2 {
            if t.0 != clk {
                t.0 = clk.clone();
            }
        }
    }
}
