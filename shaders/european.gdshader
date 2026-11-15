shader_type spatial;
render_mode diffuse_burley, cull_disabled;

#include "res://shaders/cuts.gdshaderinc"

uniform float night : hint_range(0.0, 1.0) = 0.45;
uniform float floor_h = 4.05;
uniform float bay = 3.7;

varying vec3 v_color;
varying float v_seed;

float hashf(float n) {
	return fract(sin(n) * 43758.5453);
}

void vertex() {
	v_color = COLOR.rgb;
	v_seed = UV2.x;
	v_world = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
}

void fragment() {
	if (carved(v_world)) {
		discard;
	}
	vec3 albedo = v_color * 0.92;
	float rough = 0.7;
	vec3 emission = vec3(0.0);
	if (UV.y >= 0.0) {
	float col = floor(UV.x / bay);
	float row = floor(UV.y / floor_h);
	vec2 cell = vec2(fract(UV.x / bay), fract(UV.y / floor_h));
	// White pilasters and a string course, then an arched opening.
	float pier = smoothstep(0.0, 0.07, cell.x) * smoothstep(1.0, 0.93, cell.x);
	float course = smoothstep(0.02, 0.06, cell.y) * smoothstep(0.98, 0.90, cell.y);
	vec2 q = cell - vec2(0.5, 0.62);
	q.x *= 1.35;
	float arch = 1.0 - smoothstep(0.30, 0.36, length(q));
	float shaft = step(0.18, cell.y) * step(cell.y, 0.62) * step(abs(cell.x - 0.5), 0.22);
	float opening = max(shaft, arch) * pier * course;
	float frame = smoothstep(0.30, 0.40, length(q + vec2(0.0, 0.0))) * opening;
	// Keep a white rim by shrinking the dark opening.
	vec2 q2 = cell - vec2(0.5, 0.60);
	q2.x *= 1.55;
	float inner_arch = 1.0 - smoothstep(0.20, 0.25, length(q2));
	float inner_shaft = step(0.24, cell.y) * step(cell.y, 0.60) * step(abs(cell.x - 0.5), 0.15);
	float glass = max(inner_shaft, inner_arch) * pier;
	frame = clamp(opening - glass, 0.0, 1.0);

	vec3 wall = v_color * (0.94 + 0.06 * hashf(col + row * 5.0));
	vec3 trim = vec3(0.95, 0.94, 0.90);
	float h = hashf(col * 12.1 + row * 4.4 + v_seed * 20.0);
	float lit = step(0.35, h) * smoothstep(0.05, 0.55, night);
	vec3 interior = vec3(1.0, 0.72, 0.4);
	vec3 glass_col = mix(vec3(0.06, 0.07, 0.1), interior, lit);
	albedo = mix(wall, trim, clamp(1.0 - pier * course, 0.0, 1.0));
	albedo = mix(albedo, trim, frame);
	albedo = mix(albedo, glass_col, glass);
	// Cornice band near the top of each floor.
	float cornice = smoothstep(0.86, 0.90, cell.y) * smoothstep(0.98, 0.94, cell.y);
	albedo = mix(albedo, trim, cornice);
	rough = mix(0.72, 0.18, glass);
	emission = interior * glass * lit * (1.6 + night);
	emission += albedo * 0.16;
	}
	ALBEDO = albedo;
	ROUGHNESS = rough;
	EMISSION = emission;
}
