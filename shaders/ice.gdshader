shader_type spatial;
render_mode diffuse_burley, cull_disabled;

varying vec3 v_world;

uniform sampler2D snow_alb : source_color, repeat_enable;
uniform sampler2D snow_nrm : hint_normal, repeat_enable;
uniform sampler2D dirty_alb : source_color, repeat_enable;

void vertex() {
	v_world = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
	float ridge = sin(v_world.x * 0.035 + v_world.z * 0.01);
	ridge += sin(v_world.z * 0.07 + 1.4) * 0.45;
	VERTEX.y += ridge * ridge * 0.55 + sin(v_world.x * 0.12) * 0.06;
}

void fragment() {
	vec2 uv = v_world.xz * 0.045;
	vec3 snow = texture(snow_alb, uv).rgb;
	vec3 packed = texture(dirty_alb, uv * 1.6).rgb;
	float wind = sin(v_world.x * 0.08 + v_world.z * 0.02);
	float crack = smoothstep(0.9, 0.995, abs(sin(v_world.x * 0.11 + v_world.z * 0.027)));
	float crack2 = smoothstep(0.94, 0.995, abs(sin(v_world.x * 0.02 - v_world.z * 0.06)));
	float open = clamp(crack + crack2, 0.0, 1.0);
	vec3 ice = vec3(0.28, 0.48, 0.62);
	vec3 albedo = mix(snow, packed, 0.25 + wind * 0.08);
	albedo = mix(albedo, ice, open * 0.7);
	NORMAL_MAP = texture(snow_nrm, uv).rgb;
	ALBEDO = albedo;
	ROUGHNESS = mix(0.78, 0.18, open);
	SPECULAR = mix(0.2, 0.7, open);
	METALLIC = 0.0;
}
