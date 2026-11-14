shader_type spatial;
render_mode diffuse_burley, cull_disabled;

varying vec3 v_world;

void vertex() {
	v_world = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
}

void fragment() {
	float grit = fract(sin(dot(v_world.xz, vec2(0.17, 0.41))) * 43758.5);
	vec3 snow = vec3(0.90, 0.92, 0.94) * (0.96 + 0.05 * grit);
	ALBEDO = snow;
	ROUGHNESS = 0.9;
}
