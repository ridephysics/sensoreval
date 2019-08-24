use sensoreval::*;

fn main() {
    // parse args
    let mut args = std::env::args();
    if args.len() != 2 {
        println!("Usage: {:?} CONFIG", args.nth(0));
        std::process::exit(1);
    }
    let cfgname = args.nth(1).unwrap();

    // load config
    let cfg = config::load(cfgname).expect("can't load config");
    println!("config: {:#?}", cfg);

    // load data
    let samples =
        datareader::read_all_samples(&mut std::io::stdin(), &cfg).expect("can't read all samples");

    // init render context
    let mut renderctx = render::Context::new(&cfg, Some(&samples));
    renderctx.set_ts(0).expect("can't set timestamp");

    // render
    let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, 2720, 1520)
        .expect("Can't create surface");
    let cr = cairo::Context::new(&surface);
    cr.set_antialias(cairo::Antialias::Best);
    renderctx.render(&cr).expect("can't render");
    surface.flush();
    let mut file = std::fs::File::create("/tmp/out.png").expect("can't create png file");
    surface
        .write_to_png(&mut file)
        .expect("can't write png file");
    drop(file);

    // plot
    let mut plot = plot::Plot::new(data::TimeDataSerializer::from(&samples)).unwrap();
    plot.add(data::AccelDataSerializer::from(&samples)).unwrap();
    plot.add(data::AccelLenDataSerializer::from(&samples)).unwrap();
    plot.add(data::AltitudeDataSerializer::from(&samples)).unwrap();
    plot.show().unwrap();
}
