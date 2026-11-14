shader_type spatial;
render_mode diffuse_burley, cull_disabled;

void fragment() {
	vec2 brick = vec2(UV.y * 1.6, UV.x * 7.0);
	vec2 cell = fract(brick);
	vec2 id = floor(brick);
	// Running bond.
	if (mod(id.y, 2.0) > 0.5) {
		cell.x = fract(cell.x + 0.5);
	}
	float mortar = 1.0 - smoothstep(0.0, 0.08, cell.x) * smoothstep(1.0, 0.92, cell.x)
		* smoothstep(0.0, 0.12, cell.y) * smoothstep(1.0, 0.88, cell.y);
	mortar = clamp(mortar, 0.0, 1.0);
	float n = fract(sin(dot(id, vec2(19.1, 7.7))) * 43758.5);
	vec3 paver = vec3(0.62, 0.28, 0.20) * (0.9 + 0.18 * n);
	vec3 snow = vec3(0.90, 0.92, 0.94);
	float edge_snow = smoothstep(0.15, 0.0, UV.x) + smoothstep(0.85, 1.0, UV.x);
	edge_snow = clamp(edge_snow, 0.0, 1.0);
	float patch = smoothstep(0.62, 0.78, fract(sin(dot(id, vec2(3.1, 9.2))) * 234.1));
	vec3 albedo = mix(paver, vec3(0.72, 0.74, 0.76), mortar);
	albedo = mix(albedo, snow, clamp(edge_snow + patch * 0.85, 0.0, 1.0));
	ALBEDO = albedo;
	ROUGHNESS = mix(0.78, 0.94, edge_snow + patch);
}
