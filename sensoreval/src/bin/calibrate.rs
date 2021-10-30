extern crate lapack_src;

use ndarray::array;
use ndarray_linalg::solve::Inverse;
use sensoreval::*;

fn load_samples(dir: &std::path::Path, name: &str) -> Vec<Data> {
    let path = dir.join(name);
    let cfg = sensoreval::config::Config::for_calibration(
        path.as_path().to_str().expect("can't load sensordata"),
    );
    cfg.load_data().expect("can't read samples")
}

fn calib_gyro(samples: &[Data]) -> ndarray::Array1<f64> {
    let samples_slice = &samples[0..5000];

    let mut offsets = array![0.0, 0.0, 0.0];
    for sample in samples_slice {
        offsets += &sample.gyro;
    }
    offsets /= samples_slice.len() as f64;

    offsets
}

fn accel_avg(samples: &[Data]) -> ndarray::Array1<f64> {
    let samples_slice = &samples[0..750];

    let mut offsets = array![0.0, 0.0, 0.0];
    for sample in samples_slice {
        offsets += &sample.accel;
    }
    offsets /= samples_slice.len() as f64;

    offsets
}

#[allow(non_snake_case)]
fn main() {
    // parse args
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        println!("Usage: {} CALIBDIR OUT", args[0]);
        std::process::exit(1);
    }
    let calibdir_str = &args[1];
    let outpath = &args[2];
    let calibdir = std::path::Path::new(&calibdir_str);

    let xpos = load_samples(calibdir, "x_pos.imu");
    let xneg = load_samples(calibdir, "x_neg.imu");
    let ypos = load_samples(calibdir, "y_pos.imu");
    let yneg = load_samples(calibdir, "y_neg.imu");
    let zpos = load_samples(calibdir, "z_pos.imu");
    let zneg = load_samples(calibdir, "z_neg.imu");

    let mut accel_ref = ndarray::Array2::<f64>::zeros((6, 3));
    accel_ref
        .index_axis_mut(ndarray::Axis(0), 0)
        .assign(&accel_avg(&xpos));
    accel_ref
        .index_axis_mut(ndarray::Axis(0), 1)
        .assign(&accel_avg(&xneg));
    accel_ref
        .index_axis_mut(ndarray::Axis(0), 2)
        .assign(&accel_avg(&ypos));
    accel_ref
        .index_axis_mut(ndarray::Axis(0), 3)
        .assign(&accel_avg(&yneg));
    accel_ref
        .index_axis_mut(ndarray::Axis(0), 4)
        .assign(&accel_avg(&zpos));
    accel_ref
        .index_axis_mut(ndarray::Axis(0), 5)
        .assign(&accel_avg(&zneg));

    // calculate offsets
    let mut accel_offs = array![0.0f64, 0.0f64, 0.0f64];
    for i in 0..3 {
        accel_offs[i] = (accel_ref[[i * 2, i]] + accel_ref[[i * 2 + 1, i]]) / 2.0;
    }

    // fill matrix A for linear equations system
    let mut mat_A = ndarray::Array::zeros((3, 3));
    for i in 0..3 {
        for j in 0..3 {
            let a = accel_ref[[i * 2, j]] - accel_offs[j];
            mat_A[(i, j)] = a;
        }
    }

    let mat_A_inv = mat_A.inv().expect("can't calculate matrix inverse");
    let mut accel_T = ndarray::Array::zeros((3, 3));
    for i in 0..3 {
        for j in 0..3 {
            accel_T[[i, j]] = mat_A_inv[[j, i]] * math::GRAVITY;
        }
    }

    let gyro_offs = calib_gyro(&xpos);

    println!("accel_offs = {}", &accel_offs);
    println!("accel_T = {}", accel_T);
    println!("gyro_offs = {}", gyro_offs);

    let calibration = datareader::Calibration::new(gyro_offs, accel_offs, accel_T);
    let mut file = std::fs::File::create(outpath).expect("can't create outfile");
    bincode::serialize_into(&mut file, &calibration).expect("can't serialize");
}
