use sensoreval::*;

use ndarray_linalg::norm::Norm;
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
    let mut cfg = config::load(&cfgname).expect("can't load config");
    cfg.video.startoff = 0;
    cfg.video.endoff = None;
    if let config::DataSource::SensorData(sd) = &mut cfg.data.source {
        sd.video_off = 0;
    }
    println!("config: {:#?}", cfg);

    // load data
    let samples = cfg.load_data().expect("can't read samples");

    if let Some(sample) = samples.first() {
        println!("FIRST: {}", sample.time);
    }

    if let Some(sample) = samples.last() {
        println!("LAST: {}", sample.time);
    }

    // plot
    let mut plot = TimeDataPlot::new(&DataSerializer::new(&samples, |i, _data| i)).unwrap();
    plot.add(&DataSerializer::new(&samples, |_i, data| {
        data.accel.as_slice().unwrap()
    }))
    .unwrap();
    plot.add(&DataSerializer::new(&samples, |_i, data| {
        data.accel.norm_l2()
    }))
    .unwrap();
    plot.add(&DataSerializer::new(&samples, |i, data| {
        if i > 0 {
            (data.time - samples[i - 1].time) as f64 / 1_000_000.0f64
        } else {
            0.0f64
        }
    }))
    .unwrap();
    plot.show().unwrap();

    // read index from stdin
    std::io::stdout().flush().unwrap();
    std::io::stderr().flush().unwrap();
    print!("index: ");
    std::io::stdout().flush().unwrap();
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("can't read line from stdin");
    let index: usize = input.trim().parse().expect("can't parse line as int");

    std::io::stdout().flush().unwrap();
    std::io::stderr().flush().unwrap();
    print!("videooff(mpv --osd-fractions): ");
    std::io::stdout().flush().unwrap();
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("can't read line from stdin");

    // 00:01:00.944
    let re = regex::Regex::new(r"(\d{2}):(\d{2}):(\d{2}).(\d{3})").unwrap();
    let caps = re.captures(input.trim()).unwrap();
    let h: u64 = caps[1].parse().unwrap();
    let m: u64 = caps[2].parse().unwrap();
    let s: u64 = caps[3].parse().unwrap();
    let ms: u64 = caps[4].parse().unwrap();
    let us = (((h * 60 + m) * 60 + s) * 1000 + ms) * 1000;

    // print requested sample
    println!("{:#?}", samples[index]);
    println!(
        "video_off = {}",
        (us as i64).checked_sub(samples[index].time as i64).unwrap()
    );
}
