#define EPSILON 0.00001f
#define TRIANGLE 0
#define BALL 1
#define NONE 2

#define ENTRYPOINT_SENTINEL 0x76543210
#define TRAVERSAL_STACK_SIZE 63
#define TRAVERSAL_BRANCH 0
#define TRAVERSAL_LEAF 1

bool hits_ball(
  float4 origin,
  float4 tangent,
  float4 center,
  float radius,
  bool zero_if_inside,
  float* hit_angle
);
bool hits_ball(
  float4 origin,
  float4 tangent,
  float4 center,
  float radius,
  bool zero_if_inside,
  float* hit_angle
) {
  float r = cos(radius);
  float center_origin = dot(center, origin);
  if (zero_if_inside && center_origin > r) {
    *hit_angle = 0;
    return true;
  }
  float center_tangent = dot(center, tangent);
  float theta = 10000;
  bool swap_y = false;
  if (fabs(center_tangent * center_origin) < EPSILON * EPSILON) {
    if (zero_if_inside) *hit_angle = 0;
    return zero_if_inside;
  }
  if (fabs(center_tangent) < fabs(center_origin)) {
    swap_y = true;
    float tmp = center_tangent;
    center_tangent = center_origin;
    center_origin = tmp;
  }
  float inv_ct_sq = 1 / (center_tangent * center_tangent);
  float a = 1 + center_origin * center_origin * inv_ct_sq;
  float b = -2 * r * center_origin * inv_ct_sq;
  float c = r * r * inv_ct_sq - 1;
  float descriminant = b * b - 4 * a * c;
  if (descriminant < 0) {
    return false;
  }
  float descriminant_sqrt = sqrt(descriminant);
  float x0 = (-b + descriminant_sqrt) / (2 * a);
  float y0 = (r - center_origin * x0) / center_tangent;
  float x1 = (-b - descriminant_sqrt) / (2 * a);
  float y1 = (r - center_origin * x1) / center_tangent;

  if (swap_y) {
    float tmp = x0;
    x0 = y0;
    y0 = tmp;
    tmp = x1;
    x1 = y1;
    y1 = x1;
  }

  float theta1 = atan2(y1, x1);
  float theta0 = atan2(y0, x0);
  if (fabs(x0) > 1 || fabs(y0) > 1) theta0 = 1000000;
  if (theta0 < EPSILON) theta0 += 2 * M_PI_F;
  if (fabs(x1) > 1 || fabs(y1) > 1) theta1 = 1000000;
  if (theta1 < EPSILON) theta1 += 2 * M_PI_F;
  *hit_angle = fmin(theta0, theta1);
  return *hit_angle != 1000000;
}

__kernel void trace(
  __global float4* ray_origin_in,
  __global float4* ray_tangent_in,
  __global float4* ray_color_in,
  __global uint4*  ray_info_in,
  __private const uint rays_in,

  __global float4* ray_origin_out,
  __global float4* ray_tangent_out,
  __global float4* ray_color_out,
  __global uint4*  ray_info_out,
  __global float4* hit_normals,

  __global const float4* edge_ab_normals,
  __global const float4* edge_bc_normals,
  __global const float4* edge_ca_normals,
  __global const float4* normals,
  __private const uint num_triangles,

  __global const float4* ball_centers,
  __global const float*  ball_radii,
  __private const uint num_balls,

  __global const float4* hierarchy_ball_centers,
  __global const float*  hierarchy_ball_radii,
  __global const uint4*  tri_hierarchy_data
) {
  uint global_address = get_global_id(0);
  uint4 ray_info = ray_info_in[global_address];
  if (global_address >= (uint)rays_in) return;
  float4 origin = ray_origin_in[global_address];
  float4 tangent = ray_tangent_in[global_address];
  float hit_angle = 100000000;
  float4 hit_normal = (float4)(0, 0, 0, 0);
  bool was_hit = false;
  int hit_index = -1;
  int hit_type = -1;

  uint traversal_stack[TRAVERSAL_STACK_SIZE];
  traversal_stack[0] = ENTRYPOINT_SENTINEL;
  int traversal_ptr = 0;
  uint node_address = 0;
  uint next_tri = 0;
  uint last_tri = 0;

  while (
    traversal_ptr < TRAVERSAL_STACK_SIZE &&
    (node_address != ENTRYPOINT_SENTINEL || next_tri < last_tri)
  ) {
    // If we have a triangle to test, then we test that triangle and move to
    // the next triangle
    if (next_tri < last_tri) {
      float4 normal = normals[next_tri];
      float o_norm = -dot(normal, origin);
      float t_norm = dot(normal, tangent);
      if (fabs(o_norm) > EPSILON || fabs(t_norm) > EPSILON) {
        float4 scaled_hit_pos = origin * t_norm + tangent * o_norm;
        float hit_ab = dot(scaled_hit_pos, edge_ab_normals[next_tri]);
        int sign_ab = hit_ab < -EPSILON ? -1 : hit_ab <= EPSILON ? 0 : 1;
        float hit_bc = dot(scaled_hit_pos, edge_bc_normals[next_tri]);
        int sign_bc = hit_bc < -EPSILON ? -1 : hit_bc <= EPSILON ? 0 : 1;
        if (sign_ab == 0 || sign_bc == 0 || sign_ab == sign_bc) {
          float hit_ca = dot(scaled_hit_pos, edge_ca_normals[next_tri]);
          int sign_ca = hit_ca < -EPSILON ? -1 : hit_ca <= EPSILON ? 0 : 1;
          if ((sign_bc == 0 && (sign_ab == 0 || sign_ab == sign_ca)) || sign_ca == 0 || sign_bc == sign_ca) {
            bool second_hit = hit_ab + hit_bc + hit_ca <= 0;
            float angle = atan2(o_norm, t_norm);
            if (second_hit) {
              angle += M_PI_F;
            }
            if (angle < EPSILON) {
              angle += 2 * M_PI_F;
            }

            if (angle < hit_angle) {
              was_hit = true;
              hit_angle = angle;
              hit_index = next_tri;
              hit_type = TRIANGLE;
              hit_normal = normal;
            }
          }
        }
      }
      next_tri++;
      // if (next_tri == last_tri) {
      //   node_address = traversal_stack[traversal_ptr];
      //   traversal_ptr--;
      // }
    } else {
      uint4  data   = tri_hierarchy_data[node_address];
      float  radius = hierarchy_ball_radii[node_address];
      float4 center = hierarchy_ball_centers[node_address];
      float  node_hit_angle = 0.0;
      bool   in_hierarchy = hits_ball(origin, tangent, center, radius, true, &node_hit_angle);
      if (in_hierarchy) {
        if (data.x == TRAVERSAL_BRANCH) {
          // Move down to the left node.
          node_address = data.y;
          if (data.z != 0) {
            // Write to our stack that we need to go down the right node
            // as well.
            traversal_ptr++;
            traversal_stack[traversal_ptr] = data.z;
          }
        } else {
          next_tri = data.y;
          last_tri = data.z;
          node_address = traversal_stack[traversal_ptr];
          traversal_ptr--;
        }
      } else {
        node_address = traversal_stack[traversal_ptr];
        traversal_ptr--;
      }
    }
  }

  for (uint i = 0; i < num_balls; i++) {
    float4 center = ball_centers[i];
    float radius = ball_radii[i];
    float theta;
    if (hits_ball(origin, tangent, center, radius, false, &theta)) {
      if (theta < hit_angle) {
        hit_angle = theta;
        float4 hit_point = origin * cos(theta) + tangent * sin(theta);
        hit_index = i;
        hit_type = BALL;
        hit_normal = normalize(-center + hit_point * dot(center, hit_point));
        was_hit = true;
      }
    }
  }

  if (was_hit) {
    hit_normals[global_address] = hit_normal;
    float cos_angle = cos(hit_angle);
    float sin_angle = sin(hit_angle);
    ray_origin_out[global_address] = origin * cos_angle + tangent * sin_angle;
    ray_tangent_out[global_address] = -origin * sin_angle + tangent * cos_angle;
    ray_color_out[global_address] = ray_color_in[global_address];
    ray_info_out[global_address] = (uint4)(hit_type, hit_index, ray_info.zw);
  } else {
    ray_info_out[global_address] = (uint4)(NONE, NONE, ray_info.zw);
  }
}
