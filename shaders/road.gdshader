shader_type spatial;
render_mode diffuse_burley, cull_disabled;

varying vec3 v_world;

void vertex() {
	v_world = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
}

void fragment() {
	float across = clamp(UV.x, 0.0, 1.0);
	float grit = fract(sin(floor(v_world.x * 0.35) + floor(v_world.z * 0.35) * 3.0) * 43758.5);
	vec3 packed = vec3(0.78, 0.81, 0.84) * (0.96 + 0.06 * grit);
	vec3 slush = vec3(0.22, 0.24, 0.26);
	float rut = smoothstep(0.045, 0.0, abs(across - 0.36)) + smoothstep(0.045, 0.0, abs(across - 0.64));
	rut = clamp(rut, 0.0, 1.0);
	float curb = smoothstep(0.08, 0.0, across) + smoothstep(0.92, 1.0, across);
	curb = clamp(curb, 0.0, 1.0);
	vec3 albedo = mix(packed, slush, rut * 0.85);
	albedo = mix(albedo, vec3(0.55, 0.57, 0.58), curb);
	ALBEDO = albedo;
	ROUGHNESS = mix(0.9, 0.45, rut);
	SPECULAR = mix(0.05, 0.25, rut);
	METALLIC = 0.0;
}
