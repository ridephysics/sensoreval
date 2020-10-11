/// rotate 3d vector `data` by angles defined in `rot`
/// The rotation order is 0, 1, 2. This should definiitely be replaced by a
/// quaternion but would make estimating them really hard
pub fn rotate_imudata<S>(rot: &[f64], data: &mut ndarray::ArrayBase<S, ndarray::Ix1>)
where
    S: ndarray::DataMut<Elem = f64>,
{
    let ndata = nalgebra::Vector3::new(data[0], data[1], data[2]);

    let axis_east = nalgebra::Unit::new_normalize(nalgebra::Vector3::new(1.0, 0.0, 0.0));
    let q = nalgebra::UnitQuaternion::from_axis_angle(&axis_east, rot[0]);
    let ndata = q * ndata;

    let axis_north = nalgebra::Unit::new_normalize(nalgebra::Vector3::new(0.0, 1.0, 0.0));
    let q = nalgebra::UnitQuaternion::from_axis_angle(&axis_north, rot[1]);
    let ndata = q * ndata;

    let axis_up = nalgebra::Unit::new_normalize(nalgebra::Vector3::new(0.0, 0.0, 1.0));
    let q = nalgebra::UnitQuaternion::from_axis_angle(&axis_up, rot[2]);
    let ndata = q * ndata;

    data[0] = ndata[0];
    data[1] = ndata[1];
    data[2] = ndata[2];
}
