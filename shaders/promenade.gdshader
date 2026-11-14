shader_type spatial;
render_mode diffuse_burley, cull_disabled;

varying vec3 v_world;

void vertex() {
	v_world = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
}

void fragment() {
	vec2 p = v_world.xz;
	vec2 cell = fract(p / vec2(1.15, 0.72));
	vec2 id = floor(p / vec2(1.15, 0.72));
	float mortar = 1.0 - smoothstep(0.0, 0.07, cell.x) * smoothstep(1.0, 0.93, cell.x)
		* smoothstep(0.0, 0.09, cell.y) * smoothstep(1.0, 0.91, cell.y);
	mortar = clamp(mortar, 0.0, 1.0);
	float n = fract(sin(dot(id, vec2(8.1, 2.3))) * 43758.5);
	// Granite pavers with a red brick band, the riverfront pattern.
	float band = step(0.72, fract(id.y * 0.2));
	vec3 stone = mix(vec3(0.45, 0.46, 0.48), vec3(0.55, 0.52, 0.50), n);
	vec3 brick = vec3(0.52, 0.24, 0.20) * (0.9 + 0.15 * n);
	vec3 albedo = mix(mix(stone, brick, band), vec3(0.75, 0.76, 0.78), mortar);
	float snow = smoothstep(0.78, 0.95, fract(sin(dot(id, vec2(1.7, 9.2))) * 40.0));
	albedo = mix(albedo, vec3(0.90, 0.92, 0.94), snow * 0.75);
	ALBEDO = albedo;
	ROUGHNESS = mix(0.72, 0.95, snow);
}
