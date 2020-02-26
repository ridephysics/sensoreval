use sensoreval::*;
//use std::ops::DerefMut;
//use opencv::prelude::Vector;

fn open_input_file(filename: &str) -> (ffmpeg::AVFormatContext, usize, ffmpeg::AVCodecContext) {
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
    decoder_ctx.open2(&decoder).unwrap();

    (ifmt_ctx, video_stream, decoder_ctx)
}

struct FilterContext {
    graph: ffmpeg::AVFilterGraph,
    buffersrc_id: usize,
    buffersink_id: usize,
}

fn init_filters(
    video_stream: usize,
    ifmt_ctx: &ffmpeg::AVFormatContext,
    decoder_ctx: &ffmpeg::AVCodecContext,
    filters_descr: &str,
) -> Result<FilterContext, ffmpeg::Error> {
    let buffersrc = ffmpeg::AVFilter::by_name("buffer")?;
    let buffersink = ffmpeg::AVFilter::by_name("buffersink")?;
    let mut outputs = ffmpeg::AVFilterInOut::new();
    let mut inputs = ffmpeg::AVFilterInOut::new();
    let mut filter_graph = ffmpeg::AVFilterGraph::new()?;

    let time_base = ifmt_ctx.get_stream(video_stream)?.get_time_base();
    let sample_aspect_ratio = decoder_ctx.get_sample_aspect_ratio();
    let buffersrc_id = filter_graph.create_filter(
        &buffersrc,
        "in",
        Some(&format!(
            "video_size={}x{}:pix_fmt={}:time_base={}/{}:pixel_aspect={}/{}",
            decoder_ctx.width(),
            decoder_ctx.height(),
            decoder_ctx.pix_fmt() as std::os::raw::c_int,
            time_base.num,
            time_base.den,
            sample_aspect_ratio.num,
            sample_aspect_ratio.den
        )),
    )?;
    outputs.append("in", buffersrc_id, 0);

    let buffersink_id = filter_graph.create_filter(&buffersink, "out", None)?;
    inputs.append("out", buffersink_id, 0);

    filter_graph.parse_ptr(filters_descr, &inputs, &outputs)?;
    filter_graph.config()?;

    Ok(FilterContext {
        graph: filter_graph,
        buffersrc_id,
        buffersink_id,
    })
}

struct FFMpegContext {
    initialized: bool,
    enc_codec: ffmpeg::AVCodec,
    video_stream: usize,
    decoder_ctx: ffmpeg::AVCodecContext,
    encoder_ctx: ffmpeg::AVCodecContext,
    ifmt_ctx: ffmpeg::AVFormatContext,
    ofmt_ctx: ffmpeg::AVFormatContext,
    filterctx: FilterContext,
}

fn encode_write(
    ctx: &mut FFMpegContext,
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
        enc_pkt.rescale_ts(
            ctx.ifmt_ctx.get_stream(ctx.video_stream)?.time_base(),
            ctx.ofmt_ctx.get_stream(0)?.time_base(),
        );
        ctx.ofmt_ctx.interleaved_write_frame(&mut enc_pkt)?;
    }

    Ok(())
}

fn dec_enc(
    ctx: &mut FFMpegContext,
    pkt: &ffmpeg::AVPacket,
    frame: &mut ffmpeg::AVFrame,
) -> Result<(), ffmpeg::Error> {
    ctx.decoder_ctx.send_packet(pkt)?;

    loop {
        let res = ctx.decoder_ctx.receive_frame(frame);
        match res {
            Err(ffmpeg::Error::AV(e))
                if e == ffmpeg::AVERROR_EOF || e == ffmpeg::AVERROR(ffmpeg::EAGAIN) =>
            {
                break;
            }
            _ => res?,
        };

        if !ctx.initialized {
            ctx.encoder_ctx
                .init_encoder_from_decoder(&mut ctx.decoder_ctx, &ctx.enc_codec)?;
            ctx.encoder_ctx.open2(&ctx.enc_codec)?;

            let mut ost = ctx.ofmt_ctx.new_stream(&ctx.enc_codec)?;
            ost.set_time_base(ctx.encoder_ctx.get_time_base());
            ost.codecpar().set_from_context(&ctx.encoder_ctx)?;

            ctx.ofmt_ctx.write_header()?;

            ctx.initialized = true;
        }

        frame.set_pts(frame.get_best_effort_timestamp());

        ctx.filterctx.graph.buffersrc_add_frame_flags(
            ctx.filterctx.buffersrc_id,
            frame,
            ffmpeg::AV_BUFFERSRC_FLAG_KEEP_REF as std::os::raw::c_int,
        )?;

        loop {
            let res = ctx
                .filterctx
                .graph
                .buffersink_get_frame(ctx.filterctx.buffersink_id, frame);
            match res {
                Err(ffmpeg::Error::AV(e))
                    if e == ffmpeg::AVERROR_EOF || e == ffmpeg::AVERROR(ffmpeg::EAGAIN) =>
                {
                    break;
                }
                _ => res?,
            };

            encode_write(ctx, Some(frame))?;
        }
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
            let (ifmt_ctx, video_stream, decoder_ctx) = open_input_file(filename_in);
            let enc_codec = ffmpeg::AVCodec::by_name("libx264").unwrap();
            let mut ofmt_ctx = ffmpeg::AVFormatContext::new_output(filename_out).unwrap();
            let encoder_ctx = ffmpeg::AVCodecContext::new(&enc_codec).unwrap();
            ofmt_ctx
                .set_pb(ffmpeg::AVIOContext::new(filename_out, ffmpeg::AVIO_FLAG_WRITE).unwrap());

            let filterctx = init_filters(video_stream, &ifmt_ctx, &decoder_ctx, "negate").unwrap();
            let mut ctx = FFMpegContext {
                initialized: false,
                enc_codec,
                video_stream,
                decoder_ctx,
                encoder_ctx,
                ifmt_ctx,
                ofmt_ctx,
                filterctx,
            };

            let mut frame = ffmpeg::AVFrame::new().unwrap();
            let mut dec_pkt = ffmpeg::AVPacket::empty();
            for _ in 0..100 {
                let res = ctx.ifmt_ctx.read_frame(&mut dec_pkt);
                match res {
                    Err(ffmpeg::Error::AV(e)) if e == ffmpeg::AVERROR_EOF => {
                        break;
                    }
                    _ => res.unwrap(),
                };

                if dec_pkt.stream_index().unwrap() == video_stream {
                    dec_enc(&mut ctx, &dec_pkt, &mut frame).unwrap();
                }
            }

            // flush decoder
            dec_pkt = ffmpeg::AVPacket::default();
            dec_enc(&mut ctx, &dec_pkt, &mut frame).unwrap();

            // flush encoder
            encode_write(&mut ctx, None).unwrap();

            ctx.ofmt_ctx.write_trailer().unwrap();
        }
        mode => {
            eprintln!("invalid mode: {}", mode);
        }
    }
}
