shader_type spatial;
render_mode diffuse_burley, cull_disabled;

#include "res://shaders/cuts.gdshaderinc"

uniform float night : hint_range(0.0, 1.0) = 0.45;
uniform float floor_h = 3.0;
uniform float bay = 3.15;

varying vec3 v_color;
varying float v_seed;
varying float v_shop;
varying vec3 v_wnormal;

float hashf(float n) {
	return fract(sin(n) * 43758.5453);
}

void vertex() {
	v_color = COLOR.rgb;
	v_seed = UV2.x;
	v_shop = UV2.y;
	v_world = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
	v_wnormal = normalize((MODEL_MATRIX * vec4(NORMAL, 0.0)).xyz);
}

void fragment() {
	if (carved(v_world) || kit_hidden(v_world)) {
		discard;
	}
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

		// Northeast tile, not the raw vertex colour. Pinks in the source data
		// are pulled toward beige, cream, grey and faded yellow.
		vec3 northeast = v_color;
		if (v_color.r > v_color.b + 0.12 && v_color.r > 0.45) {
			northeast = mix(vec3(0.84, 0.78, 0.66), vec3(0.76, 0.66, 0.42), step(0.55, v_seed));
		} else if (v_color.r > 0.7 && v_color.g > 0.65) {
			northeast = mix(vec3(0.86, 0.84, 0.78), vec3(0.72, 0.73, 0.74), step(0.6, v_seed));
		}
		ceramic = mix(northeast, northeast * 0.62, grout);
		ceramic *= 0.94 + 0.08 * hashf(floor(tile.x) * 3.1 + floor(tile.y) * 9.7);

		float col = floor(u / bay);
		float row = floor(max(v, 0.0) / floor_h);
		vec2 cell = vec2(fract(u / bay), fract(max(v, 0.0) / floor_h));
		bool shop_floor = v_shop > 0.5 && row < 0.5;
		// A 3.15 m bay holds a ~1.5 m window: frame from 0.26 to 0.74 of the bay.
		float frame = 0.0;
		float glass = 0.0;
		float recess = 0.0;
		if (shop_floor) {
			float gx = smoothstep(0.18, 0.24, cell.x) * smoothstep(0.82, 0.76, cell.x);
			float gy = smoothstep(0.10, 0.16, cell.y) * smoothstep(0.62, 0.54, cell.y);
			glass = gx * gy;
			frame = smoothstep(0.12, 0.18, cell.x) * smoothstep(0.88, 0.82, cell.x);
			frame *= smoothstep(0.06, 0.10, cell.y) * smoothstep(0.70, 0.62, cell.y);
			frame *= (1.0 - glass);
		} else {
			float outer = smoothstep(0.26, 0.30, cell.x) * smoothstep(0.74, 0.70, cell.x);
			outer *= smoothstep(0.28, 0.34, cell.y) * smoothstep(0.84, 0.76, cell.y);
			float inner = smoothstep(0.33, 0.37, cell.x) * smoothstep(0.67, 0.63, cell.x);
			inner *= smoothstep(0.40, 0.46, cell.y) * smoothstep(0.76, 0.68, cell.y);
			float mullion = step(0.485, cell.x) * step(cell.x, 0.515);
			mullion = clamp(mullion, 0.0, 1.0);
			glass = inner * (1.0 - mullion);
			frame = clamp(outer - glass, 0.0, 1.0);
			recess = outer * 0.35;
		}

		vec3 upvc = vec3(0.93, 0.94, 0.92);
		float h = hashf(col * 17.17 + row * 9.13 + v_seed * 41.0);
		float lit = step(0.55, h) * smoothstep(0.15, 0.7, night);
		vec3 warm = vec3(1.0, 0.62, 0.28);
		vec3 interior = warm;
		float shop_lit = shop_floor ? night * 0.55 : 0.0;
		vec3 glass_col = mix(vec3(0.08, 0.1, 0.12), interior, max(lit, shop_lit) * 0.85);
		glass_col = mix(glass_col, vec3(0.55, 0.6, 0.66), (1.0 - night) * 0.25);
		albedo = mix(ceramic, ceramic * 0.55, recess);
		albedo = mix(albedo, upvc, frame);
		albedo = mix(albedo, glass_col, glass);
		rough = mix(0.88, 0.22, glass);
		float emit = glass * lit * (0.45 + night * 0.7);
		if (shop_floor) {
			emit = max(emit, glass * night * 0.35);
		}
		emission = interior * emit;
		// Enclosed balcony: a glass band and a security grille on alternate floors.
		if (!shop_floor && row > 0.5 && mod(row, 2.0) > 0.5) {
			float band = smoothstep(0.06, 0.14, cell.y) * smoothstep(0.46, 0.34, cell.y);
			float span = smoothstep(0.1, 0.18, cell.x) * smoothstep(0.9, 0.82, cell.x);
			float balcony = band * span;
			float bars = step(0.86, fract(cell.x * 16.0));
			albedo = mix(albedo, vec3(0.62, 0.7, 0.74), balcony * 0.7);
			albedo = mix(albedo, vec3(0.12, 0.13, 0.14), balcony * bars);
			rough = mix(rough, 0.28, balcony * 0.8);
			emission += vec3(0.35, 0.22, 0.1) * balcony * lit * 0.25;
		}
		// Snow caught on the sill.
		float sill = smoothstep(0.2, 0.28, cell.y) * smoothstep(0.38, 0.3, cell.y);
		sill *= smoothstep(0.2, 0.3, cell.x) * smoothstep(0.8, 0.7, cell.x);
		albedo = mix(albedo, vec3(0.9, 0.92, 0.94), sill);
		// Curtain behind some lit panes.
		float curtain = glass * step(0.42, fract(h * 2.7));
		vec3 curtain_col = mix(vec3(0.45, 0.16, 0.14), vec3(0.2, 0.28, 0.42), step(0.66, h));
		albedo = mix(albedo, curtain_col, curtain * max(lit, shop_lit) * 0.75);
		if (shop_floor && v_seed > 0.62) {
			float shut = smoothstep(0.08, 0.5, cell.y);
			float slat = step(0.55, fract(cell.y * 22.0));
			albedo = mix(albedo, vec3(0.62, 0.63, 0.64), shut * 0.9);
			albedo = mix(albedo, vec3(0.4, 0.41, 0.42), shut * slat);
			emission *= (1.0 - shut);
		}
		float ac = step(0.8, cell.y) * step(cell.y, 0.96) * step(0.36, cell.x) * step(cell.x, 0.64);
		ac *= step(0.45, fract(col * 0.37 + v_seed));
		albedo = mix(albedo, vec3(0.75, 0.77, 0.78), ac);
		rough = mix(0.95, rough, 1.0 - grout * 0.85);
		float bump = grout + sill;
		NORMAL = normalize(NORMAL + vec3(dFdx(bump), dFdy(bump), 0.0) * 1.6);
	}
	if (v_wnormal.y > 0.45) {
		albedo = mix(albedo, vec3(0.93, 0.94, 0.96), smoothstep(0.45, 0.85, v_wnormal.y));
		rough = mix(rough, 0.82, v_wnormal.y);
	}
	if (crack > 0.02) {
		float dist = length(v_world.xz - crack_at.xz);
		float reach = smoothstep(18.0 * crack + 4.0, 0.4, dist);
		float fissure = smoothstep(0.86, 0.98, abs(sin(v_world.x * 5.5 + v_world.y * 8.0 + v_world.z * 2.0)));
		float branch = smoothstep(0.9, 0.99, abs(sin(v_world.y * 3.0 - v_world.x * 7.0)));
		albedo = mix(albedo, albedo * 0.45, reach * 0.55);
		albedo = mix(albedo, vec3(0.05, 0.045, 0.04), reach * max(fissure, branch));
		rough = mix(rough, 1.0, reach);
	}
	ALBEDO = albedo;
	ROUGHNESS = rough;
	METALLIC = 0.0;
	EMISSION = emission;
}
