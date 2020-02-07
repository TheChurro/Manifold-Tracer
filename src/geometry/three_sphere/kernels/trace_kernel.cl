#define EPSILON 0.000001f
#define TRIANGLE 0
#define BALL 1

__kernel void trace(
  write_only image2d_t out_image,
  __global const float4* origins,
  __global const float4* tangents,
  __global const float4* edge_ab_normals,
  __global const float4* edge_bc_normals,
  __global const float4* edge_ca_normals,
  __global const float4* normals,
  __private const int num_triangles,
  __global const float4* ball_centers,
  __global const float*  ball_radii,
  __private const int num_balls
) {
  uint global_address = get_global_id(1) + get_global_id(0) * get_global_size(1);
  float4 origin = origins[global_address];
  float4 tangent = tangents[global_address];
  int2 pixel = (int2)(get_global_id(0), get_global_id(1));
  float hit_angle = 4 * M_PI_F;
  float4 hit_normal = (float4)(0, 0, 0, 0);
  bool was_hit = false;
  int hit_index = -1;
  int hit_type = -1;
  for (int i = 0; i < num_triangles; i++) {
    float4 normal = normals[i];
    float o_norm = -dot(normal, origin);
    float t_norm = dot(normal, tangent);
    // If we are moving perpendicular to the sphere of the triangle,
    // then we do not consider anything a "hit."
    if (fabs(o_norm) < EPSILON && fabs(t_norm) < EPSILON) continue;
    float angle = atan2(o_norm, t_norm);
    // Verify that we move forwards from our initial position.
    if (angle <= 0) {
      angle += M_PI_F;
    }
    for (int k = 0; k < 2; k++) {
      if (angle < hit_angle) {
        float4 hit_pos = origin * cos(angle) + tangent * sin(angle);
        angle += M_PI_F;
        // Advance to next hit (antipodal point.)
        float4 edge_ab_normal = edge_ab_normals[i];
        if (dot(hit_pos, edge_ab_normal) < 0) continue;
        float4 edge_bc_normal = edge_bc_normals[i];
        if (dot(hit_pos, edge_bc_normal) < 0) continue;
        float4 edge_ca_normal = edge_ca_normals[i];
        if (dot(hit_pos, edge_ca_normal) < 0) continue;
        hit_angle = angle - M_PI_F;
        was_hit = true;
        hit_index = i;
        hit_type = TRIANGLE;
        hit_normal = normal;
        break;
      } else {
        break;
      }
    }
  }

  for (int i = 0; i < num_balls; i++) {
    float4 center = ball_centers[i];
    float radius = ball_radii[i];
    float r = cos(radius);
    float center_origin = dot(center, origin);
    float center_tangent = dot(center, tangent);
    float theta = 10000;
    // If the origin is perpendicular to the ball.
    if (fabs(center_origin) < EPSILON) {
      if (fabs(center_tangent) < EPSILON) {
        continue;
      }
      float sin_theta = r / center_tangent;
      if (fabs(sin_theta) > 1) continue;
      theta = asin(sin_theta);
      if (theta < 0) {
        theta = M_PI_F - theta;
      }
    }
    // If the tangent is perpendicular to the ball.
    else if (fabs(center_tangent) < EPSILON) {
      float cos_theta = r / center_origin;
      if (fabs(cos_theta) > 1) continue;
      theta = acos(cos_theta);
    } else {
      //   x^2 + ((r - co x) / ct)^2 - 1
      // = (1 + co^2 / ct^2) x^2 - 2 * r * co x / ct^2 + (r^2 / ct^2 - 1)
      // x = (2 r co / ct^2 ± sqrt(4 r^2 co^2 / ct^4 - 4(1 + co^2/ct^2)(r^2 / ct^2 - 1)))/(2 + 2co^2/ct^2)
      float inv_ct_sq = 1 / (center_tangent * center_tangent);
      float a = 1 + center_origin * center_origin * inv_ct_sq;
      float b = -2 * r * center_origin * inv_ct_sq;
      float c = r * r * inv_ct_sq - 1;
      float descriminant = b * b - 4 * a * c;
      if (descriminant < 0) {
        continue;
      }
      float descriminant_sqrt = sqrt(descriminant);
      float x0 = (-b + descriminant_sqrt) / (2 * a);
      // co x + ct y = r
      // ct y = r - co x
      // y = (r - co x) / ct;
      float y0 = (r - center_origin * x0) / center_tangent;
      float theta0 = atan2(y0, x0);
      if (fabs(x0) > 1) theta0 = 1000000;
      if (theta0 < 0) theta0 += 2 * M_PI_F;
      float x1 = (-b - descriminant_sqrt) / (2 * a);
      float y1 = (r - center_origin * x1) / center_tangent;
      float theta1 = atan2(y1, x1);
      if (fabs(x1) > 1) theta1 = 1000000;
      if (theta1 < 0) theta1 += 2 * M_PI_F;
      theta = fmin(theta0, theta1);
    }
    if (theta < hit_angle) {
      hit_angle = theta;
      float4 hit_point = origin * cos(theta) + tangent * sin(theta);
      hit_index = i;
      hit_type = BALL;
      hit_normal = normalize(hit_point - center * dot(center, hit_point));
      was_hit = true;
    }
  }

  if (was_hit) {
    float3 color_mask = (float3)(
      (hit_index % 2) == 0 ? 1.0 : (hit_type == TRIANGLE ? 0.5 : 0.25),
      (hit_index % 4)  < 2 ? 1.0 : (hit_type == TRIANGLE ? 0.5 : 0.25),
      (hit_index % 8)  < 4 ? 1.0 : (hit_type == TRIANGLE ? 0.5 : 0.25)
    );
    write_imagef(
      out_image,
      pixel,
      (float4)(color_mask, 1.0f)
    );
  } else {
    write_imagef(
      out_image,
      pixel,
      (float4)(0.0f, 0.0f, 0.0f, 1.0f)
    );
  }
}
