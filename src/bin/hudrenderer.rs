use sensoreval::*;

struct BufferSrc {
    ifmt_ctx: ffmpeg::AVFormatContext,
    video_stream: usize,
    decoder_ctx: ffmpeg::AVCodecContext,
    eof_ifmt: bool,
    eof_dec: bool,
    buffersrc_id: usize,
}

struct FilterContext {
    graph: ffmpeg::AVFilterGraph,
    buffersink_id: usize,
}

struct FFMpegContext {
    initialized: bool,
    enc_codec: ffmpeg::AVCodec,
    encoder_ctx: ffmpeg::AVCodecContext,
    ofmt_ctx: ffmpeg::AVFormatContext,
    filterctx: FilterContext,
}

fn open_input_file(filename: &str) -> Result<BufferSrc, ffmpeg::Error> {
    let mut ifmt_ctx = ffmpeg::AVFormatContext::new_input(filename)?;
    ifmt_ctx.find_stream_info()?;
    let (video_stream, decoder) =
        ifmt_ctx.find_best_stream(ffmpeg::AVMediaType::AVMEDIA_TYPE_VIDEO, -1, -1, 0)?;
    let mut video = ifmt_ctx.get_stream(video_stream)?;

    let mut decoder_ctx = ffmpeg::AVCodecContext::new(&decoder)?;
    decoder_ctx.parameters_to_context(&video.codecpar())?;

    decoder_ctx.open2(&decoder)?;

    Ok(BufferSrc {
        ifmt_ctx,
        video_stream,
        decoder_ctx,
        eof_ifmt: false,
        eof_dec: false,
        buffersrc_id: 0,
    })
}

fn init_filters(
    sources: &mut Vec<BufferSrc>,
    filters_descr: &str,
) -> Result<FilterContext, ffmpeg::Error> {
    let buffersrc = ffmpeg::AVFilter::by_name("buffer")?;
    let buffersink = ffmpeg::AVFilter::by_name("buffersink")?;
    let mut outputs = ffmpeg::AVFilterInOut::new();
    let mut inputs = ffmpeg::AVFilterInOut::new();
    let mut filter_graph = ffmpeg::AVFilterGraph::new()?;

    let mut vid = 0;
    for source in sources {
        let time_base = source
            .ifmt_ctx
            .get_stream(source.video_stream)?
            .get_time_base();
        let sample_aspect_ratio = source.decoder_ctx.get_sample_aspect_ratio();
        let buffersrc_id = filter_graph.create_filter(
            &buffersrc,
            &format!("in{}", vid),
            Some(&format!(
                "video_size={}x{}:pix_fmt={}:time_base={}/{}:pixel_aspect={}/{}",
                source.decoder_ctx.width(),
                source.decoder_ctx.height(),
                source.decoder_ctx.pix_fmt() as std::os::raw::c_int,
                time_base.num,
                time_base.den,
                sample_aspect_ratio.num,
                sample_aspect_ratio.den
            )),
        )?;
        outputs.append(&format!("in{}", vid), buffersrc_id, 0);
        source.buffersrc_id = buffersrc_id;

        vid += 1;
    }

    let buffersink_id = filter_graph.create_filter(&buffersink, "out", None)?;
    inputs.append("out", buffersink_id, 0);

    filter_graph.parse_ptr(filters_descr, &inputs, &outputs)?;
    filter_graph.config()?;

    Ok(FilterContext {
        graph: filter_graph,
        buffersink_id,
    })
}

fn encode_write(
    ctx: &mut FFMpegContext,
    time_base: ffmpeg::AVRational,
    frame: Option<&ffmpeg::AVFrame>,
) -> Result<(), ffmpeg::Error> {
    let mut enc_pkt = ffmpeg::AVPacket::default();

    ctx.encoder_ctx.send_frame(frame)?;

    loop {
        let res = ctx.encoder_ctx.receive_packet(&mut enc_pkt);
        match res {
            Err(ffmpeg::Error::AV(e))
                if e == ffmpeg::AVERROR_EOF || e == ffmpeg::AVERROR(ffmpeg::EAGAIN) =>
            {
                break;
            }
            _ => res?,
        };

        enc_pkt.set_stream_index(0);
        enc_pkt.rescale_ts(time_base, ctx.ofmt_ctx.get_stream(0)?.time_base());
        ctx.ofmt_ctx.interleaved_write_frame(&mut enc_pkt)?;
    }

    Ok(())
}

fn dec_to_buffersrc(
    ctx: &mut FFMpegContext,
    source: &mut BufferSrc,
    frame: &mut ffmpeg::AVFrame,
) -> Result<(), ffmpeg::Error> {
    while !source.eof_dec {
        let res = source.decoder_ctx.receive_frame(frame);
        match res {
            Err(ffmpeg::Error::AV(e)) if e == ffmpeg::AVERROR_EOF => {
                source.eof_dec = true;
                ctx.filterctx.graph.buffersrc_add_frame_flags(
                    source.buffersrc_id,
                    None,
                    ffmpeg::AV_BUFFERSRC_FLAG_PUSH as std::os::raw::c_int,
                )?;
                break;
            }
            Err(ffmpeg::Error::AV(e)) if e == ffmpeg::AVERROR(ffmpeg::EAGAIN) => {
                break;
            }
            _ => res?,
        };

        if !ctx.initialized {
            ctx.encoder_ctx
                .init_encoder_from_decoder(&mut source.decoder_ctx, &ctx.enc_codec)?;
            ctx.encoder_ctx.open2(&ctx.enc_codec)?;

            let mut ost = ctx.ofmt_ctx.new_stream(&ctx.enc_codec)?;
            ost.set_time_base(ctx.encoder_ctx.get_time_base());
            ost.codecpar().set_from_context(&ctx.encoder_ctx)?;

            ctx.ofmt_ctx.write_header()?;

            ctx.initialized = true;
        }

        frame.set_pts(frame.get_best_effort_timestamp());

        ctx.filterctx.graph.buffersrc_add_frame_flags(
            source.buffersrc_id,
            Some(frame),
            ffmpeg::AV_BUFFERSRC_FLAG_KEEP_REF as std::os::raw::c_int,
        )?;
    }

    Ok(())
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
        "video" => {
            let filename_in = &cfg.video.filename.as_ref().unwrap();
            let filename_out = "/home/m1cha/out.mp4";
            let enc_codec =
                ffmpeg::AVCodec::by_name(if false { "h264_vaapi" } else { "libx264" }).unwrap();
            let mut ofmt_ctx = ffmpeg::AVFormatContext::new_output(filename_out).unwrap();
            let encoder_ctx = ffmpeg::AVCodecContext::new(&enc_codec).unwrap();
            ofmt_ctx
                .set_pb(ffmpeg::AVIOContext::new(filename_out, ffmpeg::AVIO_FLAG_WRITE).unwrap());

            let mut sources = Vec::new();

            let mut source = open_input_file(filename_in).unwrap();
            let time_base = source
                .ifmt_ctx
                .get_stream(source.video_stream)
                .unwrap()
                .get_time_base();
            let seek_target = ffmpeg::av_rescale_q(
                (cfg.video.startoff * 1000) as i64,
                ffmpeg::AV_TIME_BASE_Q,
                time_base,
            );
            source
                .ifmt_ctx
                .seek_frame(source.video_stream as i32, seek_target)
                .unwrap();
            sources.push(source);

            sources.push(open_input_file("/tmp/out.png").unwrap());

            let filterctx = init_filters(&mut sources, "[in0]split[in0_0][in0_1];[in1]loop=loop=-1:size=1:start=0[in1l];[in0_0][in1l]alphamerge,boxblur=20[alf];[in0_1][alf]overlay[out]").unwrap();

            let mut ctx = FFMpegContext {
                initialized: false,
                enc_codec,
                encoder_ctx,
                ofmt_ctx,
                filterctx,
            };

            let mut frame = ffmpeg::AVFrame::new().unwrap();
            let mut dec_pkt = ffmpeg::AVPacket::empty();
            let mut buffersink_eof = false;
            for _ in 0..100 {
                // while !buffersink_eof {
                for source in &mut sources {
                    if !source.eof_ifmt {
                        let res = source.ifmt_ctx.read_frame(&mut dec_pkt);
                        match res {
                            Err(ffmpeg::Error::AV(e)) if e == ffmpeg::AVERROR_EOF => {
                                source.eof_ifmt = true;

                                // flush decoder
                                source
                                    .decoder_ctx
                                    .send_packet(&ffmpeg::AVPacket::default())
                                    .unwrap();

                                continue;
                            }
                            _ => res.unwrap(),
                        };

                        dec_pkt.set_dts(dec_pkt.get_dts() - seek_target);
                        dec_pkt.set_pts(dec_pkt.get_pts() - seek_target);

                        if dec_pkt.stream_index().unwrap() == source.video_stream {
                            source.decoder_ctx.send_packet(&dec_pkt).unwrap();
                        }
                    }

                    if !source.eof_dec {
                        dec_to_buffersrc(&mut ctx, source, &mut frame).unwrap();
                    }
                }

                loop {
                    let res = ctx
                        .filterctx
                        .graph
                        .buffersink_get_frame(ctx.filterctx.buffersink_id, &mut frame);
                    match res {
                        Err(ffmpeg::Error::AV(e))
                            if e == ffmpeg::AVERROR_EOF || e == ffmpeg::AVERROR(ffmpeg::EAGAIN) =>
                        {
                            buffersink_eof = true;
                            break;
                        }
                        _ => res.unwrap(),
                    };

                    encode_write(&mut ctx, time_base, Some(&mut frame)).unwrap();
                }
            }

            // flush encoder
            encode_write(&mut ctx, time_base, None).unwrap();
            ctx.ofmt_ctx.write_trailer().unwrap();

            let mut do_panic = false;
            let mut n = 0;
            for source in &sources {
                if !source.eof_ifmt {
                    println!("[{}] ifmt didn't see eof yet", n);
                    do_panic = true;
                }
                if !source.eof_dec {
                    println!("[{}] decoder didn't see eof yet", n);
                    do_panic = true;
                }

                n += 1;
            }

            if do_panic {
                panic!("sink EOF without all inputs being finished");
            }
        }
        mode => {
            eprintln!("invalid mode: {}", mode);
        }
    }
}
