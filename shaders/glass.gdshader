shader_type spatial;
render_mode diffuse_burley, cull_disabled;

#include "res://shaders/cuts.gdshaderinc"

uniform float night : hint_range(0.0, 1.0) = 0.45;

varying float v_seed;

float hashf(float n) {
	return fract(sin(n) * 43758.5453);
}

void vertex() {
	v_seed = UV2.x;
	v_world = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
}

void fragment() {
	if (carved(v_world)) {
		discard;
	}
	float pane_x = floor(UV.x / 1.15);
	float pane_y = floor(UV.y / 1.35);
	vec2 cell = vec2(fract(UV.x / 1.15), fract(UV.y / 1.35));
	float border = 1.0 - smoothstep(0.07, 0.13, cell.x) * smoothstep(0.93, 0.87, cell.x)
		* smoothstep(0.07, 0.13, cell.y) * smoothstep(0.93, 0.87, cell.y);
	border = clamp(border, 0.0, 1.0);
	float mullion = step(0.47, cell.x) * step(cell.x, 0.53);
	float frame = clamp(border + mullion, 0.0, 1.0);
	float glass = 1.0 - frame;
	float h = hashf(pane_x * 8.1 + pane_y * 3.7 + v_seed * 19.0);
	float lit = step(0.48, h) * smoothstep(0.05, 0.6, night);
	vec3 interior = vec3(1.0, 0.64, 0.30);
	vec3 glass_col = mix(vec3(0.07, 0.09, 0.12), interior, lit);
	glass_col = mix(glass_col, vec3(0.55, 0.62, 0.7), (1.0 - night) * 0.25);
	float bars = step(0.84, fract(UV.x * 8.0)) + step(0.9, fract(UV.y * 6.0));
	bars = clamp(bars, 0.0, 1.0);
	ALBEDO = mix(glass_col, vec3(0.95, 0.96, 0.94), frame);
	ALBEDO = mix(ALBEDO, vec3(0.14, 0.15, 0.16), bars * 0.8);
	ROUGHNESS = mix(0.16, 0.55, max(frame, bars));
	METALLIC = max(0.04 * glass, bars * 0.45);
	EMISSION = interior * glass * lit * (2.2 + night * 1.4) * (1.0 - bars);
}
