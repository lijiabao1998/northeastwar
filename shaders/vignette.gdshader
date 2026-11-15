shader_type canvas_item;

uniform float amount : hint_range(0.0, 1.0) = 0.0;

void fragment() {
	vec2 p = UV - vec2(0.5);
	float v = smoothstep(0.28, 0.78, length(p * vec2(1.15, 1.35)));
	COLOR = vec4(0.02, 0.08, 0.16, v * amount);
}
