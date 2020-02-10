use sensoreval::*;

fn main() {
    // parse args
    let matches = clap::App::new("hudrenderer")
        .version("0.1")
        .arg(
            clap::Arg::with_name("plot")
                .short("p")
                .long("plot")
                .help("plot data instead of rendering"),
        )
        .arg(
            clap::Arg::with_name("CONFIG")
                .help("config file to use")
                .required(true)
                .index(1),
        )
        .get_matches();

    let cfgname = matches.value_of("CONFIG").unwrap();

    // load config
    let cfg = config::load(&cfgname).expect("can't load config");
    println!("config: {:#?}", cfg);

    // load data
    let samples = cfg.load_data().expect("can't read samples");

    // init render context
    let mut renderctx = render::Context::new(&cfg, Some(&samples));

    if matches.is_present("plot") {
        // plot
        renderctx.plot().expect("can't plot");
    } else {
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
    }
}
