//! The light of 17 January at 50.25° north. The sun never climbs above
//! nineteen degrees, it is down before four in the afternoon, and after that
//! the street belongs to sodium lamps and lit windows.

use crate::Clock;
use bevy::prelude::*;

#[derive(Component)]
pub struct Sun;

#[derive(Component)]
pub struct StreetLamp;

#[derive(Resource)]
pub struct GlowMaterials {
    pub windows: Handle<StandardMaterial>,
    pub lamps: Handle<StandardMaterial>,
}

pub const SUNRISE: f32 = 7.55;
pub const SUNSET: f32 = 15.85;
const NOON_ELEVATION: f32 = 18.8;

/// Elevation and azimuth of the sun in degrees. Azimuth is clockwise from north.
pub fn sun(hour: f32) -> (f32, f32) {
    let t = (hour - SUNRISE) / (SUNSET - SUNRISE);
    let elevation = if (0.0..=1.0).contains(&t) {
        NOON_ELEVATION * (t * std::f32::consts::PI).sin()
    } else {
        // Below the horizon: how far, for the length of dusk.
        let away = if hour > SUNSET { hour - SUNSET } else { SUNRISE - hour };
        -(away * 9.0).min(40.0)
    };
    let azimuth = 125.0 + 110.0 * t.clamp(-0.3, 1.3);
    (elevation, azimuth)
}

/// 0 in full day, 1 in full night.
pub fn night(hour: f32) -> f32 {
    let (elevation, _) = sun(hour);
    ((2.0 - elevation) / 9.0).clamp(0.0, 1.0)
}

pub fn setup(mut commands: Commands) {
    commands.spawn((
        DirectionalLight { shadow_maps_enabled: true, illuminance: 20_000.0, ..default() },
        bevy::light::CascadeShadowConfigBuilder { num_cascades: 3, first_cascade_far_bound: 180.0, maximum_distance: 520.0, ..default() }.build(),
        Transform::default(),
        Sun,
    ));
}

pub fn apply(
    clock: Res<Clock>,
    mut sun_light: Single<(&mut DirectionalLight, &mut Transform), With<Sun>>,
    mut ambient: ResMut<GlobalAmbientLight>,
    mut clear: ResMut<ClearColor>,
    mut lamps: Query<&mut PointLight, With<StreetLamp>>,
    mut exposure: Single<&mut bevy::camera::Exposure>,
    glow: Option<Res<GlowMaterials>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut last: Local<f32>,
) {
    if (clock.hour - *last).abs() < 0.004 && *last != 0.0 {
        return;
    }
    *last = clock.hour;
    let (elevation, azimuth) = sun(clock.hour);
    let dark = night(clock.hour);
    let (light, transform) = &mut *sun_light;

    // Where the sun is, or after dark a high pale moon so the roofs keep a shape.
    let (el, az) = if elevation > 0.5 { (elevation.max(4.0), azimuth) } else { (48.0, 150.0) };
    let (el, az) = (el.to_radians(), az.to_radians());
    let toward = Vec3::new(az.sin() * el.cos(), el.sin(), -az.cos() * el.cos());
    **transform = Transform::from_translation(toward * 400.0).looking_at(Vec3::ZERO, Vec3::Y);

    // Everything is in real units, so the lamps and the windows are always
    // as bright as they are. What changes between noon and night is the
    // camera: twelve stops of exposure by day, four after dark.
    let height = (elevation / NOON_ELEVATION).clamp(0.0, 1.0);
    if elevation > 0.5 {
        // Near white at noon, amber only as it nears the horizon.
        light.color = Color::srgb(1.0, 0.96 - 0.30 * (1.0 - height).powi(2), 0.90 - 0.55 * (1.0 - height).powi(2));
        light.illuminance = 3_000.0 + 19_000.0 * height;
    } else {
        light.color = Color::srgb(0.62, 0.72, 0.95);
        light.illuminance = 0.6;
    }
    let day_sky = Color::srgb(0.62, 0.74, 0.92);
    let night_sky = Color::srgb(0.30, 0.38, 0.62);
    ambient.color = day_sky.mix(&night_sky, dark);
    ambient.brightness = 1_400.0 * (1.6f32 / 1_400.0).powf(dark);
    exposure.ev100 = 10.7 - 6.3 * dark;
    clear.0 = Color::srgb(0.56, 0.68, 0.84).mix(&Color::srgb(0.015, 0.02, 0.05), dark);

    // A 250 W sodium lamp gives about 28,000 lumens. Heihe's posts carry two.
    let lamp_on = dark > 0.25;
    for mut lamp in &mut lamps {
        lamp.intensity = if lamp_on { 62_000.0 } else { 0.0 };
    }
    if let Some(glow) = glow {
        if let Some(mut m) = materials.get_mut(&glow.windows) {
            // A lit room behind glass and a curtain: a few tens of candelas per square metre.
            m.emissive = LinearRgba::new(22.0, 14.5, 6.6, 1.0);
        }
        if let Some(mut m) = materials.get_mut(&glow.lamps) {
            let k = if lamp_on { 1.0 } else { 0.0 };
            m.emissive = LinearRgba::new(160.0 * k, 96.0 * k, 34.0 * k, 1.0);
        }
    }
}
