shader_type spatial;
render_mode diffuse_burley, cull_disabled;

uniform float night : hint_range(0.0, 1.0) = 0.45;
uniform float floor_h = 3.0;
uniform float bay = 3.15;

varying vec3 v_color;
varying float v_seed;
varying float v_shop;
varying vec3 v_world;

float hashf(float n) {
	return fract(sin(n) * 43758.5453);
}

void vertex() {
	v_color = COLOR.rgb;
	v_seed = UV2.x;
	v_shop = UV2.y;
	v_world = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
}

void fragment() {
	vec3 albedo = v_color;
	float rough = 0.84;
	vec3 emission = vec3(0.0);
	// Parapets and stair bulkheads are tagged with a negative V.
	if (UV.y < 0.0) {
		vec2 tile = vec2(UV.x, v_world.y) * 1.65;
		float grout = clamp(step(fract(tile.x), 0.08) + step(fract(tile.y), 0.08), 0.0, 1.0);
		vec3 ceramic = v_color * (0.93 + 0.07 * hashf(floor(tile.x) + floor(tile.y) * 13.0));
		albedo = mix(ceramic, ceramic * 0.55, grout);
	} else {
		float u = UV.x;
		float v = UV.y;
		vec2 tile = vec2(u, v) * vec2(1.5, 1.6);
		vec2 fr = fract(tile);
		float grout = clamp(step(fr.x, 0.07) + step(fr.y, 0.07), 0.0, 1.0);
		vec3 ceramic = v_color * (0.90 + 0.10 * hashf(floor(tile.x) * 3.1 + floor(tile.y) * 9.7));
		ceramic = mix(ceramic, ceramic * 0.55, grout);

		float col = floor(u / bay);
		float row = floor(max(v, 0.0) / floor_h);
		vec2 cell = vec2(fract(u / bay), fract(max(v, 0.0) / floor_h));
		bool shop_floor = v_shop > 0.5 && row < 0.5;

		float frame = 0.0;
		float glass = 0.0;
		if (shop_floor) {
			float gx = smoothstep(0.045, 0.08, cell.x) * smoothstep(0.955, 0.92, cell.x);
			float gy = smoothstep(0.08, 0.14, cell.y) * smoothstep(0.94, 0.86, cell.y);
			glass = gx * gy;
			frame = smoothstep(0.015, 0.045, cell.x) * smoothstep(0.985, 0.955, cell.x);
			frame *= smoothstep(0.03, 0.07, cell.y) * smoothstep(0.97, 0.93, cell.y);
			frame *= (1.0 - glass);
		} else {
			float outer = smoothstep(0.15, 0.19, cell.x) * smoothstep(0.85, 0.81, cell.x);
			outer *= smoothstep(0.20, 0.26, cell.y) * smoothstep(0.80, 0.72, cell.y);
			float inner = smoothstep(0.23, 0.275, cell.x) * smoothstep(0.77, 0.725, cell.x);
			inner *= smoothstep(0.30, 0.36, cell.y) * smoothstep(0.72, 0.64, cell.y);
			float mullion = step(0.485, cell.x) * step(cell.x, 0.515);
			mullion += step(0.47, cell.y) * step(cell.y, 0.53);
			mullion = clamp(mullion, 0.0, 1.0);
			glass = inner * (1.0 - mullion);
			frame = clamp(outer - glass, 0.0, 1.0);
		}

		vec3 upvc = vec3(0.94, 0.95, 0.93);
		float h = hashf(col * 17.17 + row * 9.13 + v_seed * 41.0);
		float lit = step(0.42, h) * smoothstep(0.02, 0.5, night);
		vec3 warm = vec3(1.0, 0.58, 0.24);
		vec3 cool = vec3(0.62, 0.82, 1.0);
		float cool_mix = shop_floor ? 0.45 : step(0.8, h);
		vec3 interior = mix(warm, cool, cool_mix);
		float shop_lit = shop_floor ? night * 0.85 : 0.0;
		vec3 glass_col = mix(vec3(0.035, 0.045, 0.06), interior, max(lit, shop_lit));
		glass_col = mix(glass_col, vec3(0.40, 0.48, 0.58), (1.0 - night) * (1.0 - lit) * 0.35);
		albedo = mix(ceramic, upvc, frame);
		albedo = mix(albedo, glass_col, glass);
		rough = mix(0.86, 0.14, glass);
		float emit = glass * lit * (1.5 + night * 1.3);
		if (shop_floor) {
			emit = max(emit, glass * night * 0.9);
		}
		emission = interior * emit;
	}
	ALBEDO = albedo;
	ROUGHNESS = rough;
	METALLIC = 0.0;
	EMISSION = emission;
}
