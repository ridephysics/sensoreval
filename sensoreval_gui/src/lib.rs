mod native {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(dead_code)]
    #![allow(deref_nullptr)]
    #![allow(clippy::redundant_static_lifetimes)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

mod error;
pub use error::Error;

macro_rules! unwrap_opt_or {
    ($opt:expr, $default:expr) => {
        match $opt {
            Some(x) => x,
            None => $default,
        }
    };
}

pub struct RuntimeContext<'a> {
    ctx: Option<&'a mut native::context>,
}

impl<'a> RuntimeContext<'a> {
    pub fn set_orientation<Q, D>(&mut self, q: Q)
    where
        Q: std::ops::Index<usize, Output = D>,
        D: Copy + Into<std::os::raw::c_double>,
    {
        let ctx = self.ctx.as_mut().unwrap();

        let raw: [std::os::raw::c_double; 4] = [q[3].into(), q[0].into(), q[1].into(), q[2].into()];

        unsafe { native::sensorevalgui_native_set_orientation(*ctx, raw.as_ptr()) };
    }
}

pub trait Callback {
    fn set_ts(&mut self, _ctx: &mut RuntimeContext, _ts: u64) {}
    fn render(&mut self, _ctx: &mut RuntimeContext, _cr: &cairo::Context) {}
}

pub struct InnerContext<'a, 'b> {
    cfg: native::sensorevalgui_cfg,
    videopath: Option<std::ffi::CString>,
    callback: Option<Box<dyn Callback + 'a>>,
    rtctx: RuntimeContext<'b>,
}

pub struct Context<'a, 'b> {
    inner: Box<InnerContext<'a, 'b>>,
}

unsafe extern "C" fn native_set_ts(ts: u64, ctx: *mut ::std::os::raw::c_void) {
    let ctx = unwrap_opt_or!((ctx as *mut InnerContext).as_mut(), return);
    let callback = unwrap_opt_or!(ctx.callback.as_mut(), return);
    callback.set_ts(&mut ctx.rtctx, ts);
}

unsafe extern "C" fn native_render(cr_ptr: *mut native::cairo_t, ctx: *mut ::std::os::raw::c_void) {
    let ctx = unwrap_opt_or!((ctx as *mut InnerContext).as_mut(), return);
    let callback = unwrap_opt_or!(ctx.callback.as_mut(), return);
    let cr = cairo::Context::from_raw_borrow(cr_ptr as *mut cairo_sys::cairo_t);
    callback.render(&mut ctx.rtctx, &cr);
}

impl<'a, 'b> Default for Context<'a, 'b> {
    fn default() -> Self {
        Self {
            inner: Box::new(InnerContext {
                cfg: native::sensorevalgui_cfg {
                    timer_ms: 0,
                    orientation_enabled: false,
                    videopath: std::ptr::null(),
                    startoff: 0,
                    endoff: 0,
                    set_ts: Some(native_set_ts),
                    render: Some(native_render),
                    pdata: std::ptr::null_mut(),
                },
                videopath: None,
                callback: None,
                rtctx: RuntimeContext { ctx: None },
            }),
        }
    }
}

impl<'a, 'b> Context<'a, 'b> {
    pub fn set_timer_ms(&mut self, ms: u64) {
        self.inner.cfg.timer_ms = ms;
    }

    pub fn set_orientation_enabled(&mut self, orientation_enabled: bool) {
        self.inner.cfg.orientation_enabled = orientation_enabled;
    }

    pub fn set_startoff(&mut self, startoff: u64) {
        self.inner.cfg.startoff = startoff;
    }

    pub fn set_endoff(&mut self, endoff: u64) {
        self.inner.cfg.endoff = endoff;
    }

    pub fn set_videopath<S>(&mut self, videopath: Option<S>)
    where
        S: AsRef<str>,
    {
        self.inner.videopath = videopath.map(|v| std::ffi::CString::new(v.as_ref()).unwrap());
        self.inner.cfg.videopath = self
            .inner
            .videopath
            .as_ref()
            .map(|v| v.as_ptr())
            .unwrap_or_else(std::ptr::null)
    }

    pub fn set_callback<C>(&mut self, callback: Option<C>)
    where
        C: Callback + 'a,
    {
        self.inner.callback = if let Some(callback) = callback {
            Some(Box::new(callback))
        } else {
            None
        };
    }

    pub fn start(&mut self) -> Result<(), Error> {
        let mut nativectx = std::ptr::null_mut();
        let rc = unsafe { native::sensorevalgui_native_create(&mut nativectx, &self.inner.cfg) };
        assert_eq!(rc, 0);

        self.inner.rtctx.ctx = unsafe { nativectx.as_mut() };
        self.inner.cfg.pdata = self.inner.as_mut() as *mut InnerContext as *mut std::ffi::c_void;

        let rc = unsafe { native::sensorevalgui_native_start(nativectx) };

        self.inner.rtctx.ctx = None;
        self.inner.cfg.pdata = std::ptr::null_mut();

        unsafe { native::sensorevalgui_native_destroy(nativectx) };

        if rc == 0 {
            Ok(())
        } else {
            Err(Error::NativeReturn(rc))
        }
    }
}
