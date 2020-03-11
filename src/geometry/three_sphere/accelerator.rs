use crate::geometry::three_sphere::{Ball, Direction, Object, Point, Triangle};

use rand::{distributions::Distribution, thread_rng};
use rand_distr::Uniform;

#[derive(Debug)]
pub enum BVHNode {
    Branch {
        boundary: Ball,
        left: usize,
        right: usize,
    },
    Leaf {
        boundary: Ball,
        min: usize,
        max: usize,
    },
}

pub struct BoundingVolumeHierarchy {
    pub triangle_hierarchy: Vec<BVHNode>,
    pub ball_hierarchy: Vec<BVHNode>,
}

fn build_tri_bvh(
    hierarchy: &mut Vec<BVHNode>,
    in_triangles: &mut Vec<Object<Triangle>>,
    start: usize,
    end: usize,
    leaf_size: usize,
    depth_at: usize,
    max_depth: usize,
) {
    let mut accum = Direction::zero();
    for tri_obj in &in_triangles[start..end] {
        let mut center: Direction = tri_obj.geometry.verticies[0].into();
        center = center + tri_obj.geometry.verticies[1].into();
        center = center + tri_obj.geometry.verticies[2].into();
        center = center * (1.0 / 3.0);
        accum = accum + center;
    }
    let center = Point::in_direction(accum).unwrap_or(Point::one());
    let mut radius = 0.0f32;
    for tri_obj in &in_triangles[start..end] {
        radius = radius.max(f32::acos(center.dot(&tri_obj.geometry.verticies[0])));
        radius = radius.max(f32::acos(center.dot(&tri_obj.geometry.verticies[1])));
        radius = radius.max(f32::acos(center.dot(&tri_obj.geometry.verticies[2])));
    }
    let boundary = Ball::new(center, radius);
    if end - start < leaf_size || depth_at == max_depth {
        hierarchy.push(BVHNode::Leaf {
            boundary: boundary,
            min: start,
            max: end,
        });
    } else {
        let mut rng = thread_rng();
        let dist = Uniform::new_inclusive(-1.0f32, 1.0f32);
        let mut axis = center.clone();
        while axis.dot(&center).abs() > 0.999 {
            axis = Point::in_direction(Direction::new(
                dist.sample(&mut rng),
                dist.sample(&mut rng),
                dist.sample(&mut rng),
                dist.sample(&mut rng),
            ))
            .unwrap_or(Point::one());
        }
        in_triangles[start..end].sort_by(|t0, t1| {
            t0.geometry.verticies[0]
                .dot(&axis)
                .partial_cmp(&t1.geometry.verticies[0].dot(&axis))
                .unwrap()
        });
        let branch_index = hierarchy.len();
        hierarchy.push(BVHNode::Branch {
            boundary: boundary.clone(),
            left: 0,
            right: 0,
        });
        let left_index = hierarchy.len();
        build_tri_bvh(
            hierarchy,
            in_triangles,
            start,
            (start + end + 1) / 2,
            leaf_size,
            depth_at + 1,
            max_depth,
        );
        if end != (start + end + 1) / 2 {
            let right_index = hierarchy.len();
            build_tri_bvh(
                hierarchy,
                in_triangles,
                (start + end + 1) / 2,
                end,
                leaf_size,
                depth_at + 1,
                max_depth,
            );
            hierarchy[branch_index] = BVHNode::Branch {
                boundary: boundary,
                left: left_index,
                right: right_index,
            };
        } else {
            hierarchy[branch_index] = BVHNode::Branch {
                boundary: boundary,
                left: left_index,
                right: 0usize,
            };
        }
    }
}

fn build_ball_bvh(
    hierarchy: &mut Vec<BVHNode>,
    in_balls: &mut Vec<Object<Ball>>,
    start: usize,
    end: usize,
    leaf_size: usize,
    depth_at: usize,
    max_depth: usize,
) {
    let mut accum = Direction::zero();
    for ball_obj in in_balls.iter() {
        accum = accum + ball_obj.geometry.center.into();
    }
    let center = Point::in_direction(accum).unwrap_or(Point::one());
    let mut radius = 0.0f32;
    for ball_obj in in_balls.iter() {
        radius =
            radius.max(ball_obj.geometry.radius + f32::acos(center.dot(&ball_obj.geometry.center)));
    }
    let boundary = Ball::new(center, radius);
    if end - start < leaf_size || depth_at == max_depth {
        hierarchy.push(BVHNode::Leaf {
            boundary: boundary,
            min: start,
            max: end,
        });
    } else {
        let mut rng = thread_rng();
        let dist = Uniform::new_inclusive(-1.0f32, 1.0f32);
        let mut axis = center.clone();
        while axis.dot(&center).abs() > 0.999 {
            axis = Point::in_direction(Direction::new(
                dist.sample(&mut rng),
                dist.sample(&mut rng),
                dist.sample(&mut rng),
                dist.sample(&mut rng),
            ))
            .unwrap_or(Point::one());
        }
        in_balls[start..end].sort_by(|t0, t1| {
            t0.geometry
                .center
                .dot(&axis)
                .partial_cmp(&t1.geometry.center.dot(&axis))
                .unwrap()
        });
        let branch_index = hierarchy.len();
        hierarchy.push(BVHNode::Branch {
            boundary: boundary.clone(),
            left: 0,
            right: 0,
        });
        let left_index = hierarchy.len();
        build_ball_bvh(
            hierarchy,
            in_balls,
            start,
            start + end / 2,
            leaf_size,
            depth_at + 1,
            max_depth,
        );
        let right_index = hierarchy.len();
        build_ball_bvh(
            hierarchy,
            in_balls,
            start + end / 2,
            end,
            leaf_size,
            depth_at + 1,
            max_depth,
        );
        hierarchy[branch_index] = BVHNode::Branch {
            boundary: boundary,
            left: left_index,
            right: right_index,
        };
    }
}

impl BoundingVolumeHierarchy {
    pub fn new(
        in_triangles: &mut Vec<Object<Triangle>>,
        in_balls: &mut Vec<Object<Ball>>,
    ) -> BoundingVolumeHierarchy {
        let mut tri_hierarchy = Vec::new();
        let num_triangles = in_triangles.len();
        build_tri_bvh(&mut tri_hierarchy, in_triangles, 0, num_triangles, 8, 0, 63);
        let mut ball_hierarchy = Vec::new();
        let num_balls = in_balls.len();
        build_ball_bvh(&mut ball_hierarchy, in_balls, 0, num_balls, 8, 0, 63);

        BoundingVolumeHierarchy {
            triangle_hierarchy: tri_hierarchy,
            ball_hierarchy: ball_hierarchy,
        }
    }

    pub fn traverse_tris(&self) {
        const ENTRYPOINT_SENTINEL: u32 = 0x76543210u32;
        const TRAVERSAL_STACK_SIZE: u32 = 63;
        let mut traversal_stack = [0u32; TRAVERSAL_STACK_SIZE as usize];
        let mut traversal_ptr = 0i32;
        traversal_stack[0] = ENTRYPOINT_SENTINEL;
        let mut node = 0u32;
        while node != ENTRYPOINT_SENTINEL {
            match &self.triangle_hierarchy[node as usize] {
                &BVHNode::Branch {
                    ref left,
                    ref right,
                    ..
                } => {
                    println!("BRANCH[{}]: {} & {}", node, left, right);
                    node = *left as u32;
                    if *right != 0 {
                        traversal_ptr += 1;
                        traversal_stack[traversal_ptr as usize] = *right as u32;
                    }
                }
                &BVHNode::Leaf {
                    ref min, ref max, ..
                } => {
                    println!("LEAF[{}]: {} -> {}", node, min, max);
                    node = traversal_stack[traversal_ptr as usize];
                    traversal_ptr -= 1;
                }
            }
        }
    }
}
