use crate::math::geometry::aabb::AABBGeometry;
use crate::math::ray::{Ray, RayCollidable, RayHit};

use crate::rendering::scene::Renderable;
use crate::rendering::materials::Material;

pub struct BoundingVolumeHierarchy {
    pub hierarchy_heap: Vec<BVHNode>,
}

fn load_hierarchy_heap_at(
    heap: &mut Vec<BVHNode>,
    volumes: &mut [(usize, AABBGeometry)],
    root_node: usize,
) {
    if volumes.len() == 0 {
        return;
    }
    if volumes.len() == 1 {
        heap[root_node] = BVHNode::Leaf(volumes[0].0);
        return;
    }
    // Compute box bounding all input objects. Simultaneously
    // compute the average farthest position in each direction.
    // Then compute the variance of the farthest position in
    // each direction of the AABBs. Use this to determine which
    // axis we will split our hierarchy on. We want the most variant
    // axis for the best results.
    let mut bounding_box = volumes[0].1;
    let mut max = bounding_box.max();
    let num_volumes = volumes.len();
    let inv_num_volumes = 1.0 / (num_volumes as f32);
    let mut x_avg = max.x * inv_num_volumes;
    let mut y_avg = max.y * inv_num_volumes;
    let mut z_avg = max.z * inv_num_volumes;
    for i in 1..num_volumes {
        max = volumes[i].1.max();
        x_avg += max.x * inv_num_volumes;
        y_avg += max.y * inv_num_volumes;
        z_avg += max.z * inv_num_volumes;
        bounding_box += volumes[i].1;
    }
    let mut x_var = 0.0;
    let mut y_var = 0.0;
    let mut z_var = 0.0;
    for &(_, ref aabb) in volumes.iter() {
        max = aabb.max();
        x_var += (max.x - x_avg) * (max.x - x_avg);
        y_var += (max.y - y_avg) * (max.y - y_avg);
        z_var += (max.z - z_avg) * (max.z - z_avg);
    }
    // Sort along the most varied axis
    if x_var > y_var && x_var > z_var {
        volumes.sort_by(|a, b| {
            a.1.max()
                .x
                .partial_cmp(&b.1.max().x)
                .unwrap_or(std::cmp::Ordering::Less)
        });
    } else if y_var > z_var {
        volumes.sort_by(|a, b| {
            a.1.max()
                .y
                .partial_cmp(&b.1.max().y)
                .unwrap_or(std::cmp::Ordering::Less)
        });
    } else {
        volumes.sort_by(|a, b| {
            a.1.max()
                .z
                .partial_cmp(&b.1.max().z)
                .unwrap_or(std::cmp::Ordering::Less)
        });
    };
    // Now we set the current node to be a split with the surrounding AABB.
    heap[root_node] = BVHNode::Split(bounding_box);
    let left_child = root_node * 2 + 1;
    let right_child = root_node * 2 + 2;
    let (left_volumes, right_volumes) = volumes.split_at_mut(num_volumes / 2);
    load_hierarchy_heap_at(heap, left_volumes, left_child);
    load_hierarchy_heap_at(heap, right_volumes, right_child);
}

impl BoundingVolumeHierarchy {
    pub fn construct(
        renderables: &Vec<Renderable>,
        t_min: f32,
        t_max: f32,
    ) -> BoundingVolumeHierarchy {
        let mut volumes: Vec<_> = renderables
            .iter()
            .map(|x| x.collider.bounding_box(t_min, t_max))
            .enumerate()
            .filter(|x| x.1.is_some())
            .map(|x| (x.0, x.1.unwrap()))
            .collect();
        let num_elems = volumes.len();
        if num_elems == 0 {
            return BoundingVolumeHierarchy {
                hierarchy_heap: Vec::with_capacity(0),
            };
        }
        let mut capacity = 1;
        while capacity < num_elems {
            capacity <<= 1;
        }
        capacity <<= 1;
        capacity -= 1;
        let mut heap = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            heap.push(BVHNode::Empty);
        }
        load_hierarchy_heap_at(&mut heap, &mut volumes, 0);
        BoundingVolumeHierarchy {
            hierarchy_heap: heap,
        }
    }

    pub fn cast_ray(
        &self,
        ref_renderables: &Vec<Renderable>,
        ray: &Ray,
        t_min: f32,
        t_max: f32,
    ) -> Option<(RayHit, Material)> {
        self.cast_ray_impl(ref_renderables, 0, ray, t_min, t_max)
    }

    pub fn print(&self) {
        for node in &self.hierarchy_heap {
            println!("{:?}", node);
        }
    }

    fn cast_ray_impl(
        &self,
        ref_renderables: &Vec<Renderable>,
        node: usize,
        ray: &Ray,
        t_min: f32,
        t_max: f32,
    ) -> Option<(RayHit, Material)> {
        match self.hierarchy_heap[node] {
            BVHNode::Empty => {
                println!("HIT EMPTY!!");
                None
            },
            BVHNode::Leaf(index) => {
                if let Some(hit) = ref_renderables[index].collider.hit(ray, t_min, t_max) {
                    Some((hit, ref_renderables[index].material))
                } else {
                    None
                }
            }
            BVHNode::Split(geom) => {
                if !geom.overlaps(ray, t_min, t_max) {
                    return None;
                }
                let left_hit = self.cast_ray_impl(ref_renderables, 2 * node + 1, ray, t_min, t_max);
                let right_hit = self.cast_ray_impl(ref_renderables, 2 * node + 2, ray, t_min, t_max);
                match (left_hit, right_hit) {
                    (None, right) => right,
                    (left, None) => left,
                    (Some((l_hit, l_mat)), Some((r_hit, r_mat))) => {
                        if l_hit.hit_fraction < r_hit.hit_fraction {
                            Some((l_hit, l_mat))
                        } else {
                            Some((r_hit, r_mat))
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum BVHNode {
    Empty,
    Leaf(usize),
    Split(AABBGeometry),
}
