use sensoreval::*;
use serde::Deserialize;
use std::io::Write;

#[derive(Deserialize, Debug, Clone)]
struct FFProbeStream {
    width: usize,
    height: usize,
    avg_frame_rate: std::string::String,
}

#[derive(Deserialize, Debug)]
struct FFProbeInfo {
    streams: Vec<FFProbeStream>,
}

fn wait_for_child(child: &mut std::process::Child) {
    let status = child.wait().expect("can't wait for child");
    if !status.success() {
        panic!("child exited with: {:?}", status.code());
    }
}

fn get_video_stream_info(filename: &str) -> FFProbeStream {
    let mut child = std::process::Command::new("ffprobe")
        .args(vec![
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=width,height,avg_frame_rate",
            "-of",
            "json",
            filename,
        ])
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("can't spawn ffprobe");
    let probe_info: FFProbeInfo = serde_json::from_reader(child.stdout.take().unwrap()).unwrap();
    wait_for_child(&mut child);

    probe_info.streams[0].clone()
}

fn main() {
    // parse args
    let matches = clap::App::new("hudrenderer")
        .version("0.1")
        .arg(
            clap::Arg::with_name("mode")
                .short("m")
                .long("mode")
                .value_name("MODE")
                .default_value("video")
                .help("plot data instead of rendering"),
        )
        .arg(
            clap::Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("OUTPUT")
                .help("output file"),
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

    match matches.value_of("mode").unwrap() {
        "plot" => {
            // plot
            renderctx.plot().expect("can't plot");
        }
        "video" => {
            let output_file = match matches.value_of("output") {
                Some(v) => v,
                None => panic!("no output file specified."),
            };

            let video_file = cfg.video.filename.clone().expect("no video URL");
            let stream_info = get_video_stream_info(&video_file);
            let fps_numden: Vec<&str> = stream_info.avg_frame_rate.split('/').collect();
            if fps_numden.len() != 2 {
                panic!("invalid avg_frame_rate: {}", stream_info.avg_frame_rate);
            }
            let fps_num: f64 = fps_numden[0].parse().unwrap();
            let fps_den: f64 = fps_numden[1].parse().unwrap();

            let mut surface = cairo::ImageSurface::create(
                cairo::Format::ARgb32,
                stream_info.width as i32,
                stream_info.height as i32,
            )
            .expect("Can't create surface");

            let mut args = Vec::new();
            args.push("-y");

            // video
            let arg_ss = format!("{}", cfg.video.startoff as f64 / 1000.0);
            args.extend_from_slice(&["-ss", &arg_ss, "-i", &video_file]);

            // blur mask
            args.extend_from_slice(&["-i", "/tmp/out.png"]);

            // HUD
            let arg_videosize = format!("{}x{}", stream_info.width, stream_info.height);
            args.extend_from_slice(&[
                "-f",
                "rawvideo",
                "-pix_fmt",
                "bgra",
                "-framerate",
                &stream_info.avg_frame_rate,
                "-video_size",
                &arg_videosize,
                "-i",
                "pipe:0",
            ]);

            // filter
            args.extend_from_slice(&[
                "-filter_complex",
                "\
                        [1]loop=loop=-1:size=1:start=0[1l];\
                        [0][1l]alphamerge,boxblur=20[0a];
                        [0][0a]overlay[0b];\
                        [0b][2]overlay\
                    ",
            ]);

            // output
            let arg_t = format!(
                "{}",
                (cfg.video.endoff.unwrap() - cfg.video.startoff) as f64 / 1000.0
            );
            args.extend_from_slice(&[
                "-codec:v",
                "libx264",
                "-pix_fmt",
                "yuv420p",
                "-crf",
                "17",
                "-t",
                &arg_t,
                "-an",
                output_file,
            ]);

            let mut child = std::process::Command::new("ffmpeg")
                .args(args)
                .stdin(std::process::Stdio::piped())
                .spawn()
                .expect("can't spawn ffmpeg");

            let mut child_stdin = child.stdin.take().unwrap();
            let mut frameid: u64 = 0;
            let frame_time = 1_000_000.0f64 * fps_den / fps_num;
            loop {
                // render
                {
                    let ts = cfg.video.startoff * 1000 + (frameid as f64 * frame_time) as u64;
                    let ret = renderctx.set_ts(ts);
                    match &ret {
                        Err(Error::SampleNotFound) => break,
                        _ => ret.unwrap(),
                    }

                    let cr = cairo::Context::new(&surface);
                    cr.set_antialias(cairo::Antialias::Best);
                    surface.flush();
                    renderctx.render(&cr).unwrap();
                }

                // write frame
                let data = surface.get_data().unwrap();
                let ret = child_stdin.write_all(&data);
                match &ret {
                    Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => break,
                    _ => ret.unwrap(),
                }

                frameid += 1;
            }

            println!("DONE RENDERING");
            wait_for_child(&mut child);
        }
        mode => {
            eprintln!("invalid mode: {}", mode);
        }
    }
}
