shader_type spatial;
render_mode unshaded, cull_disabled, blend_mix, depth_draw_never, shadows_disabled;

void fragment() {
	vec2 p = UV * 2.0 - vec2(1.0);
	float d = length(p);
	ALBEDO = vec3(0.96, 0.97, 1.0);
	ALPHA = smoothstep(1.0, 0.35, d) * 0.75;
}
