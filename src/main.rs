use sensoreval::*;

fn main() {
    let mut args = std::env::args();
    if args.len() != 2 {
        println!("Usage: {:?} CONFIG", args.nth(0));
        std::process::exit(1);
    }
    let cfgname = args.nth(1).unwrap();
    let cfg = config::load(cfgname).unwrap();
    println!("config: {:#?}", cfg);

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
