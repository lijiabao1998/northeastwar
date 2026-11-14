shader_type spatial;
render_mode diffuse_burley, cull_disabled;

// Poly Haven snow_02, snow_04 and asphalt_02. World-space UVs so a long
// road is not one stretched tile. UV.x is still across the carriageway.

varying vec3 v_world;

uniform sampler2D snow_alb : source_color, repeat_enable;
uniform sampler2D snow_nrm : hint_normal, repeat_enable;
uniform sampler2D dirty_alb : source_color, repeat_enable;
uniform sampler2D asphalt_alb : source_color, repeat_enable;
uniform sampler2D asphalt_nrm : hint_normal, repeat_enable;

void vertex() {
	v_world = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
}

void fragment() {
	vec2 uv = v_world.xz * 0.16;
	float across = clamp(UV.x, 0.0, 1.0);
	vec3 snow = texture(snow_alb, uv).rgb;
	vec3 dirty = texture(dirty_alb, uv * 1.35).rgb;
	vec3 tar = texture(asphalt_alb, uv * 1.7).rgb;
	float rut = smoothstep(0.06, 0.0, abs(across - 0.34)) + smoothstep(0.06, 0.0, abs(across - 0.66));
	rut = clamp(rut, 0.0, 1.0);
	float slush = smoothstep(0.2, 0.02, across) + smoothstep(0.8, 0.98, across);
	slush = clamp(slush, 0.0, 1.0);
	vec3 albedo = mix(dirty, snow, 0.55);
	albedo = mix(albedo, tar, rut * 0.82);
	albedo = mix(albedo, mix(tar, dirty, 0.45), slush * 0.65);
	vec3 n_snow = texture(snow_nrm, uv).rgb;
	vec3 n_tar = texture(asphalt_nrm, uv * 1.7).rgb;
	NORMAL_MAP = mix(n_snow, n_tar, max(rut, slush * 0.5));
	ALBEDO = albedo;
	ROUGHNESS = mix(0.82, 0.38, rut);
	SPECULAR = mix(0.12, 0.4, rut);
	METALLIC = 0.0;
}
