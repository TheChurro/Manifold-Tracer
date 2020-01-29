#define EPSILON 0.000001f
#define TRIANGLE 0
#define BALL 1
#define LAMBERTIAN 0
#define EMISSIVE 1

// According to recommendations from Parker and Miller
// http://www0.cs.ucl.ac.uk/staff/ucacbbl/ftp/papers/langdon_2009_CIGPU.pdf
uint rand_int(uint* seed) {
  uint const a = 16807; //ie 7**5
  uint const m = 2147483647; //ie 2**31-1
  *seed = (long(seed * a))%m;
  return *seed;
}

float rand_float(uint* seed) {
  int i = rand_int(seed);
  return i / 2147483647.0f;
}

// Generate a random point in the unit hemisphere (x dir is up)
float3 rand_hemisphere(uint* seed) {
  float cos_theta = rand_float(seed);
  float sin_theta = sqrt(fmax(0.0f, 1 - cos_theta * cos_theta));
  float phi = 2 * M_PI_F * rand_float(seed);
  return (float3)(cos_theta, sin_theta * cos(phi), sin_theta * sin(phi));
}

float det(float3 r0, float3 r1, float3 r2) {
  return r0.x * (r1.y * r2.z - r1.z * r2.y)
       - r0.y * (r1.x * r2.z - r1.z * r2.x)
       + r0.z * (r1.x * r2.y - r1.y * r2.x);
}

float4 cross_three(float4 a, float4 b, float4 c) {
  float x = det(a.yzw, b.yzw, c.yzw);
  float y = det(a.xzw, b.xzw, c.xzw);
  float z = det(a.xyw, b.xyw, c.xyw);
  float w = det(a.xyz, b.xyz, c.xyz);
  return (float4)(-x, y, -z, w);
}

void get_basis(
  float4 point,
  float4 normal,
  float4* out_left,
  float4* out_forwards
) {
  for (int i = 0; i < 4; i++) {
    float4 v = (float4)(
      i == 0 ? 1.0 : 0.0,
      i == 1 ? 1.0 : 0.0,
      i == 2 ? 1.0 : 0.0,
      i == 3 ? 1.0 : 0.0,
    );
    float p_dot = dot(v, point);
    float n_dot = dot(v, normal);
    float4 v_perp = v - p_dot * point - n_dot * normal;
    float v_length = length(v_perp);
    if (left_length > EPSILON) {
      *out_left = v_perp / v_length;
      break;
    }
  }
  float4 forwards = cross_three(point, normal, *out_left);
  *out_forwards = forwards / length(forwards);
}

__kernel void shade(
  write_only image2d_t out_image,
  __global  float4* ray_locations,
  __global  float4* ray_tangents,
  __global  float4* ray_colors,
  __global  float4* hit_normal,
  __global  uint4*  hit_infos,
  __private const int num_hits_in,
  __global        int num_rays_out,
  __global const float4* tri_material_colors,
  __global const int*    tri_material_type,
  __global const float4* ball_material_colors,
  __global const int*    ball_material_type
) {
  uint global_address = get_global_id(1) + get_global_id(0) * get_global_size(1);
  if (global_address >= num_rays_in) return;
  uint4 hit_info = hit_infos[global_address];
  uint hit_type = hit_info.x;
  uint hit_index = hit_info.y;
  float4 mat_color = (float4)(0);
  uint mat_type = 0;
  switch (hit_type) {
    // hit a triangle
    case TRIANGLE:
      mat_color = tri_material_colors[hit_index];
      mat_type = tri_material_type[hit_index];
      break;
    case BALL:
      mat_color = ball_material_colors[hit_index];
      mat_type = ball_material_type[hit_index];
      break;
    default:
      write_imagef(
        out_image,
        hit_info.zw,
        (float4)(0.0f, 0.0f, 0.0f, 1.0f)
      );
      return;
  }
  float4 ray_color = ray_colors[global_address];
  // Hit emitter! Change the color
  if (mat_type == EMISSIVE) {
    write_imagef(
      out_image,
      hit_info.zw,
      (float4)(fmin(ray_color * mat_color, 1.0f), 1.0f)
    );
    return;
  }
  // Otherwise we hit a lambertian material
  float4 origin = ray_locations[global_address];
  float4 tangent = ray_tangents[global_address];
  float4 normal = hit_normal[global_address];
  uint seed = global_address ^ as_uint(ray_color.x) ^ as_uint(tangent.y) ^ as_uint(origin.z);
  float3 sampled_point = rand_hemisphere(&seed);
  float4 right;
  float4 forwards;
  get_basis(origin, normal, &right, &forwards);
  uint new_ray_idx = atomic_add(&num_rays_out, 1);
  ray_locations[new_ray_idx] = origin;
  ray_tangents[new_ray_idx] = normal * sampled_point.x
                             + right * sampled_point.y
                          + forwards * sampled_point.z;
  ray_colors[new_ray_idx] = mat_color * ray_color;
  hit_infos[new_ray_idx] = hit_info;
}
