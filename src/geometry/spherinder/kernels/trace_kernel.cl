typedef struct RayHit {
  float4 pos;
  float4 normal;
  float3 uvw;
  float  t;
  int    tri_coord;
} RayHit;

#define EPSILON 0.00000001f

typedef struct Ray {
  float4 origin;
  float4 direction;
  float  s_geodesic_rate;
  float3 s_geodesic_normal;
  float3 s_geodesic_start;
  float3 s_geodesic_end;
} Ray;

typedef struct GreatArcIntersection {
  bool same_arc;
  bool vertical;
  bool intersected;
  float4 position;
  float4 segment_end;
} GreatArcIntersection;

float4 mul_quat(float4 a, float4 b);
float3 rotate_point(float3 p, float4 q);
void init_ray(float4 origin, float4 direction, Ray* ray);
bool intersect(Ray* ray, float4 triA, float4 triB, float4 triC, RayHit* hit);
void intersect_great_arc(Ray* ray, float4 start, float4 end, GreatArcIntersection* intersection);

float4 mul_quat(float4 a, float4 b) {
  return float4(
    a.x * b.x - a.y * b.y - a.z * b.z - a.w * b.w,
    a.x * b.y + a.y * b.x + a.z * b.w - a.w * b.z,
    a.x * b.z + a.z * b.x - a.y * b.w + a.w * b.y,
    a.x * b.w + a.w * b.x + a.y * b.z - a.z * b.y
  );
}

float3 rotate_point(float3 p, float4 q) {
  float4 inverse = (float4)(q.x, -q.yzw) / length(q);
  float4 p_quat = (float4)(0, p);
  float4 p_rot_quat = mul_quat(q, mul_quat(p_quat, inverse));
  return p_rot_quat.yzw;
}

void init_ray(float4 origin, float4 direction, Ray* ray) {
  ray->origin = origin;
  ray->direction = direction;
  float3 tangent_movement = normalize(direction.xyz);
  ray->s_geodesic_rate = length(direction.xyz);
  ray->s_geodesic_start = origin.xyz;
  ray->s_geodesic_normal = normalize(cross(ray->s_geodesic_start, tangent_movement));
  ray->s_geodesic_end = tangent_movement;
}

// ASSUMPTION: This function always chooses "start" as position if it is a choice.
void intersect_great_arc(Ray* ray, float4 start, float4 end, GreatArcIntersection* intersection) {
  intersection->intersected = false;
  intersection->vertical = false;
  intersection->same_arc = false;

  // Compute whether the start and end are on the same sphere position. This
  // is equivalent to a vertical line. This will almost always result in a
  // triangle in cylinder space. The great arc here is the zero geodesic really...
  if (length(start.xyz - end.xyz) < EPSILON) {
    if (fabs(dot(start.xyz, ray->s_geodesic_normal)) < EPSILON) {
      intersection->position = start;
      intersection->vertical = true;
      intersection->segment_end = end;
      intersection->intersected = true;
      return;
    }
    return;
  }

  float3 arc_normal = normalize(cross(start.xyz, end.xyz));
  float3 p_intersect = 0;
  bool check_antipode = true;

  // If the ray does not move along a circle, that is, only moves in the w direction,
  // then return true only if the ray origin.xyz lies along the start to end circle.
  // We do need to compute at what point that angle is along the circle, however.
  if (ray->s_geodesic_rate < EPSILON) {
    p_intersect = ray->origin.xyz;
    if (dot(arc_normal, p_intersect) > EPSILON) return;
    check_antipode = false;
  } else {
    // Otherwise, our ray will need to cross a specific part of a great circle.
    // Compute the points of intersection between our ray circle and the supplied
    // circle (circle centered on the origin going through start and end).
    // This point must be perpindicular to the normals of **both** circles. The
    // only points that satisfiy this are proprotional to the cross product of the
    // normals.
    float3 line_of_intersection = cross(ray->s_geodesic_normal, arc_normal);
    float length_of_line_of_intersection = length(line_of_intersection);

    // The cross product is zero so the circles overlap. Return the start as
    // assumed by this function.
    if (length_of_line_of_intersection < EPSILON) {
      intersection->position = start;
      intersection->same_arc = true;
      intersection->segment_end = end;
      intersection->intersected = true;
      return;
    }
    p_intersect = line_of_intersection / length_of_line_of_intersection;
  }

  // Now we have one point of intersection on the sphere between the two circles.
  // That is, the normalized cross product of the norms. However, we don't know
  // if this lies along the shortest arc between start and end.
  float  min_cos = dot(start.xyz, end.xyz);
  float  start_intersect_cos = dot(start.xyz, p_intersect.xyz);
  float  end_intersect_cos = dot(end.xyz, p_intersect.xyz);
  // Check to see if the intersection lies along the great arc going through start and end
  if (start_intersect_cos < min_cos || end_intersect_cos < min_cos) {
    if (!check_antipode) return;
    // If that failed, the antipodal point may still work. We use the flipped
    // normal to recompute in order to not depend on edge order.
    p_intersect *= -1;
    start_intersect_cos *= -1;
    end_intersect_cos *= -1;
    if (start_intersect_cos < min_cos || end_intersect_cos < min_cos) {
      // If the antipodal point fails, we have no intersection with the arc.
      // NOTE: In the case of a non-moving ray, we do not check the antipode and
      // just fail.
      return;
    }
  }
  // Linearly interpolate the w value of start and end over the length of the
  // arc between them.
  float arc_percent = acos(start_intersect_cos);
  float arc_max = acos(min_cos);
  arc_percent /= arc_max;
  intersection->position = (float4)(p_intersect, start.w * (1 - arc_percent) + end.w * arc_percent);
  intersection->intersected = true;
}

bool intersect_line_segment(Ray* ray, float4 p0, float4 p1, RayHit* hit) {

  // First check to see if the ray isn't moving forwards in w. This means we need
  // to check that our ray's starting position w overlaps with the segment and
  // find when we hit said segment.
  if (fabs(ray->direction.w) < EPSILON) {
    if (fmin(p0.w, p1.w) <= ray->origin.w && ray->origin.w <= fmax(p0.w, p1.w)) {
      float theta_0 = acos(dot(ray->s_geodesic_start, p0.xyz));
      if (dot(ray->s_geodesic_end, p0.xyz) < 0) {
        theta_0 *= -1;
      }
      float theta_1 = acos(dot(ray->s_geodesic_start, p1.xyz));
      if (dot(ray->s_geodesic_end, p1.xyz) < 0) {
        theta_1 *= -1;
      }
      float dtheta = theta_1 - theta_0;
      if (dtheta > M_PI_F) {
        theta_0 += 2 * M_PI_F;
        dtheta = -2 * M_PI_F + dtheta;
      }
      if (dtheta < -M_PI_F) {
        theta_1 += 2 * M_PI_F;
        dtheta = 2 * M_PI_F + dtheta;
      }
      float target_theta = theta_0 + dtheta * (ray->origin.w - p0.w) / (p1.w - p0.w);
      if (target_theta < 0) { target_theta += 2 * M_PI_F; }
      float t = target_theta / ray->s_geodesic_rate;
      if (t < hit->t) {
        hit->t = t;
        hit->pos = (float4)(ray->s_geodesic_start * cos(target_theta) + ray->s_geodesic_end * sin(target_theta), ray->origin.w);
        return true;
      }
    }
    return false;
  }

  // If there are two points we intersected, we need to find out at what time.
  // We must overlap in the w range, so it is between the following two times.
  float t_0 = (p0.w - ray->origin.w) / ray->direction.w;
  float t_1 = (p1.w - ray->origin.w) / ray->direction.w;
  // Swap so direction of intersects is the same as direction of ray.
  if (t_0 > t_1) {
    float t_tmp = t_1;
    t_1 = t_0;
    t_0 = t_tmp;
    float4 p_tmp = p1;
    p1 = p0;
    p0 = p_tmp;
  }
  // If our last possible time of intersection is too early, then, exit early.
  if (t_1 < 0) return false;
  // If our first possible time of intersection is too late, then exit early.
  if (t_0 > hit->t) return false;

  // Adjust positions of our intersects so they fall within the time bounds
  // of [0, hit->t]
  float3 p0_sphere = p0.xyz;
  float3 p1_sphere = p1.xyz;
  float3 arc_normal = cross(p0_sphere, p1_sphere);
  if (t_0 < 0) {
    float3 arc_start_perp = cross(arc_normal, p0_sphere);
    float3 angle = -t_0 / (t_1 - t_0) * acos(dot(p0_sphere, p1_sphere));
    p0_sphere = p0_sphere * cos(angle) + arc_start_perp * sin(angle);
    t_0 = 0;
  }
  if (t_1 > hit->t) {
    float3 arc_start_perp = cross(arc_normal, p0_sphere);
    float3 angle = (hit->t - t_0) / (t_1 - t_0) * acos(dot(p0_sphere, p1_sphere));
    p1_sphere = p0_sphere * cos(angle) + arc_start_perp * sin(angle);
    t_1 = hit->t;
  }

  // Now, we compute the offset of our intersecting arc relative to the ray
  // at time t_0.
  float dt = t_1 - t_0;
  float start_angle = t_0 * ray->s_geodesic_rate;
  float3 ray_start = ray->s_geodesic_start * cos(start_angle) + ray->s_geodesic_end * sin(start_angle);
  float3 ray_start_perp = ray->s_geodesic_end * cos(start_angle) - ray->s_geodesic_start * sin(start_angle);
  float theta_0 = acos(dot(ray_start, p0_sphere));
  if (dot(ray_start_perp, p0_sphere) < 0) {
    theta_0 *= -1;
  }
  float theta_1 = acos(dot(ray_start, p1_sphere));
  if (dot(ray_start_perp, p1_sphere) < 0) {
    theta_1 *= -1;
  }
  float dtheta = theta_1 - theta_0;
  if (dtheta > M_PI_F) {
    theta_0 += 2 * M_PI_F;
    dtheta = -2 * M_PI_F + dtheta;
  }
  if (dtheta < -M_PI_F) {
    theta_1 += 2 * M_PI_F;
    dtheta = 2 * M_PI_F + dtheta;
  }
  // If they started out at the same spot, a quick angle check will suffice.
  if (dt < EPSILON) {
    if (dtheta < 0) {
      float tmp = theta_0;
      theta_0 = theta_1;
      theta_1 = tmp;
      dtheta = -dtheta;
    }

    if (theta_0 <= 0 && theta_1 >= 0) {
      hit->t = t_0;
      hit->pos = (float4)(ray_start, p0.w);
      return true;
    }
  } else {
    float desc = ray->s_geodesic_rate - dtheta / dt;
    // Parallel case...
    if (fabs(desc) < EPSILON) {
      return false;
    }
    if (theta_0 > 0) {
      theta_0 -= 2 * M_PI_F;
    }
    float t_un_adj = theta_0 / desc;
    float t_un_adj_off = (theta_0 + 2 * M_PI_F) / desc;
    if (t_un_adj < 0 || t_un_adj > dt) {
      t_un_adj = t_un_adj_off;
    }
    if (t_un_adj_off >= 0 && t_un_adj_off <= dt && t_un_adj_off < t_un_adj) {
      t_un_adj = t_un_adj_off;
    }
    float t = t_0 + t_un_adj;

    if (t >= t_0 && t <= t_1) {
      hit->t = t;
      float ang = t * ray->s_geodesic_rate;
      float3 final_pos = ray->s_geodesic_start * cos(ang) + ray->s_geodesic_end * sin(ang);
      hit->pos = (float4)(final_pos, ray->origin.w + t * ray->direction.w);
      return true;
    }
  }
}

// This computes the intersection between a ray and (most) triangles. We require
// the following of the triangles: No edge has antipodal endpoints for their
// spherical component. Two edges meet only at their shared vertex.
bool intersect(Ray* ray, float4 triA, float4 triB, float4 triC, RayHit* hit) {
  GreatArcIntersection intersections[3];
  intersect_great_arc(ray, triA, triB, &intersections[0]);
  intersect_great_arc(ray, triB, triC, &intersections[1]);
  intersect_great_arc(ray, triC, triA, &intersections[2]);
  int num_intersected = 0;
  float4 p_intersects[3];
  float3 p_type[3];
  float p_dist[3];
  int p_idx[3];
  for (int i = 0; i < 3; i++) {
    if (intersections[i].intersected) {
      float4 intersect = intersections[i].position;
      bool add = true;
      // Check that the intersection position is not a duplicate. Happens when
      // edges meet.
      for (int j = 0; j < 3 && j < num_intersected; j++) {
        add = add && (length(p_intersects[j] - intersect) > EPSILON);
      }
      if (add) {
        p_intersects[num_intersected] = intersect;
        p_type[num_intersected] = (float3)(intersections[i].vertical ? 1.0 : 0.0, intersections[i].same_arc ? 1.0 : 0.0, i / 2.0);
        p_dist[num_intersected] = length(intersections[i].position - intersections[i].segment_end);
        p_idx[num_intersected] = i;
        num_intersected += 1;
      }
    }
  }

  if (num_intersected == 0) {
    // hit->uvw = (float3)(intersections[0].intersected ? 1.0 : 0.0, intersections[1].intersected ? 1.0 : 0.0, intersections[2].intersected ? 1.0 : 0.0);
  } else if (num_intersected == 1) {
    // If there was only one intersection point, check to see if that point is
    // equal to the geodesic evaluated at the time it takes to reach that point's
    // w value.
    if (fabs(ray->direction.w) < EPSILON) {
      if (fabs(ray->origin.w - p_intersects[0].w) < EPSILON) {
        if (fabs(ray->s_geodesic_rate) < EPSILON) {
          if (dot(ray->origin.xyz, p_intersects[0].xyz) == 1) {
            hit->t = 0;
            hit->pos = p_intersects[0];
            return true;
          }
        } else {
          float hit_angle = acos(dot(ray->s_geodesic_start, p_intersects[0].xyz));
          if (dot(ray->s_geodesic_end, p_intersects[0].xyz) < 0)
            hit_angle = 2 * M_PI_F - hit_angle;
          float t = hit_angle / ray->s_geodesic_rate;
          if (t < hit->t) {
            hit->t = t;
            hit->pos = p_intersects[0];
          }
        }
      }
    } else {
      float t = (p_intersects[0].w - ray->origin.w) / ray->direction.w;
      float angle = t * ray->s_geodesic_rate;
      float3 final_pos = ray->s_geodesic_start * cos(angle) + ray->s_geodesic_end * sin(angle);
      if (fast_length(final_pos - p_intersects[0].xyz) < EPSILON && t > 0 && t < hit->t) {
        hit->t = t;
        hit->pos = p_intersects[0];
        return true;
      }
    }
  } else if (num_intersected == 2) {
    return intersect_line_segment(ray, p_intersects[0], p_intersects[1], hit);
  } else { // num_intersected == 3
    bool found_hit = intersect_line_segment(ray, p_intersects[0], p_intersects[1], hit);
    found_hit = found_hit || intersect_line_segment(ray, p_intersects[1], p_intersects[2], hit);
    found_hit = found_hit || intersect_line_segment(ray, p_intersects[2], p_intersects[0], hit);
    return found_hit;
  }

  return false;
}

__kernel void trace(
  write_only image2d_t out_image,
  __global const float4* origin,
  __global const float4* direction,
  __global const uint4*  triangles,
  __global const float4* verticies,
  __global const float4* normals,
  __private const int num_triangles
) {
  uint global_address = get_global_id(1) + get_global_id(0) * get_global_size(1);
  Ray ray;
  init_ray(origin[global_address], direction[global_address], &ray);

  RayHit hit;
  hit.t = MAXFLOAT;
  bool has_hit = false;
  int2 pixel = (int2)(get_global_id(0), get_global_id(1));
  for (int i = 0; i < num_triangles; i++) {
    uint4 triangle_verticies = triangles[i];
    if (intersect(
      &ray,
      verticies[triangle_verticies.x],
      verticies[triangle_verticies.y],
      verticies[triangle_verticies.z],
      &hit
    )) {
      has_hit = true;
      hit.tri_coord = i;
    }
  }
  if (has_hit) {
    float4 tri_a = verticies[triangles[hit.tri_coord].x];
    float angle = acos(dot((float3)(1.0, 0.0, 0.0), tri_a.xyz));
    float3 rot_axis = normalize(cross(tri_a.xyz, (float3)(1.0, 0.0, 0.0)));
    float4 rot_quat = (float4)(cos(angle / 2.0f), sin(angle/2.0f) * rot_axis.xyz);
    float3 new_sphere_normal = rotate_point(triangle_normal[hit.tri_coord].xyz, rot_quat);
    float3 tangent_color = (float3)(
      new_sphere_normal.z,
      new_sphere_normal.y,
      triangle_normal[hit.tri_coord].w
    ) * 0.5f + 0.5f;
    write_imagef(
      out_image,
      pixel,
      (float4)(tangent_color, 1.0)
    );
  } else {
    write_imagef(
      out_image,
      pixel,
      (float4)(0.0, 0.0, 0.0, 1.0)
    );
  }
}
