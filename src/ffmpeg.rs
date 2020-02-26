pub use ffmpeg4_sys::AVHWDeviceType;
pub use ffmpeg4_sys::AVMediaType;
pub use ffmpeg4_sys::AVPixelFormat;
pub use ffmpeg4_sys::AVRational;
pub use ffmpeg4_sys::AVERROR;
pub use ffmpeg4_sys::AVERROR_EOF;
pub use ffmpeg4_sys::AVIO_FLAG_READ;
pub use ffmpeg4_sys::AVIO_FLAG_WRITE;
pub use ffmpeg4_sys::AV_BUFFERSRC_FLAG_KEEP_REF;
pub use ffmpeg4_sys::EAGAIN;

pub struct AVError {
    pub code: std::os::raw::c_int,
}

impl std::fmt::Debug for AVError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut v = Vec::with_capacity(ffmpeg4_sys::AV_ERROR_MAX_STRING_SIZE);
        let ptr = v.as_mut_ptr();
        let rc = unsafe { ffmpeg4_sys::av_strerror(self.code, ptr, v.capacity()) };
        if rc == 0 {
            let cstr = unsafe { std::ffi::CStr::from_ptr(ptr) };
            write!(f, "AVError({})", cstr.to_str().unwrap())
        } else {
            write!(f, "AVError({})", self.code)
        }
    }
}

impl PartialEq<std::os::raw::c_int> for AVError {
    fn eq(&self, other: &std::os::raw::c_int) -> bool {
        &self.code == other
    }
}

impl From<std::os::raw::c_int> for AVError {
    fn from(code: std::os::raw::c_int) -> Self {
        Self { code }
    }
}

#[derive(Debug)]
pub enum Error {
    AV(AVError),
    Nul(std::ffi::NulError),
    StreamNotFound,
    NullReturnValue,
    InvalidPacket,
}

/*impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::AV(e) => write!(),
            _ =>
        }
        write!(f, "Point {{ x: {}, y: {} }}", self.x, self.y)
    }
}*/

impl From<std::ffi::NulError> for Error {
    fn from(e: std::ffi::NulError) -> Self {
        Self::Nul(e)
    }
}

#[derive(Debug)]
pub struct AVBuffer {
    ptr: *mut ffmpeg4_sys::AVBufferRef,
}

impl AVBuffer {
    pub fn new_hw_device(type_: AVHWDeviceType) -> Result<Self, Error> {
        let mut ptr: *mut ffmpeg4_sys::AVBufferRef = std::ptr::null_mut();

        let rc = unsafe {
            ffmpeg4_sys::av_hwdevice_ctx_create(
                &mut ptr,
                type_,
                std::ptr::null(),
                std::ptr::null_mut(),
                0,
            )
        };
        if rc != 0 {
            Err(Error::AV(rc.into()))
        } else {
            assert!(!ptr.is_null());
            Ok(Self { ptr })
        }
    }
}

impl Drop for AVBuffer {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffmpeg4_sys::av_buffer_unref(&mut self.ptr) }
            assert!(self.ptr.is_null());
        }
    }
}

#[derive(Debug)]
pub struct AVFormatContext {
    ptr: *mut ffmpeg4_sys::AVFormatContext,
}

impl AVFormatContext {
    pub fn new_input(url: &str) -> Result<AVFormatContext, Error> {
        let url = std::ffi::CString::new(url)?;

        let mut ptr: *mut ffmpeg4_sys::AVFormatContext = std::ptr::null_mut();
        let rc = unsafe {
            ffmpeg4_sys::avformat_open_input(
                &mut ptr,
                url.as_ptr(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        };
        if rc != 0 {
            Err(Error::AV(rc.into()))
        } else {
            assert!(!ptr.is_null());
            Ok(Self { ptr })
        }
    }

    pub fn new_output(filename: &str) -> Result<AVFormatContext, Error> {
        let filename = std::ffi::CString::new(filename)?;

        let mut ptr: *mut ffmpeg4_sys::AVFormatContext = std::ptr::null_mut();
        let rc = unsafe {
            ffmpeg4_sys::avformat_alloc_output_context2(
                &mut ptr,
                std::ptr::null_mut(),
                std::ptr::null(),
                filename.as_ptr(),
            )
        };
        if rc != 0 {
            Err(Error::AV(rc.into()))
        } else {
            assert!(!ptr.is_null());
            Ok(Self { ptr })
        }
    }

    pub fn find_stream_info(&mut self) -> Result<(), Error> {
        assert!(!self.ptr.is_null());
        let rc = unsafe { ffmpeg4_sys::avformat_find_stream_info(self.ptr, std::ptr::null_mut()) };
        if rc != 0 {
            Err(Error::AV(rc.into()))
        } else {
            Ok(())
        }
    }

    pub fn find_best_stream(
        &self,
        type_: ffmpeg4_sys::AVMediaType,
        wanted_stream_nb: std::os::raw::c_int,
        related_stream: std::os::raw::c_int,
        flags: std::os::raw::c_int,
    ) -> Result<(usize, AVCodec), Error> {
        assert!(!self.ptr.is_null());
        let mut decoder: *mut ffmpeg4_sys::AVCodec = std::ptr::null_mut();
        let rc = unsafe {
            ffmpeg4_sys::av_find_best_stream(
                self.ptr,
                type_,
                wanted_stream_nb,
                related_stream,
                &mut decoder,
                flags,
            )
        };
        if rc < 0 {
            Err(Error::AV(rc.into()))
        } else {
            assert!(!decoder.is_null());
            Ok((rc as usize, AVCodec { ptr: decoder }))
        }
    }

    pub fn get_stream(&self, id: usize) -> Result<AVStream<'_>, Error> {
        assert!(!self.ptr.is_null());
        let ptr = unsafe { &*(self.ptr) };
        let streams =
            unsafe { std::slice::from_raw_parts_mut(ptr.streams, ptr.nb_streams as usize) };
        Ok(AVStream {
            ptr: *streams.get_mut(id).ok_or(Error::StreamNotFound)?,
            phantom: std::marker::PhantomData,
        })
    }

    pub fn new_stream<'a>(&'a mut self, codec: &AVCodec) -> Result<AVStream<'a>, Error> {
        assert!(!self.ptr.is_null());
        assert!(!codec.ptr.is_null());
        let raw_ptr = unsafe { ffmpeg4_sys::avformat_new_stream(self.ptr, codec.ptr) };
        if raw_ptr.is_null() {
            Err(Error::NullReturnValue)
        } else {
            Ok(AVStream {
                ptr: raw_ptr,
                phantom: std::marker::PhantomData,
            })
        }
    }

    pub fn set_pb(&mut self, pb: AVIOContext) {
        assert!(!self.ptr.is_null());
        let ptr = unsafe { &mut *(self.ptr) };

        // free old value
        if !ptr.pb.is_null() {
            AVIOContext::free_raw(&mut ptr.pb);
        }

        // move ptr from pb ctx to pb
        ptr.pb = pb.into_raw();
    }

    pub fn read_frame(&mut self, pkt: &mut AVPacket) -> Result<(), Error> {
        assert!(!self.ptr.is_null());
        assert!(!pkt.ptr.is_null());

        pkt.invalidate();

        let rc = unsafe { ffmpeg4_sys::av_read_frame(self.ptr, pkt.ptr) };
        if rc == 0 {
            pkt.valid = true;
            Ok(())
        } else {
            Err(Error::AV(rc.into()))
        }
    }

    pub fn write_header(&mut self) -> Result<(), Error> {
        assert!(!self.ptr.is_null());

        let rc = unsafe { ffmpeg4_sys::avformat_write_header(self.ptr, std::ptr::null_mut()) };
        if rc != 0 {
            Err(Error::AV(rc.into()))
        } else {
            Ok(())
        }
    }

    pub fn interleaved_write_frame(&mut self, pkt: &mut AVPacket) -> Result<(), Error> {
        assert!(!self.ptr.is_null());
        assert!(!pkt.ptr.is_null());

        let rc = unsafe { ffmpeg4_sys::av_interleaved_write_frame(self.ptr, pkt.ptr) };
        if rc != 0 {
            Err(Error::AV(rc.into()))
        } else {
            Ok(())
        }
    }

    pub fn write_trailer(&mut self) -> Result<(), Error> {
        assert!(!self.ptr.is_null());

        let rc = unsafe { ffmpeg4_sys::av_write_trailer(self.ptr) };
        if rc != 0 {
            Err(Error::AV(rc.into()))
        } else {
            Ok(())
        }
    }
}

impl Drop for AVFormatContext {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffmpeg4_sys::avformat_close_input(&mut self.ptr) }
            assert!(self.ptr.is_null());
        }
    }
}

#[derive(Debug)]
pub struct AVCodec {
    ptr: *const ffmpeg4_sys::AVCodec,
}

impl AVCodec {
    pub fn by_name(name: &str) -> Result<Self, Error> {
        let name = std::ffi::CString::new(name)?;
        let ptr = unsafe { ffmpeg4_sys::avcodec_find_encoder_by_name(name.as_ptr()) };

        if ptr.is_null() {
            Err(Error::NullReturnValue)
        } else {
            Ok(Self { ptr })
        }
    }
}

#[derive(Debug)]
pub struct AVStream<'a> {
    ptr: *mut ffmpeg4_sys::AVStream,
    phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> AVStream<'a> {
    pub fn codecpar(&mut self) -> AVCodecParameters<'_> {
        assert!(!self.ptr.is_null());

        let ptr = unsafe { &*(self.ptr) };
        assert!(!ptr.codecpar.is_null());

        AVCodecParameters {
            ptr: ptr.codecpar,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn time_base(&self) -> ffmpeg4_sys::AVRational {
        assert!(!self.ptr.is_null());

        let ptr = unsafe { &mut *(self.ptr) };
        ptr.time_base
    }

    pub fn set_time_base(&mut self, time_base: ffmpeg4_sys::AVRational) {
        assert!(!self.ptr.is_null());

        let ptr = unsafe { &mut *(self.ptr) };
        ptr.time_base = time_base;
    }

    pub fn get_time_base(&self) -> AVRational {
        assert!(!self.ptr.is_null());
        unsafe { &*(self.ptr) }.time_base
    }
}

#[derive(Debug)]
pub struct AVCodecParameters<'a> {
    ptr: *mut ffmpeg4_sys::AVCodecParameters,
    phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> AVCodecParameters<'a> {
    pub fn set_from_context(&mut self, codec_ctx: &AVCodecContext) -> Result<(), Error> {
        assert!(!self.ptr.is_null());
        assert!(!codec_ctx.ptr.is_null());

        let rc = unsafe { ffmpeg4_sys::avcodec_parameters_from_context(self.ptr, codec_ctx.ptr) };
        if rc != 0 {
            Err(Error::AV(rc.into()))
        } else {
            Ok(())
        }
    }
}

pub struct AVCodecContextOpaque {
    get_format: Option<
        fn(&mut AVCodecContext, &[ffmpeg4_sys::AVPixelFormat]) -> ffmpeg4_sys::AVPixelFormat,
    >,
}

pub struct AVCodecContext {
    ptr: *mut ffmpeg4_sys::AVCodecContext,
}

impl AVCodecContext {
    pub fn new(codec: &AVCodec) -> Result<Self, Error> {
        let ptr = unsafe { ffmpeg4_sys::avcodec_alloc_context3(codec.ptr) };
        if ptr.is_null() {
            Err(Error::NullReturnValue)
        } else {
            let opaque = Box::new(AVCodecContextOpaque { get_format: None });
            unsafe { &mut *ptr }.opaque = Box::into_raw(opaque) as *mut std::ffi::c_void;
            Ok(Self { ptr })
        }
    }

    fn opaque(&mut self) -> &mut AVCodecContextOpaque {
        assert!(!self.ptr.is_null());

        let ptr = unsafe { &*(self.ptr) };
        assert!(!ptr.opaque.is_null());

        unsafe { &mut *(ptr.opaque as *mut AVCodecContextOpaque) }
    }

    pub fn parameters_to_context(&mut self, par: &AVCodecParameters) -> Result<(), Error> {
        assert!(!self.ptr.is_null());
        assert!(!par.ptr.is_null());

        let rc = unsafe { ffmpeg4_sys::avcodec_parameters_to_context(self.ptr, par.ptr) };
        if rc != 0 {
            Err(Error::AV(rc.into()))
        } else {
            Ok(())
        }
    }

    pub fn set_hw_device_ctx(&mut self, hw_device_ctx: AVBuffer) -> Result<(), Error> {
        assert!(!self.ptr.is_null());
        assert!(!hw_device_ctx.ptr.is_null());

        let raw_hw_device_ctx = unsafe { ffmpeg4_sys::av_buffer_ref(hw_device_ctx.ptr) };
        if raw_hw_device_ctx.is_null() {
            Err(Error::NullReturnValue)
        } else {
            unsafe { &mut *(self.ptr) }.hw_device_ctx = raw_hw_device_ctx;
            Ok(())
        }
    }

    pub fn init_encoder_from_decoder(
        &mut self,
        decoder: &mut Self,
        codec: &AVCodec,
    ) -> Result<(), Error> {
        assert!(!self.ptr.is_null());
        assert!(!decoder.ptr.is_null());
        assert!(!codec.ptr.is_null());

        let raw_encoder = unsafe { &mut *(self.ptr) };
        let raw_decoder = unsafe { &mut *(decoder.ptr) };

        raw_encoder.time_base = unsafe { ffmpeg4_sys::av_inv_q(raw_decoder.framerate) };
        raw_encoder.pix_fmt = raw_decoder.pix_fmt;
        raw_encoder.width = raw_decoder.width;
        raw_encoder.height = raw_decoder.height;

        Ok(())
    }

    pub fn open2(&mut self, codec: &AVCodec) -> Result<(), Error> {
        assert!(!self.ptr.is_null());

        let rc = unsafe { ffmpeg4_sys::avcodec_open2(self.ptr, codec.ptr, std::ptr::null_mut()) };
        if rc != 0 {
            Err(Error::AV(rc.into()))
        } else {
            Ok(())
        }
    }

    pub fn set_get_format(
        &mut self,
        get_format: Option<
            fn(&mut AVCodecContext, &[ffmpeg4_sys::AVPixelFormat]) -> ffmpeg4_sys::AVPixelFormat,
        >,
    ) {
        assert!(!self.ptr.is_null());

        self.opaque().get_format = get_format;
        unsafe { &mut *(self.ptr) }.get_format = match get_format {
            Some(_) => Some(Self::get_format),
            None => None,
        };
    }

    unsafe extern "C" fn get_format(
        raw_ptr: *mut ffmpeg4_sys::AVCodecContext,
        fmt: *const ffmpeg4_sys::AVPixelFormat,
    ) -> ffmpeg4_sys::AVPixelFormat {
        assert!(!raw_ptr.is_null());
        assert!(!fmt.is_null());

        let ptr = &*raw_ptr;
        assert!(!ptr.opaque.is_null());

        let opaque = &*(ptr.opaque as *mut AVCodecContextOpaque);

        let mut i: usize = 0;
        loop {
            if *fmt.add(i) == ffmpeg4_sys::AVPixelFormat::AV_PIX_FMT_NONE {
                break;
            }

            i += 1;
        }

        let formats = std::slice::from_raw_parts(fmt, i);

        let mut codec_ctx = AVCodecContext { ptr: raw_ptr };
        let ret = opaque.get_format.unwrap()(&mut codec_ctx, formats);
        codec_ctx.ptr = std::ptr::null_mut();
        ret
    }

    pub fn send_packet(&mut self, pkt: &AVPacket) -> Result<(), Error> {
        assert!(!self.ptr.is_null());
        assert!(!pkt.ptr.is_null());

        if !pkt.valid {
            return Err(Error::InvalidPacket);
        }

        let rc = unsafe { ffmpeg4_sys::avcodec_send_packet(self.ptr, pkt.ptr) };
        if rc != 0 {
            Err(Error::AV(rc.into()))
        } else {
            Ok(())
        }
    }

    pub fn receive_packet(&mut self, pkt: &mut AVPacket) -> Result<(), Error> {
        assert!(!self.ptr.is_null());
        assert!(!pkt.ptr.is_null());

        let rc = unsafe { ffmpeg4_sys::avcodec_receive_packet(self.ptr, pkt.ptr) };
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::AV(rc.into()))
        }
    }

    pub fn receive_frame(&mut self, frame: &mut AVFrame) -> Result<(), Error> {
        assert!(!self.ptr.is_null());
        assert!(!frame.ptr.is_null());

        frame.unref_data();

        let rc = unsafe { ffmpeg4_sys::avcodec_receive_frame(self.ptr, frame.ptr) };
        if rc == 0 {
            frame.refed = true;
            Ok(())
        } else {
            Err(Error::AV(rc.into()))
        }
    }

    pub fn send_frame(&mut self, frame: Option<&AVFrame>) -> Result<(), Error> {
        assert!(!self.ptr.is_null());

        let frame_ptr = match frame {
            Some(frame) => {
                assert!(!frame.ptr.is_null());
                frame.ptr
            }
            None => std::ptr::null(),
        };

        let rc = unsafe { ffmpeg4_sys::avcodec_send_frame(self.ptr, frame_ptr) };
        if rc != 0 {
            Err(Error::AV(rc.into()))
        } else {
            Ok(())
        }
    }

    pub fn get_time_base(&self) -> AVRational {
        assert!(!self.ptr.is_null());
        unsafe { &*(self.ptr) }.time_base
    }

    pub fn width(&self) -> std::os::raw::c_int {
        assert!(!self.ptr.is_null());
        unsafe { &*(self.ptr) }.width
    }

    pub fn height(&self) -> std::os::raw::c_int {
        assert!(!self.ptr.is_null());
        unsafe { &*(self.ptr) }.height
    }

    pub fn pix_fmt(&self) -> ffmpeg4_sys::AVPixelFormat {
        assert!(!self.ptr.is_null());
        unsafe { &*(self.ptr) }.pix_fmt
    }

    pub fn get_sample_aspect_ratio(&self) -> AVRational {
        assert!(!self.ptr.is_null());
        unsafe { &*(self.ptr) }.sample_aspect_ratio
    }
}

impl Drop for AVCodecContext {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            let ptr = unsafe { &mut *(self.ptr) };

            assert!(!ptr.opaque.is_null());
            let _ = unsafe { Box::from_raw(ptr.opaque as *mut AVCodecContextOpaque) };
            ptr.opaque = std::ptr::null_mut();

            unsafe { ffmpeg4_sys::avcodec_free_context(&mut self.ptr) };
            assert!(self.ptr.is_null());
        }
    }
}

pub struct AVIOContext {
    ptr: *mut ffmpeg4_sys::AVIOContext,
}

impl AVIOContext {
    pub fn new(url: &str, flags: std::os::raw::c_int) -> Result<AVIOContext, Error> {
        let url = std::ffi::CString::new(url)?;

        let mut ptr: *mut ffmpeg4_sys::AVIOContext = std::ptr::null_mut();
        let rc = unsafe { ffmpeg4_sys::avio_open(&mut ptr, url.as_ptr(), flags) };
        if rc != 0 {
            Err(Error::AV(rc.into()))
        } else {
            assert!(!ptr.is_null());
            Ok(Self { ptr })
        }
    }

    fn into_raw(mut self) -> *mut ffmpeg4_sys::AVIOContext {
        assert!(!self.ptr.is_null());
        let ptr = self.ptr;
        self.ptr = std::ptr::null_mut();
        ptr
    }

    fn free_raw(ptr: *mut *mut ffmpeg4_sys::AVIOContext) {
        if !ptr.is_null() {
            let rc = unsafe { ffmpeg4_sys::avio_closep(ptr) };
            assert_eq!(rc, 0);
        }
    }
}

impl Drop for AVIOContext {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            Self::free_raw(&mut self.ptr);
        }
    }
}

pub struct AVPacket {
    ptr: *mut ffmpeg4_sys::AVPacket,
    valid: bool,
}

impl Default for AVPacket {
    fn default() -> Self {
        let raw_box = Box::new(unsafe { std::mem::zeroed() });
        let raw_pkt = Box::into_raw(raw_box);
        unsafe { ffmpeg4_sys::av_init_packet(raw_pkt) };
        Self {
            ptr: raw_pkt,
            valid: true,
        }
    }
}

impl Drop for AVPacket {
    fn drop(&mut self) {
        assert!(!self.ptr.is_null());
        self.invalidate();
        unsafe { Box::from_raw(self.ptr) };
    }
}

impl AVPacket {
    pub fn empty() -> Self {
        let raw_box = Box::new(unsafe { std::mem::zeroed() });
        Self {
            ptr: Box::into_raw(raw_box),
            valid: false,
        }
    }

    fn invalidate(&mut self) {
        assert!(!self.ptr.is_null());
        if self.valid {
            unsafe { ffmpeg4_sys::av_packet_unref(self.ptr) }
            self.valid = false;
        }
    }

    pub fn stream_index(&self) -> Option<usize> {
        assert!(!self.ptr.is_null());
        if self.valid {
            Some(unsafe { &*(self.ptr) }.stream_index as usize)
        } else {
            None
        }
    }

    pub fn set_stream_index(&mut self, stream_index: usize) {
        assert!(!self.ptr.is_null());
        unsafe { &mut *(self.ptr) }.stream_index = stream_index as std::os::raw::c_int;
    }

    pub fn rescale_ts(&mut self, tb_src: ffmpeg4_sys::AVRational, tb_dst: ffmpeg4_sys::AVRational) {
        assert!(!self.ptr.is_null());

        unsafe { ffmpeg4_sys::av_packet_rescale_ts(self.ptr, tb_src, tb_dst) };
    }
}

pub struct AVFrame {
    ptr: *mut ffmpeg4_sys::AVFrame,
    refed: bool,
}

impl AVFrame {
    pub fn new() -> Result<AVFrame, Error> {
        let ptr = unsafe { ffmpeg4_sys::av_frame_alloc() };
        if ptr.is_null() {
            Err(Error::NullReturnValue)
        } else {
            Ok(AVFrame { ptr, refed: false })
        }
    }

    fn unref_data(&mut self) {
        assert!(!self.ptr.is_null());

        if self.refed {
            unsafe { ffmpeg4_sys::av_frame_unref(self.ptr) };
        }
    }

    pub fn get_best_effort_timestamp(&self) -> i64 {
        assert!(!self.ptr.is_null());
        assert!(self.refed);

        unsafe { &*(self.ptr) }.best_effort_timestamp
    }

    pub fn set_pts(&self, pts: i64) {
        assert!(!self.ptr.is_null());
        assert!(self.refed);

        unsafe { &mut *(self.ptr) }.pts = pts;
    }
}

impl Drop for AVFrame {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffmpeg4_sys::av_frame_free(&mut self.ptr) }
        }
    }
}

pub struct AVFilter {
    ptr: *const ffmpeg4_sys::AVFilter,
}

impl AVFilter {
    pub fn by_name(name: &str) -> Result<Self, Error> {
        let name = std::ffi::CString::new(name)?;
        let ptr = unsafe { ffmpeg4_sys::avfilter_get_by_name(name.as_ptr()) };

        if ptr.is_null() {
            Err(Error::NullReturnValue)
        } else {
            Ok(Self { ptr })
        }
    }
}

struct AVFilterInOutEntry {
    io_name: std::ffi::CString,
    filterid: usize,
    pad_idx: std::os::raw::c_int,
}

pub struct AVFilterInOut {
    list: Vec<AVFilterInOutEntry>,
}

impl AVFilterInOut {
    pub fn new() -> Self {
        Self { list: Vec::new() }
    }

    pub fn append(&mut self, io_name: &str, filterid: usize, pad_idx: std::os::raw::c_int) {
        self.list.push(AVFilterInOutEntry {
            io_name: std::ffi::CString::new(io_name).unwrap(),
            filterid,
            pad_idx,
        });
    }

    fn to_raw_list(&self, graph: &AVFilterGraph) -> *mut ffmpeg4_sys::AVFilterInOut {
        assert!(!graph.ptr.is_null());

        let raw_graph = unsafe { &*(graph.ptr) };
        let filters = unsafe {
            std::slice::from_raw_parts_mut(raw_graph.filters, raw_graph.nb_filters as usize)
        };

        let mut raw_list: *mut ffmpeg4_sys::AVFilterInOut = std::ptr::null_mut();

        for entry in self.list.iter().rev() {
            let raw_entry = unsafe { ffmpeg4_sys::avfilter_inout_alloc() };
            assert!(!raw_entry.is_null());

            let raw_entry_ref = unsafe { &mut *raw_entry };
            raw_entry_ref.name = unsafe { ffmpeg4_sys::av_strdup(entry.io_name.as_ptr()) };
            assert!(!raw_entry_ref.name.is_null());
            raw_entry_ref.filter_ctx = filters[entry.filterid];
            assert!(!raw_entry_ref.filter_ctx.is_null());
            raw_entry_ref.pad_idx = entry.pad_idx;
            raw_entry_ref.next = raw_list;

            raw_list = raw_entry;
        }

        raw_list
    }
}

pub struct AVFilterGraph {
    ptr: *mut ffmpeg4_sys::AVFilterGraph,
}

impl AVFilterGraph {
    pub fn new() -> Result<Self, Error> {
        let ptr = unsafe { ffmpeg4_sys::avfilter_graph_alloc() };

        if ptr.is_null() {
            Err(Error::NullReturnValue)
        } else {
            Ok(Self { ptr })
        }
    }

    pub fn create_filter(
        &mut self,
        filter: &AVFilter,
        name: &str,
        args: Option<&str>,
    ) -> Result<usize, Error> {
        assert!(!self.ptr.is_null());
        assert!(!filter.ptr.is_null());

        println!("{:?}", args);

        let name = std::ffi::CString::new(name)?;
        let args = match args {
            Some(v) => Some(std::ffi::CString::new(v)?),
            None => None,
        };
        let args_ptr = match &args {
            Some(v) => v.as_ptr(),
            None => std::ptr::null(),
        };

        let mut ptr: *mut ffmpeg4_sys::AVFilterContext = std::ptr::null_mut();
        let rc = unsafe {
            ffmpeg4_sys::avfilter_graph_create_filter(
                &mut ptr,
                filter.ptr,
                name.as_ptr(),
                args_ptr,
                std::ptr::null_mut(),
                self.ptr,
            )
        };
        if rc != 0 {
            Err(Error::AV(rc.into()))
        } else {
            let raw_graph = unsafe { &*(self.ptr) };
            assert!(!ptr.is_null());
            assert!(raw_graph.nb_filters > 0);
            let filterid = raw_graph.nb_filters as usize - 1;
            assert_eq!(unsafe { *(raw_graph.filters.add(filterid)) }, ptr);
            Ok(filterid)
        }
    }

    pub fn parse_ptr(
        &mut self,
        filters: &str,
        inputs: &AVFilterInOut,
        outputs: &AVFilterInOut,
    ) -> Result<(), Error> {
        assert!(!self.ptr.is_null());
        let filters = std::ffi::CString::new(filters)?;

        let mut raw_inputs = inputs.to_raw_list(self);
        let mut raw_outputs = outputs.to_raw_list(self);

        let rc = unsafe {
            ffmpeg4_sys::avfilter_graph_parse_ptr(
                self.ptr,
                filters.as_ptr(),
                &mut raw_inputs,
                &mut raw_outputs,
                std::ptr::null_mut(),
            )
        };

        unsafe { ffmpeg4_sys::avfilter_inout_free(&mut raw_inputs) };
        unsafe { ffmpeg4_sys::avfilter_inout_free(&mut raw_outputs) };

        if rc < 0 {
            Err(Error::AV(rc.into()))
        } else {
            Ok(())
        }
    }

    pub fn config(&mut self) -> Result<(), Error> {
        assert!(!self.ptr.is_null());

        let rc = unsafe { ffmpeg4_sys::avfilter_graph_config(self.ptr, std::ptr::null_mut()) };
        if rc != 0 {
            Err(Error::AV(rc.into()))
        } else {
            Ok(())
        }
    }

    fn get_filter_slice(&self) -> &[*mut ffmpeg4_sys::AVFilterContext] {
        assert!(!self.ptr.is_null());
        let raw_graph = unsafe { &*(self.ptr) };
        unsafe { std::slice::from_raw_parts_mut(raw_graph.filters, raw_graph.nb_filters as usize) }
    }

    pub fn buffersrc_add_frame_flags(
        &mut self,
        bufferid: usize,
        frame: &mut AVFrame,
        flags: std::os::raw::c_int,
    ) -> Result<(), Error> {
        assert!(!self.ptr.is_null());
        assert!(!frame.ptr.is_null());
        let filters = self.get_filter_slice();

        let rc = unsafe {
            ffmpeg4_sys::av_buffersrc_add_frame_flags(filters[bufferid], frame.ptr, flags)
        };
        if rc != 0 {
            Err(Error::AV(rc.into()))
        } else {
            Ok(())
        }
    }

    pub fn buffersink_get_frame(
        &mut self,
        bufferid: usize,
        frame: &mut AVFrame,
    ) -> Result<(), Error> {
        assert!(!self.ptr.is_null());
        assert!(!frame.ptr.is_null());
        let filters = self.get_filter_slice();

        frame.unref_data();

        let rc = unsafe { ffmpeg4_sys::av_buffersink_get_frame(filters[bufferid], frame.ptr) };
        if rc != 0 {
            Err(Error::AV(rc.into()))
        } else {
            frame.refed = true;
            Ok(())
        }
    }
}

impl Drop for AVFilterGraph {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffmpeg4_sys::avfilter_graph_free(&mut self.ptr) };
        }
    }
}
