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

    // plot
    let mut plot = Plot::new(&IndexDataSerializer::from(&samples)).unwrap();
    plot.add(&AccelDataSerializer::from(&samples)).unwrap();
    plot.add(&AccelLenDataSerializer::from(&samples)).unwrap();
    plot.show().unwrap();

    // read index from stdin
    print!("index: ");
    std::io::stdout().flush();
    let mut input = String::new();
    match std::io::stdin().read_line(&mut input) {
        Err(e) => {
            println!("can't read line from stdin: {}", e);
            std::process::exit(1);
        }
        _ => (),
    }

    let index: usize = match input.trim().parse() {
        Err(e) => {
            println!("can't parse line as int: {}", e);
            std::process::exit(1);
        }
        Ok(v) => v,
    };

    // print requested sample
    println!("{:#?}", samples[index]);
}
