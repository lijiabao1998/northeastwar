shader_type spatial;
render_mode diffuse_burley, cull_disabled;

void fragment() {
	float across = UV.x;
	float along = UV.y;
	float grit = fract(sin(floor(along * 2.4) * 12.3 + floor(across * 18.0) * 4.1) * 43758.5);
	// Dirty packed snow in the wheel tracks, cleaner snow toward the curb.
	vec3 dirty = vec3(0.45, 0.46, 0.47) * (0.9 + 0.15 * grit);
	vec3 packed = vec3(0.78, 0.80, 0.82);
	float shoulder = smoothstep(0.18, 0.0, across) + smoothstep(0.82, 1.0, across);
	shoulder = clamp(shoulder, 0.0, 1.0);
	float track = smoothstep(0.07, 0.0, abs(across - 0.36)) + smoothstep(0.07, 0.0, abs(across - 0.64));
	vec3 albedo = mix(dirty, packed, shoulder);
	albedo = mix(albedo, vec3(0.32, 0.33, 0.34), clamp(track, 0.0, 1.0) * 0.55);
	ALBEDO = albedo;
	ROUGHNESS = 0.92;
	SPECULAR = 0.08;
	METALLIC = 0.0;
}
