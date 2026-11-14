shader_type spatial;
render_mode diffuse_burley, cull_disabled;

void fragment() {
	float across = UV.x;
	float along = UV.y;
	float cleared = smoothstep(0.0, 0.16, across) * smoothstep(1.0, 0.84, across);
	float grit = fract(sin(floor(along * 1.7) * 12.3 + floor(across * 40.0) * 4.1) * 43758.5);
	vec3 asphalt = vec3(0.105, 0.108, 0.112) * (0.92 + 0.08 * grit);
	vec3 slush = vec3(0.78, 0.81, 0.84);
	float track_l = smoothstep(0.045, 0.0, abs(across - 0.36));
	float track_r = smoothstep(0.045, 0.0, abs(across - 0.64));
	asphalt *= 1.0 - (track_l + track_r) * 0.35;
	// Packed snow and ice stay on the shoulders. The centre is wet and dark.
	ALBEDO = mix(slush, asphalt, cleared);
	ROUGHNESS = mix(0.88, 0.16, cleared);
	SPECULAR = mix(0.2, 0.65, cleared);
	METALLIC = 0.0;
}
