use crate::*;

pub struct CContext<'a> {
    cfg: config::Config,
    dataset: Option<Vec<Data>>,
    readctx: datareader::Context,
    renderctx: Option<render::Context<'a, 'a>>,
}

#[no_mangle]
pub extern "C" fn sensoreval_create<'a>(
    path_ptr: *const std::os::raw::c_char,
    islive: bool,
) -> *mut CContext<'a> {
    let path = unwrap_res_or!(
        unsafe { std::ffi::CStr::from_ptr(path_ptr) }.to_str(),
        return std::ptr::null_mut()
    );
    let cfg = unwrap_res_or!(config::load(&path), return std::ptr::null_mut());
    let dataset = match islive {
        true => None,
        false => Some(unwrap_res_or!(
            datareader::read_all_samples_cfg(&cfg),
            return std::ptr::null_mut()
        )),
    };

    let cctx_ptr = Box::into_raw(Box::new(CContext {
        cfg: cfg,
        dataset: dataset,
        readctx: datareader::Context::new(),
        renderctx: None,
    }));
    let cctx = unsafe { cctx_ptr.as_mut() }.unwrap();

    cctx.renderctx = Some(render::Context::new(&cctx.cfg, cctx.dataset.as_ref()));

    return cctx_ptr;
}

#[no_mangle]
pub extern "C" fn sensoreval_destroy<'a>(cctx_ptr: *mut CContext<'a>) {
    unsafe { Box::from_raw(cctx_ptr) };
}

#[no_mangle]
pub extern "C" fn sensoreval_render<'a>(
    cctx_ptr: *const CContext<'a>,
    cr_ptr: *mut cairo_sys::cairo_t,
) -> std::os::raw::c_int {
    let cctx = unwrap_opt_or!(unsafe { cctx_ptr.as_ref() }, return -1);
    let cr = unsafe { cairo::Context::from_raw_borrow(cr_ptr) };
    let renderctx = unwrap_opt_or!(&cctx.renderctx, return -2);

    unwrap_res_or!(renderctx.render(&cr), return -3);

    return 0;
}

#[no_mangle]
pub extern "C" fn sensoreval_set_ts<'a>(
    cctx_ptr: *mut CContext<'a>,
    us: u64,
) -> std::os::raw::c_int {
    let cctx = unwrap_opt_or!(unsafe { cctx_ptr.as_mut() }, return -1);
    let renderctx = unwrap_opt_or!(&mut cctx.renderctx, return -2);

    unwrap_res_or!(renderctx.set_ts(us), return -3);

    return 0;
}

#[no_mangle]
pub extern "C" fn sensoreval_notify_stdin<'a>(cctx_ptr: *mut CContext<'a>) -> std::os::raw::c_int {
    let cctx = unwrap_opt_or!(unsafe { cctx_ptr.as_mut() }, return -1);
    let renderctx = unwrap_opt_or!(&mut cctx.renderctx, return -2);

    let sample = match cctx.readctx.read_sample(&mut std::io::stdin(), &cctx.cfg) {
        Err(e) => match &e.repr {
            ErrorRepr::EOF => return -3,
            ErrorRepr::Io(eio) => match eio.kind() {
                std::io::ErrorKind::WouldBlock => {
                    return 0;
                }
                _ => return -4,
            },
            _ => return -5,
        },
        Ok(v) => v,
    };

    renderctx.set_data(sample);

    return 1;
}

#[no_mangle]
pub extern "C" fn sensoreval_get_quat<'a>(
    cctx_ptr: *const CContext<'a>,
    quat_ptr: *mut std::os::raw::c_double,
) -> std::os::raw::c_int {
    let cctx = unwrap_opt_or!(unsafe { cctx_ptr.as_ref() }, return -1);
    let quat = unsafe { std::slice::from_raw_parts_mut(quat_ptr, 4) };
    let renderctx = unwrap_opt_or!(&cctx.renderctx, return -2);
    let data = unwrap_opt_or!(renderctx.current_data(), return -3);

    quat[0] = data.quat[3];
    quat[1] = data.quat[0];
    quat[2] = data.quat[1];
    quat[3] = data.quat[2];

    return 0;
}

#[no_mangle]
pub extern "C" fn sensoreval_get_video_info<'a>(
    cctx_ptr: *const CContext<'a>,
    filename_ptr: *mut std::os::raw::c_char,
    filename_sz_uint: std::os::raw::c_uint,
    video_startoff_ptr: *mut u64,
    video_endoff_ptr: *mut u64,
) -> std::os::raw::c_int {
    let cctx = unwrap_opt_or!(unsafe { cctx_ptr.as_ref() }, return -1);
    let filename_sz = filename_sz_uint as usize;

    match &cctx.cfg.video.filename {
        Some(v) => {
            if filename_sz <= v.len() {
                return -2;
            }
            unsafe {
                std::ptr::copy(v.as_ptr(), filename_ptr as *mut u8, v.len());
                *filename_ptr.offset(v.len() as isize) = 0;
            }
        }
        None => {
            if (filename_sz) < 1 {
                return -2;
            }
            unsafe {
                *filename_ptr = 0;
            }
        }
    }

    unsafe {
        *video_startoff_ptr = cctx.cfg.video.startoff;
        *video_endoff_ptr = cctx.cfg.video.endoff;
    }

    return 0;
}
