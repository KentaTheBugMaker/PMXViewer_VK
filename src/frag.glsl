#version 450

layout(location = 0) in vec4 v_normal;
layout(location = 1) in vec2 v_tex_coord;
layout(location = 2) in vec3 v_position;
layout(location = 0) out vec4 f_color;
layout(set = 0, binding=1)uniform sampler2D tex;
layout(set=0, binding=2)uniform Image{
    float w;
    float h;
} image;
//マテリアルごとのデータ
layout(set=1, binding=0)uniform Material{
    vec4 diffuse;//diffuse color
    vec4 edge_color;//
    vec3 specular;//specular color
    float specular_intensity;//
    vec3 ambient;//ambient color
    bool edge_flag;//enable Edge_rendering
//int sp;//SphereMode
} materials;
vec3 u_light=vec3(1.4, 0.4, -0.7);
void main() {
    vec3 diffuse_color=materials.diffuse.rgb;
    vec3 ambient_color=materials.ambient;
    vec3 specular_color=materials.specular;
    vec3 v_norm=v_normal.xyz;
    float diffuse = max(dot(normalize(v_norm), normalize(u_light)), 0.0);
    vec3 camera_dir = normalize(-v_position);
    vec3 half_direction = normalize(normalize(u_light) + camera_dir);
    float specular = pow(max(dot(half_direction, normalize(v_norm)), 0.0), materials.specular_intensity);
    vec4 color = vec4(ambient_color + diffuse * diffuse_color + specular * specular_color, 1.0);

    f_color=texture(tex, v_tex_coord)*color;
}
