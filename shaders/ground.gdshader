shader_type spatial;
render_mode diffuse_burley, cull_disabled;

varying vec3 v_world;

void vertex() {
	v_world = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
}

void fragment() {
	float n = fract(sin(dot(floor(v_world.xz * 0.15), vec2(3.0, 7.0))) * 90.0);
	ALBEDO = vec3(0.74, 0.78, 0.82) * (0.94 + 0.08 * n);
	ROUGHNESS = 0.9;
	METALLIC = 0.0;
}
