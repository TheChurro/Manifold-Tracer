use super::kernels::Wavefront;
use super::{Ball, MeshDescription, Object, Offset, Orientation, Point, Triangle, EPSILON};
use na::{UnitQuaternion, Vector3};
use ron::de::from_str;
use wavefront_obj::obj::parse;

#[derive(Serialize, Deserialize)]
pub struct SceneDescription {
    #[serde(default)]
    pub mesh_objs: Vec<Object<MeshLocation>>,
    #[serde(default)]
    pub triangles: Vec<Object<Triangle>>,
    #[serde(default)]
    pub balls: Vec<Object<Ball>>,
    #[serde(default)]
    pub sequences: Vec<Sequence>,
}

#[derive(Debug)]
pub enum SceneLoadErr {
    FileIO(std::io::Error),
    RONParse(ron::de::Error),
}

#[derive(Debug)]
pub struct MeshLoadWarning {
    pub mesh_path: String,
    pub error: MeshLoadErr,
}

pub struct SceneDescriptionWithWarnings {
    pub failed_mesh_loads: Vec<MeshLoadWarning>,
    pub scene: SceneDescription,
}

impl SceneDescription {
    pub fn load(path: String) -> Result<SceneDescriptionWithWarnings, SceneLoadErr> {
        use SceneLoadErr::*;
        let scene_string = std::fs::read_to_string(path).map_err(|e| FileIO(e))?;
        let mut scene: SceneDescription = from_str(&scene_string).map_err(|e| RONParse(e))?;
        let mut failed_mesh_loads = Vec::new();
        for mesh_obj in &mut scene.mesh_objs {
            if let Err(e) = mesh_obj.geometry.load_mesh_descriptions() {
                failed_mesh_loads.push(MeshLoadWarning {
                    mesh_path: mesh_obj.geometry.obj_path.clone(),
                    error: e,
                });
            }
        }
        Ok(SceneDescriptionWithWarnings {
            failed_mesh_loads: failed_mesh_loads,
            scene: scene,
        })
    }

    pub fn dump_to_wavefront(&self, wavefront: &mut Wavefront) {
        for mesh_obj in &self.mesh_objs {
            if let &Some(ref meshes) = &mesh_obj.geometry.mesh_descriptions {
                for mesh in meshes {
                    let instance =
                        mesh.instantiate(Orientation::simple(&mesh_obj.geometry.location));
                    for tri in instance.triangles {
                        wavefront.triangle(tri, mesh_obj.color, mesh_obj.material);
                    }
                }
            }
        }
        for triangle in &self.triangles {
            wavefront.triangle(triangle.geometry.clone(), triangle.color, triangle.material);
        }
        for ball in &self.balls {
            wavefront.ball(ball.geometry.clone(), ball.color, ball.material);
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum MeshPivot {
    CenterCenter,
    BottomCenter,
    TopCenter,
}

impl Default for MeshPivot {
    fn default() -> Self {
        MeshPivot::CenterCenter
    }
}

impl MeshPivot {
    pub fn to_pivot(
        self,
        center: Vector3<f32>,
        mins: Vector3<f32>,
        maxs: Vector3<f32>,
    ) -> Vector3<f32> {
        match self {
            Self::CenterCenter => center,
            Self::BottomCenter => Vector3::new(center.x, mins.y, center.z),
            Self::TopCenter => Vector3::new(center.x, maxs.y, center.z),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Sequence {
    pub move_dir: [f32; 3],
    pub view_angles: [f32; 2],
    pub angular_speed: [f32; 2],
    pub angle: f32,
    pub speed: f32,
    pub fps: u32,
    pub oversamples: u32,
    pub folder: String
}

#[derive(Serialize, Deserialize)]
pub struct MeshLocation {
    pub obj_path: String,
    pub location: Point,
    pub scale: f32,
    #[serde(default)]
    pub pivot: MeshPivot,
    #[serde(default)]
    pub euler_angles: [f32; 3],
    #[serde(skip)]
    pub mesh_descriptions: Option<Vec<MeshDescription>>,
}

#[derive(Debug)]
pub enum MeshLoadErr {
    MeshIO(std::io::Error),
    ObjParse(wavefront_obj::ParseError),
}

impl MeshLocation {
    pub fn load_mesh_descriptions(&mut self) -> Result<(), MeshLoadErr> {
        use MeshLoadErr::*;
        let mesh_obj_str = std::fs::read_to_string(self.obj_path.clone()).map_err(|e| MeshIO(e))?;
        let obj_set = parse(mesh_obj_str).map_err(|e| ObjParse(e))?;
        let mut meshes = Vec::new();
        for obj in obj_set.objects {
            use wavefront_obj::obj::Primitive::Triangle as PTri;
            let mut center_accum = Vector3::<f32>::zeros();
            use std::f32::{INFINITY, NEG_INFINITY};
            let mut mins = Vector3::new(INFINITY, INFINITY, INFINITY);
            let mut maxs = Vector3::new(NEG_INFINITY, NEG_INFINITY, NEG_INFINITY);
            for vert in &obj.vertices {
                center_accum += Vector3::new(vert.x as f32, vert.y as f32, vert.z as f32);
                mins = Vector3::new(
                    (vert.x as f32).min(mins.x),
                    (vert.y as f32).min(mins.y),
                    (vert.z as f32).min(mins.z),
                );
                maxs = Vector3::new(
                    (vert.x as f32).max(maxs.x),
                    (vert.y as f32).max(maxs.y),
                    (vert.z as f32).max(maxs.z),
                );
            }
            let center = center_accum / obj.vertices.len() as f32;
            let mut obj_radius = 0f32;
            for vert in &obj.vertices {
                obj_radius = obj_radius.max(
                    (center - Vector3::new(vert.x as f32, vert.y as f32, vert.z as f32)).norm(),
                );
            }
            let mut vertex_offsets = Vec::new();
            let pivot = self.pivot.to_pivot(center, mins, maxs);
            let rotation = UnitQuaternion::from_axis_angle(
                &Vector3::x_axis(),
                self.euler_angles[0].to_radians(),
            ) * UnitQuaternion::from_axis_angle(
                &Vector3::y_axis(),
                self.euler_angles[1].to_radians(),
            ) * UnitQuaternion::from_axis_angle(
                &Vector3::z_axis(),
                self.euler_angles[2].to_radians(),
            );
            for vert in &obj.vertices {
                let mut tangent =
                    Vector3::new(vert.x as f32, vert.y as f32, vert.z as f32) - pivot;
                let dist = tangent.norm();
                if dist < EPSILON {
                    tangent = Vector3::new(1.0, 0.0, 0.0);
                } else {
                    tangent /= dist;
                }
                tangent = rotation * tangent;
                vertex_offsets.push(Offset {
                    tangent: [tangent.x, tangent.y, tangent.z],
                    distance: self.scale * dist / obj_radius,
                });
            }
            let mut triangles = Vec::new();
            for geometry in obj.geometry {
                for shape in geometry.shapes {
                    if let PTri(x0, x1, x2) = shape.primitive {
                        triangles.push((x0.0, x1.0, x2.0))
                    }
                }
            }
            meshes.push(MeshDescription {
                triangles: triangles,
                verticies_offsets: vertex_offsets,
            });
        }
        self.mesh_descriptions = Some(meshes);
        Ok(())
    }
}
