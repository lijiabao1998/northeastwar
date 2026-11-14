shader_type spatial;
render_mode diffuse_burley, cull_disabled;

// Red brick from Poly Haven, scaled in metres. Snow sits in the joints and
// along the building side so the walk is not a field of giant tiles.

varying vec3 v_world;

uniform sampler2D brick_alb : source_color, repeat_enable;
uniform sampler2D brick_nrm : hint_normal, repeat_enable;
uniform sampler2D snow_alb : source_color, repeat_enable;
uniform sampler2D snow_nrm : hint_normal, repeat_enable;

void vertex() {
	v_world = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
}

void fragment() {
	vec2 uv = v_world.xz * vec2(0.72, 1.35);
	vec3 brick = texture(brick_alb, uv).rgb;
	vec3 snow = texture(snow_alb, v_world.xz * 0.14).rgb;
	float grain = texture(snow_alb, v_world.xz * 0.05).r;
	float cover = smoothstep(0.42, 0.78, grain);
	// More snow in a band, less where feet have packed the middle.
	float packed = smoothstep(0.15, 0.0, abs(fract(v_world.x * 0.07 + v_world.z * 0.04) - 0.5));
	vec3 albedo = mix(brick, snow, clamp(0.28 + cover * 0.55 - packed * 0.15, 0.0, 0.92));
	NORMAL_MAP = mix(texture(brick_nrm, uv).rgb, texture(snow_nrm, v_world.xz * 0.14).rgb, cover);
	ALBEDO = albedo;
	ROUGHNESS = mix(0.72, 0.9, cover);
	METALLIC = 0.0;
}
