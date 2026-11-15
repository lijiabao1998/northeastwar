shader_type spatial;
render_mode diffuse_burley, cull_disabled;

void fragment() {
	ALBEDO = vec3(0.8, 0.83, 0.86);
	ROUGHNESS = 0.88;
	METALLIC = 0.0;
}
