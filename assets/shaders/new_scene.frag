out vec4 FragColor;

in vec2 TexCoords;

uniform float time;

#define EPSILON 0.1

float sphere(vec3 p, vec3 pos, float r) {
	return length(p - pos) - r;
}

vec3 opRep(vec3 p, vec3 c) {
	vec3 q = mod(p, c) - 0.5*c;
	return q;
}

float plane(vec3 p, vec4 n) {
	return dot(p,n.xyz) + n.w;
}

float dst(vec3 p) {
	// return (min(mountains(p), water(p)));
	return min(
		sphere(opRep(p, vec3(20.0 + sin(time / 200.0) * 0.5, 20.0 + sin(time / 50.0) * 5, 40.0)), vec3(0.0, 0.0, 20.0), 5),
		// plane(p, vec4(0.0, 0.0, 10.0, 1000.0))
		10000000
	);
}

vec3 estimateNormal(vec3 p) {
    return normalize(vec3(
        dst(vec3(p.x + EPSILON, p.y, p.z)) - dst(vec3(p.x - EPSILON, p.y, p.z)),
        dst(vec3(p.x, p.y + EPSILON, p.z)) - dst(vec3(p.x, p.y - EPSILON, p.z)),
        dst(vec3(p.x, p.y, p.z  + EPSILON)) - dst(vec3(p.x, p.y, p.z - EPSILON))
    ));
}

void main() {
	vec3 c;

	vec3 pp = vec3(time / 20.0, 0.0, 0.0);
	vec3 p = pp;
	// p.x = -time / 50.1;
	p.z = time / 1.5;
	// p.y = sin(time / 200.0) * 10.0 + 60.0;
	vec3 d = normalize(vec3((TexCoords.xy - vec2(0.5)) * 3.0, 1.0));
	d.y -= 0.6;
	d *= vec3(sign(d.x), 1.0, 1.0);
	d.x += sin(time / 100.0) * 0.1 + 0.1;
	d = normalize(d);

	const int D = 70;

	float r = 0.0;

	float v;
	vec3 t;

	float a = 1.0;

	for (int i = 0; i < D; i += 1) {
		t = p + r * d;
		float dd = dst(t);
		r += dd;
		if (dd < 0.1) {
			// v = float(D - i) / float(D);
			p = t;
			r = 0.2;
			d = estimateNormal(p);
			v += 1.0 / float(i * 0.01) / pow(r * 50.0, 1.0);
		}
	}

	vec3 j = vec3(0.1, 0.2, 0.5);
	vec3 l = vec3(0.0, 0.9, 0.8);
	vec3 k = vec3(1.0, 0.0, 0.0);

	c = vec3(0.0);
	c += j * abs(0.0 - v);
	c += k * abs(0.5 - v);
	c += l * abs(1.0 - v);

	c *= vec3(pow(t.y / 10.0, 1.0/10.0));

	// c = vec3(pow(v/1.0, 4.0));

	c = pow(c, vec3(1/2.2));
	// c = vec3(v);

	FragColor = vec4(c, 1.0);
}
