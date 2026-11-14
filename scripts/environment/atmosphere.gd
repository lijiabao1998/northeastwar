extends Node3D
class_name Atmosphere
## Dusk into night over Heihe. 17 January: sunset is about 15:58, so 16:00
## is already the blue hour and 18:00 is night.

var environment: Environment
var sun: DirectionalLight3D
var _sky: ProceduralSkyMaterial
var _city: CityView
var _props: CityProps

func _ready() -> void:
	_sky = ProceduralSkyMaterial.new()
	_sky.sky_curve = 0.12
	_sky.ground_curve = 0.08
	_sky.sun_angle_max = 32.0
	_sky.sun_curve = 0.06
	environment = Environment.new()
	environment.background_mode = Environment.BG_SKY
	var sky := Sky.new()
	sky.sky_material = _sky
	environment.sky = sky
	environment.ambient_light_source = Environment.AMBIENT_SOURCE_COLOR
	environment.ambient_light_color = Color(0.62, 0.67, 0.72)
	environment.tonemap_mode = Environment.TONE_MAPPER_ACES
	environment.tonemap_white = 6.0
	environment.ssao_radius = 1.4
	environment.ssao_intensity = 1.6
	environment.ssao_power = 1.4
	environment.ssil_radius = 3.0
	environment.ssil_intensity = 0.8
	environment.glow_intensity = 0.35
	environment.glow_strength = 0.55
	environment.glow_bloom = 0.04
	environment.glow_hdr_threshold = 1.05
	environment.glow_hdr_scale = 1.1
	environment.fog_mode = Environment.FOG_MODE_EXPONENTIAL
	environment.fog_density = 0.0011
	environment.fog_aerial_perspective = 0.55
	environment.fog_sky_affect = 0.4
	environment.volumetric_fog_density = 0.0022
	environment.volumetric_fog_albedo = Color(0.62, 0.66, 0.74)
	environment.volumetric_fog_emission = Color(0.03, 0.02, 0.015)
	environment.volumetric_fog_emission_energy = 0.4
	environment.volumetric_fog_length = 180.0
	environment.volumetric_fog_detail_spread = 2.0
	environment.volumetric_fog_ambient_inject = 0.35
	environment.volumetric_fog_gi_inject = 0.25
	environment.sdfgi_cascades = 4
	environment.sdfgi_min_cell_size = 0.85
	environment.sdfgi_max_distance = 160.0
	environment.sdfgi_use_occlusion = true
	environment.sdfgi_bounce_feedback = 0.35
	environment.sdfgi_y_scale = Environment.SDFGI_Y_SCALE_75_PERCENT
	var world := WorldEnvironment.new()
	world.environment = environment
	add_child(world)
	sun = DirectionalLight3D.new()
	sun.shadow_bias = 0.08
	sun.shadow_normal_bias = 1.2
	sun.directional_shadow_max_distance = 140.0
	sun.light_angular_distance = 0.6
	add_child(sun)
	GameState.quality_changed.connect(apply_quality)
	apply_quality(GameState.quality)
	_apply_hour(GameState.hour)

func bind(city: CityView, props: CityProps) -> void:
	_city = city
	_props = props

func _process(_delta: float) -> void:
	_apply_hour(GameState.hour)

func apply_quality(level: String) -> void:
	var vp := get_viewport()
	vp.scaling_3d_mode = Viewport.SCALING_3D_MODE_BILINEAR
	environment.fog_enabled = true
	match level:
		"low":
			environment.ssao_enabled = false
			environment.ssil_enabled = false
			environment.ssr_enabled = false
			environment.glow_enabled = false
			environment.volumetric_fog_enabled = false
			environment.sdfgi_enabled = false
			environment.adjustment_enabled = false
			environment.fog_density = 0.0016
			vp.msaa_3d = Viewport.MSAA_DISABLED
			vp.scaling_3d_scale = 0.7
			sun.shadow_enabled = false
		"high":
			environment.ssao_enabled = true
			environment.ssil_enabled = true
			# Screen-space reflections smear into a grid on this orthographic
			# view, so high keeps SSAO, SSIL, volumetric fog and SDFGI.
			environment.ssr_enabled = false
			environment.glow_enabled = true
			environment.volumetric_fog_enabled = true
			environment.sdfgi_enabled = true
			environment.adjustment_enabled = true
			environment.adjustment_contrast = 1.06
			environment.adjustment_saturation = 0.9
			environment.fog_density = 0.0007
			vp.msaa_3d = Viewport.MSAA_4X
			vp.scaling_3d_scale = 1.0
			sun.shadow_enabled = true
			sun.directional_shadow_mode = DirectionalLight3D.SHADOW_PARALLEL_4_SPLITS
		_:
			environment.ssao_enabled = true
			environment.ssil_enabled = false
			environment.ssr_enabled = false
			environment.glow_enabled = true
			environment.volumetric_fog_enabled = false
			environment.sdfgi_enabled = false
			environment.adjustment_enabled = false
			environment.fog_density = 0.0011
			vp.msaa_3d = Viewport.MSAA_2X
			vp.scaling_3d_scale = 0.9
			sun.shadow_enabled = true
			sun.directional_shadow_mode = DirectionalLight3D.SHADOW_PARALLEL_2_SPLITS

func _apply_hour(hour: float) -> void:
	var t := clampf((hour - 16.0) / 2.0, 0.0, 1.0)
	var sunset := clampf((hour - 16.0) / 1.2, 0.0, 1.0)
	var night := smoothstep(15.85, 17.5, hour)
	_sky.sky_top_color = Color(0.20, 0.26, 0.34).lerp(Color(0.015, 0.02, 0.03), t)
	_sky.sky_horizon_color = Color(0.72, 0.48, 0.34).lerp(Color(0.07, 0.07, 0.08), t)
	_sky.ground_horizon_color = Color(0.58, 0.60, 0.64).lerp(Color(0.05, 0.05, 0.06), t)
	_sky.ground_bottom_color = Color(0.42, 0.44, 0.48).lerp(Color(0.03, 0.03, 0.035), t)
	# Sunset on 17 January is about 238° from north (WSW). The light shines
	# from the sun toward the city, so the node looks the other way.
	var elev := lerpf(5.5, -14.0, sunset)
	sun.rotation_degrees = Vector3(-elev, 238.0 + 180.0, 0.0)
	sun.light_energy = lerpf(0.42, 0.03, sunset)
	sun.light_color = Color(1.0, 0.78, 0.58).lerp(Color(0.55, 0.62, 0.72), sunset)
	environment.ambient_light_color = Color(0.62, 0.68, 0.74).lerp(Color(0.28, 0.32, 0.38), t)
	environment.ambient_light_energy = lerpf(0.7, 0.22, t)
	environment.tonemap_exposure = 0.92
	environment.fog_light_color = Color(0.62, 0.66, 0.70).lerp(Color(0.08, 0.08, 0.09), t)
	var haze := lerpf(0.0009, 0.0016, t)
	if GameState.quality == "low":
		haze = lerpf(0.0014, 0.0022, t)
	environment.fog_density = haze
	environment.volumetric_fog_albedo = Color(0.55, 0.58, 0.62).lerp(Color(0.25, 0.24, 0.22), t)
	environment.volumetric_fog_emission = Color(0.08, 0.05, 0.03)
	if _city:
		_city.set_night(night)
	if _props:
		_props.set_lamp_strength(smoothstep(15.8, 16.35, hour))
