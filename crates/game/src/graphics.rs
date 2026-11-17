//! The choices that trade looks for frame time. Each can be set for a run with an
//! environment variable, so --shots can show them side by side and --bench can
//! time them; what each costs at 3200 x 1800 on this laptop is in
//! docs/rust-r3-receipt.md.
//!
//! The first value of each is the default.
//!
//! - NORTHEAST_MSAA = 1, 2 or 4 samples per pixel. Off by default: 4x cost 6 to 12 ms
//!   a frame here, much of it in resolves the GPU timestamps do not see, and 2x was no
//!   faster. At 200% scaling edges are only a little rougher close up, but zoomed out
//!   to the whole town, lines under a pixel wide (road centre lines, lamp posts) break
//!   into dots and will shimmer as the camera moves.
//! - NORTHEAST_SHADOWS = soft (Bevy's 9-tap Gaussian) or hard (one 2x2 hardware tap).
//!   Hard saves about 1 ms of GPU time with the defaults (2 to 4 with 4x MSAA and no
//!   depth prepass) but stair-steps every shadow edge close up.
//! - NORTHEAST_GHOST = unlit or lit (the glass a building turns into when it hides
//!   you). Unlit means unlit in full day and full night, with its colour set by the
//!   hour, and lit in twilight, where no fixed colour matches (sky::apply). By day the
//!   unlit walls lose their light and dark faces. Lit all the time, it is fully shaded
//!   over a whole building's worth of screen: about 3 ms by day, 4 to 5 at night.
//! - NORTHEAST_LAMPS = posts (one light per lamp post, between its heads and as bright
//!   as all of them) or heads (one per head). The pools on the ground are the same
//!   within 10%; the post's own shaft is no longer lit by its lamp (lamp_radius). 332
//!   lights instead of 498 saved up to 1.9 ms of GPU time at night (round 6; at one
//!   view within the noise).
//! - NORTHEAST_PREPASS = on or off: draw depth first, so the main pass lights each
//!   pixel once instead of once for every wall, roof and window drawn over it.
//!   The picture is the same; it saved 1 to 8 ms of GPU time, most at night.

use bevy::{light::ShadowFilteringMethod, prelude::*};

#[derive(Resource, Clone, Copy, Debug)]
pub struct Graphics {
    pub msaa: Msaa,
    pub shadows: ShadowFilteringMethod,
    pub ghost_lit: bool,
    pub lamp_per_head: bool,
    pub prepass: bool,
}

impl Graphics {
    pub fn from_env() -> Graphics {
        let var = |name: &str| std::env::var(name).ok().map(|v| v.trim().to_lowercase());
        let msaa = match var("NORTHEAST_MSAA").as_deref() {
            Some("2") => Msaa::Sample2,
            Some("4") => Msaa::Sample4,
            _ => Msaa::Off,
        };
        let shadows = match var("NORTHEAST_SHADOWS").as_deref() {
            Some("hard") => ShadowFilteringMethod::Hardware2x2,
            _ => ShadowFilteringMethod::Gaussian,
        };
        let ghost_lit = matches!(var("NORTHEAST_GHOST").as_deref(), Some("lit"));
        let lamp_per_head = matches!(var("NORTHEAST_LAMPS").as_deref(), Some("heads"));
        let prepass = !matches!(var("NORTHEAST_PREPASS").as_deref(), Some("off"));
        Graphics { msaa, shadows, ghost_lit, lamp_per_head, prepass }
    }

    pub fn describe(&self) -> String {
        format!(
            "MSAA {:?}, shadows {:?}, ghost {}, lamps {}, depth prepass {}",
            self.msaa,
            self.shadows,
            if self.ghost_lit { "lit" } else { "unlit" },
            if self.lamp_per_head { "one per head" } else { "one per post" },
            if self.prepass { "on" } else { "off" }
        )
    }
}

/// The size of a lamp's light. One per head stands under its head, clear of the
/// post and the arms. One per post stands between the heads, which on a post with
/// two heads is inside the post, so it gets no size: a sphere reaching through the
/// post's faces lit them white.
pub fn lamp_radius(gfx: &Graphics) -> f32 {
    if gfx.lamp_per_head {
        0.2
    } else {
        0.0
    }
}

/// The near plane of the orthographic camera: just in front of everything it can
/// see at this zoom (the front of the shadow slab, sky::shadow_slab). Bevy slices
/// the light clusters in depth from the near plane, so a near plane at 1 m puts
/// most slices in the empty air between the camera and the roofs.
pub fn near_plane(zoom: f32) -> f32 {
    (Vec3::from_array(sim::cutaway::EYE_OFFSET).length() - 0.69 * crate::VIEW_METRES / zoom - 100.0).max(1.0)
}
