use manifold_tracer::geometry::three_sphere::{orientation::*, representation::*};

#[test]
fn ij_rotation() {
    use na::UnitQuaternion;
    let i = Point::new(0.0, 1.0, 0.0, 0.0);
    let j = Point::new(0.0, 0.0, 1.0, 0.0);
    let k = Point::new(0.0, 0.0, 0.0, 1.0);
    let identity = Point::new(1.0, 0.0, 0.0, 0.0);
    for n in 0..360 {
        let angle = (n as f32).to_radians();
        let orientation = Orientation::rotate_on_plane(&i, &j, angle, 0.0);
        let expected = UnitQuaternion::new_normalize(
            i.into_inner() * angle.cos() + j.into_inner() * angle.sin(),
        )
        .into_inner();
        let output = &orientation * i;
        if (output.into_inner() - expected).magnitude() > 0.001 {
            panic!(
                "Failed at angle {}!\n\tactual: {}\n\texpected: {}",
                angle,
                output.into_inner(),
                expected
            );
        }
        let output = &orientation * identity;
        if (output.into_inner() - identity.into_inner()).magnitude() > 0.001 {
            panic!(
                "Failed at angle {} for identity!\n\tactual: {}\n\texpected: {}",
                angle,
                output.into_inner(),
                identity.into_inner()
            );
        }
        let output = orientation * k;
        if (output.into_inner() - k.into_inner()).magnitude() > 0.001 {
            panic!(
                "Failed at angle {} for k!\nactual: {}\nexpected: {}\n",
                angle,
                output.into_inner(),
                k.into_inner()
            );
        }
    }
}

#[test]
fn ij_and_1k_rotation() {
    use na::UnitQuaternion;
    let i = Point::new(0.0, 1.0, 0.0, 0.0);
    let j = Point::new(0.0, 0.0, 1.0, 0.0);
    let k = Point::new(0.0, 0.0, 0.0, 1.0);
    let identity = Point::new(1.0, 0.0, 0.0, 0.0);
    for n in 0..360 {
        let angle = (n as f32).to_radians();
        let orientation = Orientation::rotate_on_plane(&i, &j, angle, angle);
        let expected = UnitQuaternion::new_normalize(
            i.into_inner() * angle.cos() + j.into_inner() * angle.sin(),
        )
        .into_inner();
        let output = &orientation * i;
        if (output.into_inner() - expected).magnitude() > 0.001 {
            panic!(
                "Failed at angle {}!\n\tactual: {}\n\texpected: {}",
                angle,
                output.into_inner(),
                expected
            );
        }
        let expected = UnitQuaternion::new_normalize(
            identity.into_inner() * angle.cos() + k.into_inner() * angle.sin(),
        )
        .into_inner();
        let output = &orientation * identity;
        if (output.into_inner() - expected).magnitude() > 0.001 {
            panic!(
                "Failed at angle {} for identity!\n\tactual: {}\n\texpected: {}",
                angle,
                output.into_inner(),
                identity.into_inner()
            );
        }
    }
}

#[test]
fn ij_and_1k_fixed_rotation() {
    use na::UnitQuaternion;
    let i = Point::new(0.0, 1.0, 0.0, 0.0);
    let j = Point::new(0.0, 0.0, 1.0, 0.0);
    let k = Point::new(0.0, 0.0, 0.0, 1.0);
    let identity = Point::new(1.0, 0.0, 0.0, 0.0);
    let contra_angle = std::f32::consts::FRAC_PI_4;
    for n in 0..360 {
        let angle = (n as f32).to_radians();
        let orientation = Orientation::rotate_on_plane(&i, &j, angle, contra_angle);
        let expected = UnitQuaternion::new_normalize(
            i.into_inner() * angle.cos() + j.into_inner() * angle.sin(),
        )
        .into_inner();
        let output = &orientation * i;
        if (output.into_inner() - expected).magnitude() > 0.001 {
            panic!(
                "Failed at angle {}!\n\tactual: {}\n\texpected: {}",
                angle,
                output.into_inner(),
                expected
            );
        }
        let expected = UnitQuaternion::new_normalize(
            identity.into_inner() * contra_angle.cos() + k.into_inner() * contra_angle.sin(),
        )
        .into_inner();
        let output = &orientation * identity;
        if (output.into_inner() - expected).magnitude() > 0.001 {
            panic!(
                "Failed at angle {} for identity!\n\tactual: {}\n\texpected: {}",
                angle,
                output.into_inner(),
                identity.into_inner()
            );
        }
    }
}

#[test]
fn ij_fixed_and_1k_rotation() {
    use na::UnitQuaternion;
    let i = Point::new(0.0, 1.0, 0.0, 0.0);
    let j = Point::new(0.0, 0.0, 1.0, 0.0);
    let k = Point::new(0.0, 0.0, 0.0, 1.0);
    let identity = Point::new(1.0, 0.0, 0.0, 0.0);
    let angle = std::f32::consts::FRAC_PI_4;
    for n in 0..360 {
        let contra_angle = (n as f32).to_radians();
        let orientation = Orientation::rotate_on_plane(&i, &j, angle, contra_angle);
        let expected = UnitQuaternion::new_normalize(
            i.into_inner() * angle.cos() + j.into_inner() * angle.sin(),
        )
        .into_inner();
        let output = &orientation * i;
        if (output.into_inner() - expected).magnitude() > 0.001 {
            panic!(
                "Failed at angle {}!\n\tactual: {}\n\texpected: {}",
                angle,
                output.into_inner(),
                expected
            );
        }
        let expected = UnitQuaternion::new_normalize(
            identity.into_inner() * contra_angle.cos() + k.into_inner() * contra_angle.sin(),
        )
        .into_inner();
        let output = &orientation * identity;
        if (output.into_inner() - expected).magnitude() > 0.001 {
            panic!(
                "Failed at angle {} for identity!\n\tactual: {}\n\texpected: {}",
                angle,
                output.into_inner(),
                identity.into_inner()
            );
        }
    }
}
