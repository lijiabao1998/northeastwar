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

/// True when the cut-away glass is drawn unlit (in full day and full night); in
/// twilight it is lit, since no fixed colour matches the light there.
pub fn ghost_unlit(hour: f32) -> bool {
    let d = night(hour);
    d <= 0.0 || d >= 1.0
}

/// True when the street lamps are lit.
pub fn lamps_on(hour: f32) -> bool {
    night(hour) > 0.25
}

/// 0 in full day, 1 in full night.
pub fn night(hour: f32) -> f32 {
    let (elevation, _) = sun(hour);
    ((2.0 - elevation) / 9.0).clamp(0.0, 1.0)
}

pub fn setup(mut commands: Commands) {
    commands.spawn((
        DirectionalLight { shadow_maps_enabled: true, illuminance: 20_000.0, ..default() },
        shadow_slab(1.0),
        Transform::default(),
        Sun,
    ));
}

/// One shadow cascade holding exactly what the camera sees at this zoom.
///
/// The camera is orthographic and looks down at 36°, so everything on screen
/// lies between D − 0.684·H + 1.7 − 1.695·h and D + 0.684·H + 1.7 from it, where
/// D is its distance from you, H the height of the view in metres and h the
/// height of a roof. The tallest things drawn are three 17-storey CMAB towers,
/// 51.7 m to the top of the parapet; the 100 m margin below covers roofs up to
/// 59 m. Casters nearer the sun than the slab are clamped onto it, so they still
/// cast. Orthographic depth does not change how big things are on screen,
/// so splitting the slab into several cascades buys little; one slab at zoom 1 is
/// about 0.09 m a texel, where three fixed cascades gave 0.15–0.20 m.
pub fn shadow_slab(zoom: f32) -> bevy::light::CascadeShadowConfig {
    let d = Vec3::from_array(sim::cutaway::EYE_OFFSET).length();
    let h = crate::VIEW_METRES / zoom;
    let far = d + 0.69 * h + 5.0;
    bevy::light::CascadeShadowConfigBuilder {
        num_cascades: 1,
        minimum_distance: (d - 0.69 * h - 100.0).max(1.0),
        first_cascade_far_bound: far,
        maximum_distance: far,
        ..default()
    }
    .build()
}

/// Rebuild the shadow slab when the zoom changes.
pub fn follow_zoom(eye: Single<&crate::Eye>, mut sun: Single<&mut bevy::light::CascadeShadowConfig, With<Sun>>, mut last: Local<f32>) {
    if eye.zoom == *last {
        return;
    }
    *last = eye.zoom;
    **sun = shadow_slab(eye.zoom);
}

pub fn apply(
    clock: Res<Clock>,
    mut sun_light: Single<(&mut DirectionalLight, &mut Transform), With<Sun>>,
    mut ambient: ResMut<GlobalAmbientLight>,
    mut clear: ResMut<ClearColor>,
    mut lamps: Query<&mut Visibility, With<StreetLamp>>,
    mut exposure: Single<&mut bevy::camera::Exposure>,
    glow: Option<Res<GlowMaterials>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    (cutaway, gfx): (Option<Res<crate::Cutaway>>, Res<crate::graphics::Graphics>),
    mut last: Local<f32>,
    mut lamps_were_on: Local<Option<bool>>,
    mut ghost_state: Local<Option<u8>>,
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

    // The lamps are hidden by day rather than dimmed to nothing: a light of zero
    // intensity is still clustered and still shaded in every pixel it reaches, which
    // cost 3 to 7 ms of GPU time a frame at 3200 x 1800 by day, with the depth prepass
    // on, and more without it (--bench, November 2026). Lights
    // and materials are touched only when the lamps switch, since changing a
    // material makes Bevy prepare everything drawn with it again.
    // The cut-away glass. Unlit, it costs nothing extra, but an unlit colour takes
    // neither light nor exposure, so its colour is set by hand: white in full day, the
    // night sky's dark in full night (both match the lit glass). In twilight the light
    // and the exposure change too fast for any fixed colour to match, and an unlit
    // tint there was a grey veil twice as bright as the lit glass, so in twilight it is
    // lit. Lit and unlit share one pipeline (unlit is a flag in the material's uniform),
    // so switching only prepares the material again.
    if !gfx.ghost_lit {
        let state = if dark <= 0.0 { 0 } else if dark >= 1.0 { 2 } else { 1 };
        if *ghost_state != Some(state) {
            *ghost_state = Some(state);
            if let Some(mut ghost) = cutaway.as_ref().and_then(|c| materials.get_mut(&c.ghost)) {
                ghost.unlit = state != 1;
                let colour = if state == 2 { Color::srgb(0.07, 0.08, 0.13) } else { Color::WHITE };
                ghost.base_color = colour.with_alpha(0.16);
            }
        }
    }

    let lamp_on = lamps_on(clock.hour);
    if *lamps_were_on != Some(lamp_on) {
        *lamps_were_on = Some(lamp_on);
        for mut lamp in &mut lamps {
            *lamp = if lamp_on { Visibility::Inherited } else { Visibility::Hidden };
        }
        if let Some(mut m) = glow.and_then(|g| materials.get_mut(&g.lamps)) {
            let k = if lamp_on { 1.0 } else { 0.0 };
            m.emissive = LinearRgba::new(160.0 * k, 96.0 * k, 34.0 * k, 1.0);
        }
    }
}
