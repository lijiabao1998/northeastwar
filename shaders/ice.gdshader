shader_type spatial;
render_mode diffuse_burley, cull_disabled;

varying vec3 v_world;

void vertex() {
	v_world = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
	// Pressure ridges a few tens of centimetres high, running with the current.
	float ridge = sin(v_world.x * 0.045 + v_world.z * 0.012);
	ridge += sin(v_world.z * 0.09 + 2.0) * 0.35;
	VERTEX.y += ridge * ridge * 0.22;
}

void fragment() {
	float x = v_world.x;
	float z = v_world.z;
	float wind = sin(x * 0.22 + z * 0.05);
	float streak = smoothstep(0.35, 0.9, fract(sin(x * 0.17 + z * 0.04) * 12.0));
	float ridge = sin(x * 0.03 + z * 0.008);
	float crack = smoothstep(0.9, 0.99, abs(sin(x * 0.19 + z * 0.05)));
	vec3 snow = vec3(0.93, 0.94, 0.96);
	vec3 packed = vec3(0.78, 0.82, 0.86);
	vec3 ice = vec3(0.62, 0.72, 0.78);
	vec3 albedo = mix(snow, packed, streak * 0.35 + ridge * 0.08);
	albedo = mix(albedo, ice, crack * 0.45);
	albedo *= 0.96 + 0.06 * wind;
	ALBEDO = albedo;
	ROUGHNESS = mix(0.72, 0.9, streak);
	SPECULAR = 0.25;
}
