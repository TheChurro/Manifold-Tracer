pub trait Manifold<Point, Tangent, Geodesic, GeodesicSegment> {
    fn exponential(&self, a: Point, b: Tangent) -> Geodesic;
    fn logarithmic(&self, a: Point, b: Point) -> Geodesic;
    fn set_length(&self, start: Point, length: f32, g: Geodesic) -> GeodesicSegment;
    fn shorten(&self, g: GeodesicSegment, new_length: f32) -> GeodesicSegment;
    fn segment(&self, a: Point, b: Point, g: Geodesic) -> GeodesicSegment;
    fn eval_length(&self, g: Geodesic, length: f32) -> Point;
    fn eval_time(&self, g: Geodesic, time: f32) -> Point;
}
