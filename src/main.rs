use sensoreval::*;

fn main() {
    let mut x = Vec::new();
    let mut y_accel = Vec::new();
    let mut y_pressure = Vec::new();

    let samples = data::read_all_samples(&mut std::io::stdin()).unwrap();
    for sample in samples {
        x.push(sample.time);
        y_accel.push(sample.accel);
        y_pressure.push(sample.pressure);
    }

    let mut plot = plot::Plot::new(x).unwrap();
    plot.add(y_accel).unwrap();
    plot.add(y_pressure).unwrap();
    plot.show().unwrap();
}
