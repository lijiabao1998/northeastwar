shader_type spatial;
render_mode diffuse_burley, cull_disabled;

varying vec3 v_world;

void vertex() {
	v_world = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
}

void fragment() {
	float x = v_world.x;
	float z = v_world.z;
	float broad = sin(x * 0.031 + z * 0.017) * sin(z * 0.02);
	float grain = fract(sin(dot(floor(v_world.xz * 0.35), vec2(12.9, 78.2))) * 43758.5);
	float crack = smoothstep(0.82, 0.97, abs(sin(x * 0.21 + z * 0.13) * sin(x * 0.05 - z * 0.08)));
	vec3 snow = vec3(0.90, 0.93, 0.95);
	vec3 ice = vec3(0.62, 0.74, 0.80);
	vec3 albedo = mix(ice, snow, 0.45 + 0.35 * broad + 0.1 * grain);
	albedo = mix(albedo, vec3(0.45, 0.55, 0.62), crack * 0.55);
	ALBEDO = albedo;
	ROUGHNESS = mix(0.42, 0.78, grain);
	SPECULAR = 0.55;
}
