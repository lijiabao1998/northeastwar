// Shared demolition cuts. One slot per failed building section.
// Named uniforms, not arrays: Godot 4.7 drops a vec3[N] block and then
// keeps the last shader that compiled, so the wall never changes.

uniform float cuts = 0.0;
uniform vec3 a0 = vec3(0.0);
uniform vec3 b0 = vec3(0.0);
uniform vec3 a1 = vec3(0.0);
uniform vec3 b1 = vec3(0.0);
uniform vec3 a2 = vec3(0.0);
uniform vec3 b2 = vec3(0.0);
uniform vec3 a3 = vec3(0.0);
uniform vec3 b3 = vec3(0.0);
uniform vec3 a4 = vec3(0.0);
uniform vec3 b4 = vec3(0.0);
uniform vec3 a5 = vec3(0.0);
uniform vec3 b5 = vec3(0.0);
uniform vec3 a6 = vec3(0.0);
uniform vec3 b6 = vec3(0.0);
uniform vec3 a7 = vec3(0.0);
uniform vec3 b7 = vec3(0.0);
uniform vec3 crack_at = vec3(0.0, 8.0, 0.0);
uniform float crack = 0.0;

varying vec3 v_world;

bool inside_box(vec3 p, vec3 mn, vec3 mx) {
	return p.x >= mn.x && p.x <= mx.x && p.y >= mn.y && p.y <= mx.y && p.z >= mn.z && p.z <= mx.z;
}

bool carved(vec3 p) {
	if (cuts > 0.5 && inside_box(p, a0, b0)) return true;
	if (cuts > 1.5 && inside_box(p, a1, b1)) return true;
	if (cuts > 2.5 && inside_box(p, a2, b2)) return true;
	if (cuts > 3.5 && inside_box(p, a3, b3)) return true;
	if (cuts > 4.5 && inside_box(p, a4, b4)) return true;
	if (cuts > 5.5 && inside_box(p, a5, b5)) return true;
	if (cuts > 6.5 && inside_box(p, a6, b6)) return true;
	if (cuts > 7.5 && inside_box(p, a7, b7)) return true;
	return false;
}
