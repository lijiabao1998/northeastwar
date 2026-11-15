shader_type spatial;
render_mode diffuse_burley, cull_disabled;

void fragment() {
	vec2 cell = fract(UV * vec2(5.0, 9.0));
	vec2 id = floor(UV * vec2(5.0, 9.0));
	float pane = step(0.22, cell.x) * step(cell.x, 0.78) * step(0.18, cell.y) * step(cell.y, 0.75);
	float lit = step(0.38, fract(sin(dot(id, vec2(17.0, 5.1))) * 43758.0));
	vec3 wall = vec3(0.16, 0.17, 0.19);
	vec3 glow = vec3(1.0, 0.7, 0.32);
	ALBEDO = mix(wall, glow, pane * lit);
	EMISSION = glow * pane * lit * 1.6;
	ROUGHNESS = 0.7;
	METALLIC = 0.0;
}
