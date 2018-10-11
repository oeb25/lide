layout (points) in;
layout (triangle_strip, max_vertices = 6) out;

out vec2 TexCoords;

const vec2[] RectCoords = vec2[](
	vec2(0, 0),
	vec2(0, 1),
	vec2(1, 1),
	vec2(1, 0)
);

void e(int i) {
	vec2 p = RectCoords[i];
	TexCoords = p;
	gl_Position = vec4((p - vec2(0.5)) * 2, 0.0, 1.0);
	EmitVertex();
}

void main() {
	e(0);
	e(1);
	e(2);
	EndPrimitive();
	e(0);
	e(2);
	e(3);
	EndPrimitive();
}