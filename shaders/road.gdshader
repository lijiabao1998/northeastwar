shader_type spatial;
render_mode diffuse_burley, cull_disabled;

void fragment() {
	float across = UV.x;
	float along = UV.y;
	float grit = fract(sin(floor(along * 3.1) * 12.3 + floor(across * 22.0) * 4.1) * 43758.5);
	vec3 dirty = vec3(0.42, 0.43, 0.44) * (0.88 + 0.18 * grit);
	vec3 packed = vec3(0.8, 0.82, 0.84);
	vec3 ice = vec3(0.55, 0.64, 0.7);
	float shoulder = smoothstep(0.16, 0.02, across) + smoothstep(0.84, 0.98, across);
	shoulder = clamp(shoulder, 0.0, 1.0);
	float rut_l = smoothstep(0.055, 0.0, abs(across - 0.34));
	float rut_r = smoothstep(0.055, 0.0, abs(across - 0.66));
	float track = clamp(rut_l + rut_r, 0.0, 1.0);
	vec3 albedo = mix(dirty, packed, shoulder);
	albedo = mix(albedo, vec3(0.28, 0.29, 0.3), track * 0.45);
	albedo = mix(albedo, ice, track * 0.35);
	float bump = shoulder * 0.4 - track * 0.8 + grit * 0.05;
	NORMAL = normalize(NORMAL + vec3(dFdx(bump), dFdy(bump), 0.0) * 2.2);
	ALBEDO = albedo;
	ROUGHNESS = mix(0.9, 0.28, track * 0.65);
	SPECULAR = mix(0.08, 0.45, track);
	METALLIC = 0.0;
}
