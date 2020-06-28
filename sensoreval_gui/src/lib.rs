mod native {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(dead_code)]
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
    fn render(&mut self, _ctx: &mut RuntimeContext, _cr: &mut cairo::Context) {}
}

pub struct Context<'a, 'b> {
    cfg: native::sensorevalgui_cfg,
    videopath: Option<std::ffi::CString>,
    callback: Option<Box<dyn Callback + 'a>>,
    rtctx: RuntimeContext<'b>,
}

unsafe extern "C" fn native_set_ts(ts: u64, ctx: *mut ::std::os::raw::c_void) {
    let ctx = unwrap_opt_or!((ctx as *mut Context).as_mut(), return);
    let callback = unwrap_opt_or!(ctx.callback.as_mut(), return);
    callback.set_ts(&mut ctx.rtctx, ts);
}

unsafe extern "C" fn native_render(cr_ptr: *mut native::cairo_t, ctx: *mut ::std::os::raw::c_void) {
    let ctx = unwrap_opt_or!((ctx as *mut Context).as_mut(), return);
    let callback = unwrap_opt_or!(ctx.callback.as_mut(), return);
    let mut cr = cairo::Context::from_raw_borrow(cr_ptr as *mut cairo_sys::cairo_t);
    callback.render(&mut ctx.rtctx, &mut cr);
}

impl<'a, 'b> Default for Context<'a, 'b> {
    fn default() -> Self {
        Self {
            cfg: native::sensorevalgui_cfg {
                timer_ms: 0,
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
        }
    }
}

impl<'a, 'b> Context<'a, 'b> {
    pub fn set_timer_ms(&mut self, ms: u64) {
        self.cfg.timer_ms = ms;
    }

    pub fn set_startoff(&mut self, startoff: u64) {
        self.cfg.startoff = startoff;
    }

    pub fn set_endoff(&mut self, endoff: u64) {
        self.cfg.endoff = endoff;
    }

    pub fn set_videopath<S>(&mut self, videopath: Option<S>)
    where
        S: AsRef<str>,
    {
        self.videopath = videopath.map(|v| std::ffi::CString::new(v.as_ref()).unwrap());
        self.cfg.videopath = self
            .videopath
            .as_ref()
            .map(|v| v.as_ptr())
            .unwrap_or_else(|| std::ptr::null())
    }

    pub fn set_callback<C>(&mut self, callback: Option<C>)
    where
        C: Callback + 'a,
    {
        self.callback = if let Some(callback) = callback {
            Some(Box::new(callback))
        } else {
            None
        };
    }

    pub fn start(&mut self) -> Result<(), Error> {
        let mut nativectx = std::ptr::null_mut();
        let rc = unsafe { native::sensorevalgui_native_create(&mut nativectx, &self.cfg) };
        assert_eq!(rc, 0);

        self.rtctx.ctx = unsafe { nativectx.as_mut() };
        self.cfg.pdata = self as *mut Context as *mut std::ffi::c_void;

        let rc = unsafe { native::sensorevalgui_native_start(nativectx) };

        self.rtctx.ctx = None;
        self.cfg.pdata = std::ptr::null_mut();

        unsafe { native::sensorevalgui_native_destroy(nativectx) };

        if rc == 0 {
            Ok(())
        } else {
            Err(Error::NativeReturn(rc))
        }
    }
}
