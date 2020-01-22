use crate::geometry::space::manifold::Manifold;
use crate::geometry::spherinder::spaces::{
    Spherinder, SpherinderGeodesic, SpherinderGeodesicSegment, SpherinderPoint, SpherinderTangent,
    SquaredNorm,
};

#[derive(Clone, Copy, Debug)]
pub struct MeshTriangle(pub usize, pub usize, pub usize);

pub struct MeshInstance<P> {
    pub points: Vec<SpherinderPoint<P>>,
}

pub struct Mesh<P> {
    pub geometry: MeshInstance<P>,
    pub tris: Vec<MeshTriangle>,
}

pub struct OffsetMesh<T> {
    pub offsets: Vec<SpherinderTangent<T>>,
    pub tris: Vec<MeshTriangle>,
}

impl<T> OffsetMesh<T> {
    pub fn base_at<P>(&self, base: SpherinderPoint<P>) -> MeshInstance<P>
    where
        Spherinder<P, T>: Manifold<
            SpherinderPoint<P>,
            SpherinderTangent<T>,
            SpherinderGeodesic<P>,
            SpherinderGeodesicSegment<P>,
        >,
        T: SquaredNorm,
        P: Clone + Copy
    {
        self.scaled_base_at(base, 1.0)
    }

    pub fn scaled_base_at<P>(&self, base: SpherinderPoint<P>, scale: f32) -> MeshInstance<P>
    where
        Spherinder<P, T>: Manifold<
            SpherinderPoint<P>,
            SpherinderTangent<T>,
            SpherinderGeodesic<P>,
            SpherinderGeodesicSegment<P>,
        >,
        T: SquaredNorm + Clone + Copy,
        P: Clone + Copy
    {
        let spherinder = Spherinder::new();
        MeshInstance {
            points: self
                .offsets
                .iter()
                .map(move |tangent| {
                    spherinder.eval_length(
                        spherinder.exponential(base, *tangent),
                        scale * tangent.norm(),
                    )
                })
                .collect(),
        }
    }
}
