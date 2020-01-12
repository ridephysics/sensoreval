#![allow(clippy::not_unsafe_ptr_arg_deref)]

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
    let dataset = if islive {
        None
    } else {
        Some(unwrap_res_or!(
            datareader::read_all_samples_cfg(&cfg),
            return std::ptr::null_mut()
        ))
    };

    let cctx_ptr = Box::into_raw(Box::new(CContext {
        cfg,
        dataset,
        readctx: datareader::Context::new(),
        renderctx: None,
    }));
    let cctx = unsafe { cctx_ptr.as_mut() }.unwrap();

    cctx.renderctx = Some(render::Context::new(&cctx.cfg, cctx.dataset.as_ref()));

    cctx_ptr
}

#[no_mangle]
pub extern "C" fn sensoreval_destroy(cctx_ptr: *mut CContext) {
    unsafe { Box::from_raw(cctx_ptr) };
}

#[no_mangle]
pub extern "C" fn sensoreval_render(
    cctx_ptr: *const CContext,
    cr_ptr: *mut cairo_sys::cairo_t,
) -> std::os::raw::c_int {
    let cctx = unwrap_opt_or!(unsafe { cctx_ptr.as_ref() }, return -1);
    let cr = unsafe { cairo::Context::from_raw_borrow(cr_ptr) };
    let renderctx = unwrap_opt_or!(&cctx.renderctx, return -2);

    unwrap_res_or!(renderctx.render(&cr), return -3);

    0
}

#[no_mangle]
pub extern "C" fn sensoreval_set_ts(cctx_ptr: *mut CContext, us: u64) -> std::os::raw::c_int {
    let cctx = unwrap_opt_or!(unsafe { cctx_ptr.as_mut() }, return -1);
    let renderctx = unwrap_opt_or!(&mut cctx.renderctx, return -2);

    unwrap_res_or!(renderctx.set_ts(us), return -3);

    0
}

#[no_mangle]
pub extern "C" fn sensoreval_notify_stdin(cctx_ptr: *mut CContext) -> std::os::raw::c_int {
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

    1
}

#[no_mangle]
pub extern "C" fn sensoreval_get_quat(
    cctx_ptr: *const CContext,
    quat_ptr: *mut std::os::raw::c_double,
) -> std::os::raw::c_int {
    let _cctx = unwrap_opt_or!(unsafe { cctx_ptr.as_ref() }, return -1);
    let quat = unsafe { std::slice::from_raw_parts_mut(quat_ptr, 4) };

    quat[0] = 1.0;
    quat[1] = 0.0;
    quat[2] = 0.0;
    quat[3] = 0.0;

    0
}

#[no_mangle]
pub extern "C" fn sensoreval_get_video_info(
    cctx_ptr: *const CContext,
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
                *filename_ptr.add(v.len()) = 0;
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
        *video_endoff_ptr = match cctx.cfg.video.endoff {
            Some(v) => v,
            None => std::u64::MAX,
        }
    }

    0
}
