use sensoreval::*;
//use std::ops::DerefMut;
//use opencv::prelude::Vector;

fn get_vaapi_format(
    _codec: &mut ffmpeg::AVCodecContext,
    formats: &[ffmpeg::AVPixelFormat],
) -> ffmpeg::AVPixelFormat {
    for format in formats {
        if *format == ffmpeg::AVPixelFormat::AV_PIX_FMT_VAAPI {
            return *format;
        }
    }

    eprintln!("Unable to decode this file using VA-API.");
    ffmpeg::AVPixelFormat::AV_PIX_FMT_NONE
}

fn open_input_file(filename: &str) -> (ffmpeg::AVFormatContext, usize, ffmpeg::AVCodecContext) {
    let hw_device =
        ffmpeg::AVBuffer::new_hw_device(ffmpeg::AVHWDeviceType::AV_HWDEVICE_TYPE_VAAPI).unwrap();

    let mut ifmt_ctx = ffmpeg::AVFormatContext::new_input(filename).unwrap();
    ifmt_ctx.find_stream_info().unwrap();
    let (video_stream, decoder) = ifmt_ctx
        .find_best_stream(ffmpeg::AVMediaType::AVMEDIA_TYPE_VIDEO, -1, -1, 0)
        .unwrap();
    let mut video = ifmt_ctx.get_stream(video_stream).unwrap();

    let mut decoder_ctx = ffmpeg::AVCodecContext::new(&decoder).unwrap();
    decoder_ctx
        .parameters_to_context(&video.codecpar())
        .unwrap();
    decoder_ctx.set_hw_device_ctx(hw_device).unwrap();
    decoder_ctx.set_get_format(Some(get_vaapi_format));
    decoder_ctx.open2(&decoder).unwrap();

    (ifmt_ctx, video_stream, decoder_ctx)
}

struct FFMpegContext {
    initialized: bool,
    enc_codec: ffmpeg::AVCodec,
    video_stream: usize,
    decoder_ctx: ffmpeg::AVCodecContext,
    encoder_ctx: ffmpeg::AVCodecContext,
    ifmt_ctx: ffmpeg::AVFormatContext,
    ofmt_ctx: ffmpeg::AVFormatContext,
}

fn encode_write(ctx: &mut FFMpegContext, frame: Option<&ffmpeg::AVFrame>) {
    let mut enc_pkt = ffmpeg::AVPacket::default();

    ctx.encoder_ctx.send_frame(frame).unwrap();

    loop {
        if !ctx.encoder_ctx.receive_packet(&mut enc_pkt).unwrap() {
            break;
        }

        enc_pkt.set_stream_index(0);
        enc_pkt.rescale_ts(
            ctx.ifmt_ctx
                .get_stream(ctx.video_stream)
                .unwrap()
                .time_base(),
            ctx.ofmt_ctx.get_stream(0).unwrap().time_base(),
        );
        ctx.ofmt_ctx.interleaved_write_frame(&mut enc_pkt).unwrap();
    }
}

fn dec_enc(ctx: &mut FFMpegContext, pkt: &ffmpeg::AVPacket) {
    ctx.decoder_ctx.send_packet(pkt).unwrap();

    loop {
        let mut frame = ffmpeg::AVFrame::new().unwrap();
        if !ctx.decoder_ctx.receive_frame(&mut frame).unwrap() {
            break;
        }

        if !ctx.initialized {
            ctx.encoder_ctx
                .init_encoder_from_decoder(&mut ctx.decoder_ctx, &ctx.enc_codec)
                .unwrap();
            ctx.encoder_ctx.open2(&ctx.enc_codec).unwrap();

            let mut ost = ctx.ofmt_ctx.new_stream(&ctx.enc_codec).unwrap();
            ost.set_time_base(ctx.encoder_ctx.get_time_base());
            ost.codecpar().set_from_context(&ctx.encoder_ctx).unwrap();

            ctx.ofmt_ctx.write_header().unwrap();

            ctx.initialized = true;
        }

        encode_write(ctx, Some(&frame));
    }
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
    //let samples = cfg.load_data().expect("can't read samples");

    // init render context
    //let filename = cfg.video.filename.clone();
    //let mut renderctx = render::Context::new(&cfg, Some(&samples));

    match matches.value_of("mode").unwrap() {
        "plot" => {
            // plot
            //renderctx.plot().expect("can't plot");
        }
        /*"frame" => {
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
        }*/
        "video" => {
            let filename_in = &cfg.video.filename.as_ref().unwrap();
            let filename_out = "/home/m1cha/out.mp4";
            let (ifmt_ctx, video_stream, decoder_ctx) = open_input_file(filename_in);
            let enc_codec = ffmpeg::AVCodec::by_name("h264_vaapi").unwrap();
            let mut ofmt_ctx = ffmpeg::AVFormatContext::new_output(filename_out).unwrap();
            let encoder_ctx = ffmpeg::AVCodecContext::new(&enc_codec).unwrap();
            ofmt_ctx
                .set_pb(ffmpeg::AVIOContext::new(filename_out, ffmpeg::AVIO_FLAG_WRITE).unwrap());

            let mut ctx = FFMpegContext {
                initialized: false,
                enc_codec,
                video_stream,
                decoder_ctx,
                encoder_ctx,
                ifmt_ctx,
                ofmt_ctx,
            };

            let mut dec_pkt = ffmpeg::AVPacket::empty();
            loop {
                if !ctx.ifmt_ctx.read_frame(&mut dec_pkt).unwrap() {
                    break;
                }

                if dec_pkt.stream_index().unwrap() == video_stream {
                    dec_enc(&mut ctx, &dec_pkt);
                }
            }

            // flush decoder
            dec_pkt.null_data();
            dec_enc(&mut ctx, &dec_pkt);

            // flush encoder
            encode_write(&mut ctx, None);

            ctx.ofmt_ctx.write_trailer().unwrap();

            /*            let now = std::time::Instant::now();
            if false {
                renderctx.render_video("/tmp/out.mkv").unwrap();
            } else {
                let window = "video capture";
                opencv::highgui::named_window(window, 1).unwrap();

                // open videofile
                let mut cap = opencv::videoio::VideoCapture::new_from_file_with_backend(
                    &format!("{}", filename.unwrap()),
                    opencv::videoio::CAP_ANY,
                )
                .unwrap();
                let opened = opencv::videoio::VideoCapture::is_opened(&cap).unwrap();
                if !opened {
                    panic!("Unable to open default camera!");
                }

                // get basic video info
                let frame_width = cap
                    .get(opencv::videoio::CAP_PROP_FRAME_WIDTH)
                    .unwrap()
                    .ceil() as i32;
                let frame_height = cap
                    .get(opencv::videoio::CAP_PROP_FRAME_HEIGHT)
                    .unwrap()
                    .ceil() as i32;
                let fps = cap.get(opencv::videoio::CAP_PROP_FPS).unwrap();
                let fourcc = cap.get(opencv::videoio::CAP_PROP_FOURCC).unwrap() as i32;

                let mut surface =
                    cairo::ImageSurface::create(cairo::Format::ARgb32, frame_width, frame_height)
                        .expect("Can't create surface");
                let mut hud = opencv::core::Mat::new_rows_cols_with_data(
                    frame_height,
                    frame_width,
                    opencv::core::CV_8UC4,
                    unsafe {
                        &mut *(surface.get_data().unwrap().as_mut_ptr()
                            as *mut std::os::raw::c_void)
                    },
                    opencv::core::Mat_AUTO_STEP,
                )
                .unwrap();

                // create writer
                let mut videowriter = opencv::videoio::VideoWriter::new(
                    "/tmp/out.mkv",
                    fourcc,
                    fps,
                    opencv::core::Size::new(frame_width, frame_height),
                    true,
                )
                .unwrap();

                cap.set(
                    opencv::videoio::CAP_PROP_POS_MSEC,
                    cfg.video.startoff as f64,
                )
                .unwrap();

                let now = std::time::Instant::now();
                loop {
                    let msec = cap.get(opencv::videoio::CAP_PROP_POS_MSEC).unwrap();
                    let usec = (msec * 1000.0).round() as u64;
                    if let Some(endoff) = cfg.video.endoff {
                        if usec > endoff * 1000 {
                            break;
                        }
                    }

                    // read frame
                    //let now = std::time::Instant::now();
                    let mut frame = opencv::core::Mat::default().unwrap();
                    cap.read(&mut frame).unwrap();
                    //println!("READ: {}", now.elapsed().as_millis());
                    assert_eq!(frame.rows().unwrap(), frame_height);
                    assert_eq!(frame.cols().unwrap(), frame_width);

                    // render HUD
                    {
                        renderctx.set_ts(usec).expect("can't set timestamp");

                        let cr = cairo::Context::new(&surface);
                        cr.set_antialias(cairo::Antialias::Best);
                        renderctx.render(&cr).expect("can't render");
                        surface.flush();
                    }

                    let mut hud_bgr = unsafe{opencv::core::Mat::new_rows_cols(frame_height, frame_width, opencv::core::CV_8UC3)}.unwrap();
                    let mut hud_a = unsafe{opencv::core::Mat::new_rows_cols(frame_height, frame_width, opencv::core::CV_8UC1)}.unwrap();
                    let mut dst = opencv::types::VectorOfMat::new();
                    dst.push(hud_bgr);
                    dst.push(hud_a);
                    let mut from_to = opencv::types::VectorOfint::new();
                    from_to.push(0);
                    from_to.push(0);
                    from_to.push(1);
                    from_to.push(1);
                    from_to.push(2);
                    from_to.push(2);
                    from_to.push(3);
                    from_to.push(3);
                    opencv::core::mix_channels(&hud, &mut dst, &from_to).unwrap();
                    let mut hud_bgr = dst.get(0).unwrap();
                    let mut hud_a = dst.get(1).unwrap();
                    hud_a.add_scalar(hud_a.to_mat(), -255, );

                    //opencv::imgproc::blend_linear(&frame, &hud, &1.0, &1.0, &vec![0, 1, 2, 3]).unwrap();

                    /*
                    #[inline]
                    fn culma(c: u8, a: f64) -> u8 {
                        let cn = (c as f64) * (1.0f64 - a);
                        cn.round() as u8
                    }

                    //unsafe {frame.convert_to(&mut frame, 0, 0.0, 0.0)};


                    let now = std::time::Instant::now();

                    let surface_data = surface.get_data().unwrap();
                    let framedata: &mut [opencv::core::Vec3b] = frame.data_typed_mut().unwrap();

                    // blend HUD on video frame
                    for k in 0..(frame_width * frame_height) as usize {
                        let px_frame = &mut framedata[k];
                        let px_cairo = &surface_data[k * 4..k * 4 + 4];
                        //let a = 1.0f64 / 255.0f64 * (px_cairo[3] as f64);

                        px_frame[0] = px_cairo[0] + ((px_frame[0] as f64 * (255 - px_cairo[3]) as f64) / 255.0).round() as u8;
                        px_frame[1] = px_cairo[1] + ((px_frame[1] as f64 * (255 - px_cairo[3]) as f64) / 255.0).round() as u8;
                        px_frame[2] = px_cairo[2] + ((px_frame[2] as f64 * (255 - px_cairo[3]) as f64) / 255.0).round() as u8;
                    }

                    println!("CONVERT: {}", now.elapsed().as_millis());*/

                    // write
                    //videowriter.write(&hud).unwrap();

                    opencv::highgui::imshow(window, &hud_a).unwrap();
                    if opencv::highgui::wait_key(10).unwrap() > 0 {
                        break;
                    }

                    //break;
                }
                println!("TIME: {}", now.elapsed().as_millis());
            }
            println!("TOTAL: {}", now.elapsed().as_millis());
            */
        }
        mode => {
            eprintln!("invalid mode: {}", mode);
        }
    }
}
