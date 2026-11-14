shader_type spatial;
render_mode diffuse_burley, cull_disabled;

varying vec3 v_world;

uniform sampler2D stone_alb : source_color, repeat_enable;
uniform sampler2D stone_nrm : hint_normal, repeat_enable;
uniform sampler2D brick_alb : source_color, repeat_enable;
uniform sampler2D snow_alb : source_color, repeat_enable;

void vertex() {
	v_world = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
}

void fragment() {
	vec2 uv = v_world.xz * vec2(0.42, 0.7);
	vec3 stone = texture(stone_alb, uv).rgb;
	vec3 brick = texture(brick_alb, uv * 1.4).rgb;
	float band = step(0.78, fract(v_world.x * 0.08 + v_world.z * 0.05));
	vec3 albedo = mix(stone, brick, band);
	vec3 snow = texture(snow_alb, v_world.xz * 0.1).rgb;
	float drift = smoothstep(0.5, 0.85, texture(snow_alb, v_world.xz * 0.04).g);
	albedo = mix(albedo, snow, drift * 0.62);
	NORMAL_MAP = texture(stone_nrm, uv).rgb;
	ALBEDO = albedo;
	ROUGHNESS = mix(0.7, 0.9, drift);
	METALLIC = 0.0;
}
