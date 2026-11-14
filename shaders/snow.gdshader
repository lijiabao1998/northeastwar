shader_type spatial;
render_mode diffuse_burley, cull_disabled;

varying vec3 v_world;
varying vec3 v_color;

void vertex() {
	v_world = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
	v_color = COLOR.rgb;
}

void fragment() {
	float n = fract(sin(dot(v_world.xz, vec2(17.1, 45.7))) * 43758.5);
	float drift = smoothstep(0.35, 0.9, fract(sin(dot(floor(v_world.xz / 3.5), vec2(3.0, 7.0))) * 90.0));
	vec3 albedo = v_color * (0.9 + 0.12 * n);
	albedo = mix(albedo, albedo * vec3(0.82, 0.86, 0.92), drift * 0.35);
	ALBEDO = albedo;
	ROUGHNESS = 0.9;
}
