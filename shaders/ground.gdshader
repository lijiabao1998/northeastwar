shader_type spatial;
render_mode diffuse_burley, cull_disabled;

varying vec3 v_world;

uniform sampler2D snow_alb : source_color, repeat_enable;
uniform sampler2D snow_nrm : hint_normal, repeat_enable;

void vertex() {
	v_world = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
}

void fragment() {
	vec2 uv = v_world.xz * 0.08;
	ALBEDO = texture(snow_alb, uv).rgb;
	NORMAL_MAP = texture(snow_nrm, uv).rgb;
	ROUGHNESS = 0.88;
	METALLIC = 0.0;
}
