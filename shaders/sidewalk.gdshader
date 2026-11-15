shader_type spatial;
render_mode diffuse_burley, cull_disabled;

varying vec3 v_world;

void vertex() {
	v_world = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
}

void fragment() {
	vec2 cell = fract(v_world.xz * vec2(2.2, 4.5));
	vec2 id = floor(v_world.xz * vec2(2.2, 4.5));
	if (mod(id.y, 2.0) > 0.5) {
		cell.x = fract(cell.x + 0.5);
	}
	float mortar = step(cell.x, 0.08) + step(cell.y, 0.1);
	mortar = clamp(mortar, 0.0, 1.0);
	vec3 brick = vec3(0.55, 0.22, 0.16);
	vec3 snow = vec3(0.82, 0.84, 0.86);
	float cover = 0.55 + 0.2 * step(0.6, fract(sin(dot(id, vec2(3.1, 8.7))) * 40.0));
	vec3 albedo = mix(brick, vec3(0.72, 0.7, 0.68), mortar);
	albedo = mix(albedo, snow, cover);
	ALBEDO = albedo;
	ROUGHNESS = 0.86;
	METALLIC = 0.0;
}
