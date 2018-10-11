#include "./lights.glsl"

out vec4 FragColor;

in vec2 TexCoords;

uniform float time;

float balls(vec3 p) {
	float rep = 11.0;
	p.x += floor(p.z / 20) * 2.5;
	float x = mod(p.x, rep);
	float z = mod(p.z, rep);
	float xx = sin(floor(p.x / rep) + time / 10.0) * 5.0;
	float zz = cos(floor(p.z / rep) + time / 12.0) * 6.0;

	return length(vec3(x, p.y + xx + zz, z) - vec3(rep / 2.0, 0.0, 10.0)) - 5.0;
}

float water(vec3 p) {
	float j =
		sin(p.x / 2.0) *
		sin(p.z / 3.0 + 4.0) *
		sin(p.z / 2.0 + p.x / 3.0 + 4.0);
	float k =
		sin(p.x / 20.3) *
		sin(p.z / 20.3 + 4.0) *
		sin(p.z / 2.0 + p.x / 3.0 + 4.0);

	return p.y -
		(sin(p.x / 30.0 - 100.0) * sin(p.z / 100.0 + time / 10)) * 2.0 -
		sin(p.z * 0.2 + cos(p.x * 0.2) + time / 20) * 2.0 -
		sin(p.x / 50.0 + time * 0.02 + cos(p.z / 200.0) + p.z * 0.02) * 2.0 +
		(pow(-2, abs(pow(j, 1.0/2.0))) + 2) * 3.0 -
		(pow(-2, abs(pow(k, 1.0/2.0))) + 2) * 7.0;
}

float mountains(vec3 p) {
	return p.y -
		(sin(p.x / 20.0 + 10.0) * sin(p.z / 10.0) + cos(p.z / 30.0)) * 15.0 -
		sin(p.z * 0.02 + cos(p.x * 0.2)) * 5.0 +
		sin(p.x / 500.0 + cos(p.z / 200.0)) * 12.0;
}

float dst(vec3 p) {
	return (min(mountains(p), water(p)));
	// return min(min(mountains(p), water(p)), balls(p - vec3(0.0, 100.0, 0.0)));
}

void main() {
	vec3 c;

	vec3 p = vec3(0.0);
	p.x = -time / 50.1;
	p.z = time / 1.5;
	p.y = sin(time / 200.0) * 10.0 + 60.0;
	vec3 d = normalize(vec3((TexCoords.xy - vec2(0.5)) * 3.0, 1.0));
	d.y -= 0.6;
	// d *= vec3(-sign(d.x), 1.0, 1.0);
	d.x += sin(time / 100.0) * 0.1 + 0.1;
	d = normalize(d);

	const int D = 100;

	float r = 0.0;

	float v;
	vec3 t;

	for (int i = 0; i < D; i += 1) {
		t = p + r * d;
		float dd = dst(t);
		r += dd;
		if (dd < 0.01) {
			v = float(D - i) / float(D);
			v /= pow(r / 25.0, 1.0);
			break;
		}
	}

	vec3 j = vec3(0.1, 0.2, 0.5);
	vec3 l = vec3(0.0, 0.9, 0.8);
	vec3 k = vec3(1.0, 0.0, 0.0);

	c = vec3(0.0);
	c += j * abs(0.0 - v);
	c += k * abs(0.5 - v);
	c += l * abs(1.0 - v);

	// c *= vec3(pow(t.y / 10.0, 1.0/10.0));

	// c = vec3(pow(v/1.0, 4.0));

	c = pow(c, vec3(1/1.2));
	// c = vec3(v);

	FragColor = vec4(c, 1.0);
}
