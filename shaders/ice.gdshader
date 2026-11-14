shader_type spatial;
render_mode diffuse_burley, cull_disabled;

varying vec3 v_world;

void vertex() {
	v_world = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
	float ridge = sin(v_world.x * 0.04 + v_world.z * 0.011);
	ridge += sin(v_world.z * 0.08 + 1.7) * 0.4;
	VERTEX.y += ridge * ridge * 0.28;
}

void fragment() {
	float x = v_world.x;
	float z = v_world.z;
	float wind = sin(x * 0.18 + z * 0.04);
	float streak = smoothstep(0.2, 0.85, fract(sin(x * 0.13 + z * 0.031) * 19.0));
	float ridge = sin(x * 0.028 + z * 0.007);
	float crack = smoothstep(0.86, 0.995, abs(sin(x * 0.21 + z * 0.047)));
	float crack2 = smoothstep(0.93, 0.995, abs(sin(x * 0.05 - z * 0.09 + 2.0)));
	float track = smoothstep(0.08, 0.0, abs(fract((x + z * 0.15) * 0.04) - 0.5));
	vec3 snow = vec3(0.94, 0.95, 0.97);
	vec3 packed = vec3(0.75, 0.8, 0.84);
	vec3 ice = vec3(0.45, 0.62, 0.72);
	vec3 albedo = mix(snow, packed, streak * 0.4 + ridge * 0.1);
	albedo = mix(albedo, ice, clamp(crack + crack2, 0.0, 1.0) * 0.55);
	albedo = mix(albedo, vec3(0.7, 0.74, 0.76), track * 0.25);
	albedo *= 0.95 + 0.07 * wind;
	float height = crack + ridge * 0.2;
	NORMAL = normalize(NORMAL + vec3(dFdx(height), dFdy(height), 0.0));
	ALBEDO = albedo;
	ROUGHNESS = mix(0.78, 0.22, clamp(crack + crack2, 0.0, 1.0));
	SPECULAR = mix(0.2, 0.65, crack);
	METALLIC = 0.0;
}
