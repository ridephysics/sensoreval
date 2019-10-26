use sensoreval::*;

use std::io::Write;

fn main() {
    // parse args
    let mut args = std::env::args();
    if args.len() != 2 {
        println!("Usage: {} CONFIG", args.nth(0).unwrap());
        std::process::exit(1);
    }
    let cfgname = args.nth(1).unwrap();

    // load config
    let cfg = config::load(&cfgname).expect("can't load config");
    println!("config: {:#?}", cfg);

    // load data
    let samples = datareader::read_all_samples_cfg(&cfg).expect("can't read all samples");

    if let Some(sample) = samples.first() {
        println!("FIRST: {}", sample.time);
    }

    if let Some(sample) = samples.last() {
        println!("LAST: {}", sample.time);
    }

    // plot
    let mut plot = Plot::new(&DataSerializer::new(&samples, |i, _data| {
        i
    }))
    .unwrap();
    plot.add(&DataSerializer::new(&samples, |_i, data| data.accel))
        .unwrap();
    plot.add(&DataSerializer::new(&samples, |_i, data| {
        data.accel.magnitude()
    }))
    .unwrap();
    plot.show().unwrap();

    // read index from stdin
    print!("index: ");
    std::io::stdout().flush().unwrap();
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("can't read line from stdin");
    let index: usize = input.trim().parse().expect("can't parse line as int");

    // print requested sample
    println!("{:#?}", samples[index]);
}
