#include "./lights.glsl"

out vec4 FragColor;

in vec2 TexCoords;

uniform float time;
uniform sampler2D scene;

uniform float weight[5] = float[] (0.227027, 0.1945946, 0.1216216, 0.054054, 0.016216);

void main() {
	vec3 c;

    vec2 tex_offset = 1.0 / textureSize(scene, 0) / 2.0; // gets size of single texel
    c = texture(scene, TexCoords).rgb * weight[0]; // current fragment's contribution
    if(true)
    {
        for(int i = 1; i < 5; ++i)
        {
            c += texture(scene, TexCoords + vec2(tex_offset.x * i, 0.0)).rgb * weight[i];
            c += texture(scene, TexCoords - vec2(tex_offset.x * i, 0.0)).rgb * weight[i];
        }
    }
    else
    {
        for(int i = 1; i < 5; ++i)
        {
            c += texture(scene, TexCoords + vec2(0.0, tex_offset.y * i)).rgb * weight[i];
            c += texture(scene, TexCoords - vec2(0.0, tex_offset.y * i)).rgb * weight[i];
        }
    }
	// c = texture(scene, TexCoords).rgb;

	FragColor = vec4(c, 1.0);
}
