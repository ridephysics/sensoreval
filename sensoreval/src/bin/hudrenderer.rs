use clap::Parser as _;
use sensoreval::*;
use sensoreval_utils::IntoIteratorMap;
use serde::Deserialize;
use std::io::Write;

// this forces them to get linked into the binaries
extern crate blas_src;
extern crate lapack_src;

#[derive(Deserialize, Debug, Clone)]
struct FFProbeStream {
    width: usize,
    height: usize,
    avg_frame_rate: std::string::String,
}

impl FFProbeStream {
    pub fn get_fps(&self) -> (f64, f64) {
        let fps_numden: Vec<&str> = self.avg_frame_rate.split('/').collect();
        if fps_numden.len() != 2 {
            panic!("invalid avg_frame_rate: {}", self.avg_frame_rate);
        }
        let fps_num: f64 = fps_numden[0].parse().unwrap();
        let fps_den: f64 = fps_numden[1].parse().unwrap();

        (fps_num, fps_den)
    }
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

fn svg2png(png: &str, svg: &str) {
    std::process::Command::new("inkscape")
        .args(["-z", "-e", png, svg])
        .output()
        .expect("inkscape failed");
}

fn run_blender<T: serde::ser::Serialize>(
    scene: &str,
    code: &T,
) -> Result<sensoreval_utils::Python, Error> {
    Ok(sensoreval_utils::Python::new_args(
        "blender",
        [
            "-b",
            "--factory-startup",
            "--python-exit-code",
            "255",
            scene,
            "--python-expr",
            "\
                import sys\n\
                import pickle\n\
                def load_data():\n\
                    \treturn pickle.load(sys.stdin.buffer)\n\
                exec(load_data())\n\
                ",
        ],
        code,
    )?)
}

struct DataTimestampIter {
    fps_num: f64,
    fps_den: f64,
    /// unit: us
    startoff: u64,

    frameid: u64,
}

impl DataTimestampIter {
    pub fn new(cfg: &config::Config, stream_info: &FFProbeStream) -> Self {
        let (fps_num, fps_den) = stream_info.get_fps();
        Self {
            fps_num,
            fps_den,
            startoff: cfg.video.startoff * 1000,
            frameid: 0,
        }
    }

    pub fn set_startoff(&mut self, us: u64) {
        self.startoff = us;
    }
}

impl Iterator for DataTimestampIter {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        let ts = self.startoff
            + ((self.frameid * 1_000_000) as f64 * self.fps_den / self.fps_num) as u64;
        self.frameid += 1;

        Some(ts)
    }
}

struct GuiCallback<'a, 'b, 'c> {
    renderctx: render::Context<'a, 'b, 'c>,
}

impl<'a, 'b, 'c> sensoreval_gui::Callback for GuiCallback<'a, 'b, 'c> {
    fn set_ts(&mut self, ctx: &mut sensoreval_gui::RuntimeContext, ts: u64) {
        let _ = self.renderctx.set_ts(ts);

        if let Ok(q) = self.renderctx.orientation() {
            ctx.set_orientation(*q);
        }
    }

    fn render(&mut self, _ctx: &mut sensoreval_gui::RuntimeContext, cr: &cairo::Context) {
        self.renderctx.render(cr).unwrap();
    }
}

#[derive(Clone, clap::ValueEnum)]
enum Mode {
    Average,
    Plot,
    #[value(name = "dataviewer")]
    DataViewer,
    #[value(name = "webdata")]
    WebData,
    Blender,
    Video,
    Psim,
}

#[derive(clap::Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Config file to use
    config: std::path::PathBuf,

    /// Render mode
    #[arg(value_enum)]
    mode: Mode,

    /// Output directory
    #[arg(short, long)]
    output: Option<std::path::PathBuf>,

    /// Blender scene directory
    #[arg(short, long)]
    blenderscenes: Option<std::path::PathBuf>,

    /// Render range
    #[arg(short, long)]
    range: Option<String>,

    /// Time range in seconds
    #[arg(long)]
    trange: Option<String>,

    /// Simulation config
    #[arg(long)]
    simcfg: Option<std::path::PathBuf>,

    /// Force using generic hudrenderer
    #[arg(long)]
    force_generic: bool,
}

fn main() {
    let cli = Cli::parse();

    // load config
    let mut cfg = config::load(&cli.config).expect("can't load config");
    if cli.force_generic {
        cfg.hud.renderer = config::HudRenderer::Generic;
    }
    println!("config: {cfg:#?}");

    // load data
    let samples = cfg.load_data().expect("can't read samples");

    // init render context
    let mut renderctx = render::Context::new(&cfg, Some(&samples));

    // give videosz to renderctx
    if let Some(video_filename) = &cfg.video.filename {
        let stream_info = get_video_stream_info(video_filename);
        renderctx.set_videosz(Some((stream_info.width, stream_info.height)));
    }

    // give blenderdir to renderctx
    if let Some(output) = &cli.output {
        if !output.is_dir() {
            panic!("{} is not a directory", output.display());
        }
        renderctx.set_blenderdir(Some(output));
    }

    match &cli.mode {
        Mode::Average => {
            let trange: Vec<f64> = cli
                .trange
                .unwrap()
                .split(',')
                .map(|s| s.parse::<f64>().unwrap())
                .collect();

            let start = trange[0];
            let end = trange[1];
            let mut sum = Data::default();
            let mut n: usize = 0;

            for sample in &samples {
                let t = sample.time_seconds();

                if t >= start && t <= end {
                    n += 1;
                    sum += sample;
                }
            }

            println!("avg = {:#?}", sum / n);
        }
        Mode::Plot => {
            let mut plot = sensoreval_utils::Plot::new("/tmp/sensoreval-plot.html").unwrap();

            // plot
            renderctx.plot(&mut plot).expect("can't plot");

            if let Some(simcfgname) = &cli.simcfg {
                let mut simcfg = config::load(simcfgname).expect("can't load sim config");
                if cli.force_generic {
                    simcfg.hud.renderer = config::HudRenderer::Generic;
                }

                let simsamples = simcfg.load_data().expect("can't read sim samples");
                let simrenderctx = render::Context::new(&simcfg, Some(&simsamples));

                plot.set_trace_prefix(Some("sim-"));
                simrenderctx.plot(&mut plot).expect("can't plot sim");
                plot.set_trace_prefix::<&str>(None);
            }

            plot.finish().unwrap();
        }
        Mode::DataViewer => {
            renderctx.set_allow_missing_renders(true);

            let mut gui = sensoreval_gui::Context::default();
            gui.set_timer_ms(30);
            gui.set_orientation_enabled(true);
            gui.set_startoff(renderctx.cfg.video.startoff);
            gui.set_endoff(match renderctx.cfg.video.endoff {
                Some(v) => v,
                None => std::u64::MAX,
            });
            gui.set_callback(Some(GuiCallback { renderctx }));
            gui.set_videopath(cfg.video.filename.as_ref());
            gui.start().unwrap();
        }
        Mode::WebData => {
            let output = cli.output.as_ref().expect("no output file specified.");
            renderctx.serialize_forweb(output).unwrap();
        }
        Mode::Blender => {
            let blenderscenes = cli
                .blenderscenes
                .as_ref()
                .expect("no blenderscenes specified");
            let outdir = cli.output.as_ref().expect("no output file specified.");
            let video_file = cfg.video.filename.clone().expect("no video URL");
            let stream_info = get_video_stream_info(&video_file);
            let mut orientations = Vec::new();
            for ts in DataTimestampIter::new(&cfg, &stream_info) {
                let ret = renderctx.set_ts(ts);
                match &ret {
                    Err(Error::SampleNotFound) => break,
                    _ => ret.unwrap(),
                }

                orientations.push(renderctx.orientation().unwrap());
            }

            let mut id_start = 0;
            let mut id_end = orientations.len();
            if let Some(range) = &cli.range {
                let range: Vec<&str> = range.split(':').collect();
                if range.len() != 2 {
                    panic!("invalid range");
                }

                id_start = range[0].parse().unwrap();
                id_end = range[1].parse().unwrap();
            }

            eprintln!("num_orientations: {}", orientations.len());

            let mut blender = run_blender(
                blenderscenes
                    .join("mannequin/mannequin.blend")
                    .as_path()
                    .to_str()
                    .unwrap(),
                &include_str!("../python/blender_common.py"),
            )
            .unwrap();
            blender
                .write(&include_str!("../python/blender_mannequin.py"))
                .unwrap();
            blender.write(&outdir.join("mannequin")).unwrap();
            blender.write(&"mannequin").unwrap();
            let axis = nalgebra::Unit::new_normalize(nalgebra::Vector3::new(0.0, 0.0, 1.0));
            blender
                .write(&orientations[id_start..id_end].map_intoiter(|q| {
                    let fid = render::quat_to_fid(q);

                    // the mannequin looks toward the camera, fix that
                    let q =
                        q * nalgebra::UnitQuaternion::from_axis_angle(&axis, std::f64::consts::PI);

                    let q = nalgebra::UnitQuaternion::from_quaternion(
                        render::process_quat_for_name(q.as_vector()).into(),
                    );
                    (fid, [q[3], q[0], q[1], q[2]])
                }))
                .unwrap();
            blender
                .write(&[stream_info.width, stream_info.height])
                .unwrap();
            blender.wait().unwrap();
        }
        Mode::Video => {
            let outdir = cli.output.as_ref().expect("no output file specified.");
            let out_video = outdir.join("final.mkv");
            let video_file = cfg.video.filename.clone().expect("no video URL");
            let stream_info = get_video_stream_info(&video_file);

            let mut surface = cairo::ImageSurface::create(
                cairo::Format::ARgb32,
                stream_info.width as i32,
                stream_info.height as i32,
            )
            .expect("Can't create surface");

            let mut t_start: u64 = cfg.video.startoff;
            let mut t_end: u64 = cfg.video.endoff.unwrap();
            if let Some(range) = &cli.range {
                let range: Vec<&str> = range.split(':').collect();
                if range.len() != 2 {
                    panic!("invalid range");
                }

                let x: u64 = range[1].parse().unwrap();
                t_end = t_start + x;

                let x: u64 = range[0].parse().unwrap();
                t_start += x;
            }
            println!("start:{t_start} end:{t_end}");

            let mut args = vec!["-y"];

            // video
            let arg_ss = format!("{}", t_start as f64 / 1000.0);
            args.extend_from_slice(&["-ss", &arg_ss, "-i", &video_file]);

            // blur mask
            let blurmask_png = outdir.join("blurmask.png");
            if let Some(svg) = &cfg.video.blurmask {
                let png_str = blurmask_png.as_path().to_str().unwrap();
                svg2png(png_str, svg);
                args.extend_from_slice(&["-i", png_str]);
            }

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
            let filter_str = if cfg.video.blurmask.is_some() {
                "\
                    [1]loop=loop=-1:size=1:start=0[1l];\
                    [0][1l]alphamerge,boxblur=20[0a];
                    [0][0a]overlay[0b];\
                    [0b][2]overlay=alpha=premultiplied:format=rgb\
                "
            } else {
                "\
                    [0][1]overlay=alpha=premultiplied:format=rgb\
                "
            };
            args.extend_from_slice(&["-filter_complex", filter_str]);

            // output
            let arg_t = format!("{}", (t_end - t_start) as f64 / 1000.0);
            args.extend_from_slice(&[
                "-codec:v",
                "libx264",
                "-pix_fmt",
                "yuv420p",
                "-qp",
                "0",
                "-t",
                &arg_t,
                "-an",
                out_video.as_path().to_str().unwrap(),
            ]);

            let mut child = std::process::Command::new("ffmpeg")
                .args(args)
                .stdin(std::process::Stdio::piped())
                .spawn()
                .expect("can't spawn ffmpeg");

            let mut child_stdin = child.stdin.take().unwrap();

            let mut dtiter = DataTimestampIter::new(&cfg, &stream_info);
            dtiter.set_startoff(t_start * 1000);
            for ts in dtiter {
                // render
                {
                    let ret = renderctx.set_ts(ts);
                    match &ret {
                        Err(Error::SampleNotFound) => {
                            eprintln!("OUT OF DATA");
                            break;
                        }
                        _ => ret.unwrap(),
                    }

                    let cr = cairo::Context::new(&surface).unwrap();
                    cr.set_antialias(cairo::Antialias::Best);
                    surface.flush();
                    renderctx.render(&cr).unwrap();
                }

                // write frame
                let data = surface.data().unwrap();
                let ret = child_stdin.write_all(&data);
                match &ret {
                    Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => {
                        eprintln!("FFMPEG CLOSED");
                        break;
                    }
                    _ => ret.unwrap(),
                }
            }

            println!("DONE RENDERING");
            wait_for_child(&mut child);
        }
        Mode::Psim => {
            let sd = match &cfg.data.source {
                crate::config::DataSource::SimulatorData(d) => d,
                _ => panic!("psim works with a simulator data source only"),
            };

            // TODO: add control_input support

            sensoreval_psim::run::run_sim(
                sd.dt,
                &sd.model,
                ndarray::Array::from(sd.initial.clone()),
            );
        }
    }
}
