use sensoreval::*;

use sensoreval::PlotUtils;
use std::io::Write;

// this forces them to get linked into the binaries
extern crate blas_src;
extern crate lapack_src;

#[allow(clippy::many_single_char_names)]
fn main() {
    // parse args
    let matches = clap::App::new("synctool")
        .version("0.1")
        .arg(
            clap::Arg::with_name("CONFIG")
                .help("config file to use")
                .required(true)
                .index(1),
        )
        .arg(
            clap::Arg::with_name("seconds")
                .long("seconds")
                .required(false)
                .takes_value(false)
                .help("use seconds instead of indices for the x-axis"),
        )
        .get_matches();
    let cfgname = matches.value_of("CONFIG").unwrap();
    let seconds = matches.is_present("seconds");

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
    let mut plot = sensoreval_utils::Plot::new("/tmp/sensoreval-plot.html").unwrap();
    let x: Vec<f64> = samples
        .iter()
        .enumerate()
        .map(|(i, sample)| {
            if seconds {
                sample.time_seconds()
            } else {
                i as f64
            }
        })
        .collect();

    plot.add_measurements(&samples, &x).unwrap();

    let mut trace = sensoreval_utils::Plot::default_line();
    trace.x(&x).name("measurement");
    trace.line().color(sensoreval_utils::COLOR_M);

    let y: Vec<f64> = samples
        .iter()
        .enumerate()
        .map(|(i, s)| {
            if i > 0 {
                (s.time - samples[i - 1].time) as f64 / 1_000_000.0f64
            } else {
                0.0f64
            }
        })
        .collect();
    trace.y(&y);
    plot.add_row(Some("dt")).unwrap();
    plot.add_trace(&mut trace).unwrap();

    plot.finish().unwrap();

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
