//! PoC: validate native TA-Lib C abstract-API FFI + adaq-talib `_with_output` parity.
//! 5 functions across input/output shapes (sma=R1, bbands=struct, macd=struct, cdl_doji=price, add=2-real).
//! Run: cargo bench --bench poc_bench --features bench-c

// 基准代码：部分常量/辅助（`N`/`ITERS`/`make_inputs`/`Instant`）仅在 `bench-c` feature 下被
// 引用；默认 feature 构建时即成为死代码，统一允许以免 `-D warnings` 严格构建失败。
// Bench code: some constants/helpers (`N`/`ITERS`/`make_inputs`/`Instant`) are only referenced
// under the `bench-c` feature; under the default build they are dead code, allowed here so a
// `-D warnings` strict build still compiles.
#![allow(dead_code, unused_imports)]

#[cfg(feature = "bench-c")]
mod talib_ffi {
    use std::ffi::c_void;
    use std::os::raw::{c_char, c_double, c_int, c_uint};

    pub type TA_Integer = c_int;
    pub type TA_Real = c_double;
    pub type TA_RetCode = c_int;

    #[repr(C)]
    pub struct TA_ParamHolder {
        _priv: *mut c_void,
    }

    #[repr(C)]
    pub struct TA_FuncInfo {
        pub name: *const c_char,
        pub group: *const c_char,
        pub hint: *const c_char,
        pub camel_case_name: *const c_char,
        pub flags: c_int,
        pub nb_input: c_uint,
        pub nb_opt_input: c_uint,
        pub nb_output: c_uint,
        pub handle: *const c_uint,
    }

    #[repr(C)]
    pub struct TA_InputParameterInfo {
        pub type_: c_int,
        pub param_name: *const c_char,
        pub flags: c_int,
    }

    #[repr(C)]
    pub struct TA_OptInputParameterInfo {
        pub type_: c_int,
        pub param_name: *const c_char,
        pub flags: c_int,
        pub display_name: *const c_char,
        pub data_set: *const c_void,
        pub default_value: c_double,
        pub hint: *const c_char,
        pub help_file: *const c_char,
    }

    // Input type enum (from ta_abstract.h): Price=0, Real=1, Integer=2.
    pub const TA_INPUT_PRICE: c_int = 0;
    pub const TA_INPUT_REAL: c_int = 1;
    // Opt input type enum: RealRange=0, RealList=1, IntegerRange=2, IntegerList=3.
    pub const TA_OPT_REAL: c_int = 0;
    pub const TA_OPT_REAL_LIST: c_int = 1;

    pub type TA_CallForEachFunc = extern "C" fn(*const TA_FuncInfo, *mut c_void);

    #[allow(non_camel_case_types, dead_code)]
    unsafe extern "C" {
        pub fn TA_Initialize() -> TA_RetCode;
        pub fn TA_Shutdown() -> TA_RetCode;
        pub fn TA_GetFuncHandle(name: *const c_char, handle: *mut *const c_uint) -> TA_RetCode;
        pub fn TA_GetFuncInfo(handle: *const c_uint, info: *mut *const TA_FuncInfo) -> TA_RetCode;
        pub fn TA_ParamHolderAlloc(
            handle: *const c_uint,
            params: *mut *mut TA_ParamHolder,
        ) -> TA_RetCode;
        pub fn TA_ParamHolderFree(params: *mut TA_ParamHolder) -> TA_RetCode;
        pub fn TA_SetInputParamRealPtr(
            params: *mut TA_ParamHolder,
            idx: c_uint,
            value: *const TA_Real,
        ) -> TA_RetCode;
        pub fn TA_SetInputParamPricePtr(
            params: *mut TA_ParamHolder,
            idx: c_uint,
            open: *const TA_Real,
            high: *const TA_Real,
            low: *const TA_Real,
            close: *const TA_Real,
            volume: *const TA_Real,
            open_interest: *const TA_Real,
        ) -> TA_RetCode;
        pub fn TA_SetOptInputParamInteger(
            params: *mut TA_ParamHolder,
            idx: c_uint,
            value: TA_Integer,
        ) -> TA_RetCode;
        pub fn TA_SetOptInputParamReal(
            params: *mut TA_ParamHolder,
            idx: c_uint,
            value: TA_Real,
        ) -> TA_RetCode;
        pub fn TA_SetOutputParamRealPtr(
            params: *mut TA_ParamHolder,
            idx: c_uint,
            out: *mut TA_Real,
        ) -> TA_RetCode;
        pub fn TA_CallFunc(
            params: *const TA_ParamHolder,
            start_idx: TA_Integer,
            end_idx: TA_Integer,
            out_beg_idx: *mut TA_Integer,
            out_nb_element: *mut TA_Integer,
        ) -> TA_RetCode;
        pub fn TA_GetInputParameterInfo(
            handle: *const c_uint,
            idx: c_uint,
            info: *mut *const TA_InputParameterInfo,
        ) -> TA_RetCode;
        pub fn TA_GetOptInputParameterInfo(
            handle: *const c_uint,
            idx: c_uint,
            info: *mut *const TA_OptInputParameterInfo,
        ) -> TA_RetCode;
        pub fn TA_ForEachFunc(func: TA_CallForEachFunc, opaque: *mut c_void) -> TA_RetCode;
        // Direct FFI for candle functions (abstract API has a quirk for CDL*).
        pub fn TA_CDLDOJI(
            start_idx: TA_Integer,
            end_idx: TA_Integer,
            in_open: *const TA_Real,
            in_high: *const TA_Real,
            in_low: *const TA_Real,
            in_close: *const TA_Real,
            out_beg_idx: *mut TA_Integer,
            out_nb_element: *mut TA_Integer,
            out_real: *mut TA_Real,
        ) -> TA_RetCode;
    }

    pub const TA_SUCCESS: TA_RetCode = 0;

    extern "C" fn foreach_cb(info: *const TA_FuncInfo, opaque: *mut c_void) {
        unsafe {
            if info.is_null() {
                return;
            }
            let name = std::ffi::CStr::from_ptr((*info).name).to_string_lossy().into_owned();
            let v = &mut *(opaque as *mut Vec<String>);
            v.push(name);
        }
    }

    /// Enumerate all TA-Lib function names via TA_ForEachFunc.
    pub fn list_names() -> Vec<String> {
        unsafe {
            let mut v: Vec<String> = Vec::new();
            TA_ForEachFunc(foreach_cb, &mut v as *mut Vec<String> as *mut c_void);
            v
        }
    }

    /// Drive a TA-Lib C function (by uppercase name) on caller-supplied buffers, timing it.
    macro_rules! chk {
        ($step:expr, $rc:expr) => {
            if $rc != TA_SUCCESS {
                eprintln!("c_run: step {} rc={}", $step, $rc);
                return None;
            }
        };
    }

    pub fn c_run(
        name: &str,
        n: usize,
        real0: &[f64],
        real1: &[f64],
        open: &[f64],
        high: &[f64],
        low: &[f64],
        close: &[f64],
        vol: &[f64],
        iters: usize,
    ) -> Option<(f64, f64)> {
        unsafe {
            let cname = std::ffi::CString::new(name).unwrap();
            let mut handle: *const c_uint = std::ptr::null();
            chk!("GetFuncHandle", TA_GetFuncHandle(cname.as_ptr(), &mut handle));
            let mut info: *const TA_FuncInfo = std::ptr::null();
            chk!("GetFuncInfo", TA_GetFuncInfo(handle, &mut info));
            let nb_input = (*info).nb_input as usize;
            let nb_opt = (*info).nb_opt_input as usize;
            let nb_out = (*info).nb_output as usize;
            eprintln!(
                "c_run {} nb_in={} nb_opt={} nb_out={}",
                name, nb_input, nb_opt, nb_out
            );

            let mut ph: *mut TA_ParamHolder = std::ptr::null_mut();
            chk!("ParamHolderAlloc", TA_ParamHolderAlloc(handle, &mut ph));

            let mut real_idx = 0usize;
            for i in 0..nb_input {
                let mut iinfo: *const TA_InputParameterInfo = std::ptr::null();
                chk!(
                    "GetInputParameterInfo",
                    TA_GetInputParameterInfo(handle, i as c_uint, &mut iinfo)
                );
                eprintln!("  input[{}] type={}", i, (*iinfo).type_);
                if (*iinfo).type_ == TA_INPUT_PRICE {
                    chk!(
                        "SetInputParamPricePtr",
                        TA_SetInputParamPricePtr(
                            ph,
                            i as c_uint,
                            open.as_ptr(),
                            high.as_ptr(),
                            low.as_ptr(),
                            close.as_ptr(),
                            vol.as_ptr(),
                            std::ptr::null(),
                        )
                    );
                } else {
                    let arr = if real_idx == 0 { real0 } else { real1 };
                    real_idx += 1;
                    chk!(
                        "SetInputParamRealPtr",
                        TA_SetInputParamRealPtr(ph, i as c_uint, arr.as_ptr())
                    );
                }
            }

            for o in 0..nb_opt {
                let mut oinfo: *const TA_OptInputParameterInfo = std::ptr::null();
                chk!(
                    "GetOptInputParameterInfo",
                    TA_GetOptInputParameterInfo(handle, o as c_uint, &mut oinfo)
                );
                let dv = (*oinfo).default_value;
                if (*oinfo).type_ == TA_OPT_REAL || (*oinfo).type_ == TA_OPT_REAL_LIST {
                    chk!(
                        "SetOptInputParamReal",
                        TA_SetOptInputParamReal(ph, o as c_uint, dv)
                    );
                } else {
                    chk!(
                        "SetOptInputParamInteger",
                        TA_SetOptInputParamInteger(ph, o as c_uint, dv as TA_Integer)
                    );
                }
            }

            let mut outs: Vec<Vec<f64>> = Vec::with_capacity(nb_out);
            for o in 0..nb_out {
                let mut buf = vec![0.0f64; n];
                chk!(
                    "SetOutputParamRealPtr",
                    TA_SetOutputParamRealPtr(ph, o as c_uint, buf.as_mut_ptr())
                );
                outs.push(buf);
            }

            let start = std::time::Instant::now();
            let mut checksum = 0.0f64;
            let mut beg: TA_Integer = 0;
            let mut nb: TA_Integer = 0;
            for _ in 0..iters {
                chk!(
                    "CallFunc",
                    TA_CallFunc(ph, 0, n as TA_Integer - 1, &mut beg, &mut nb)
                );
                for o in 0..nb_out {
                    checksum += outs[o][(nb - 1) as usize];
                }
            }
            let elapsed = start.elapsed();
            TA_ParamHolderFree(ph);
            let ns_per_elem = elapsed.as_nanos() as f64 / (iters as f64 * n as f64);
            Some((ns_per_elem, checksum))
        }
    }
}

use std::time::Instant;

const N: usize = 50_000;
const ITERS: usize = 20;

fn make_inputs() -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
    let mut x = 12345.0f64;
    let mut real0 = Vec::with_capacity(N);
    let mut real1 = Vec::with_capacity(N);
    let mut close = Vec::with_capacity(N);
    let mut open = Vec::with_capacity(N);
    let mut high = Vec::with_capacity(N);
    let mut low = Vec::with_capacity(N);
    let mut vol = Vec::with_capacity(N);
    for _ in 0..N {
        x = (x * 1103515245.0 + 12345.0) % 1e9;
        let base = 50.0 + (x / 1e9) * 10.0;
        let c = base;
        close.push(c);
        real0.push(c);
        let h = c + 0.5;
        let l = c - 0.5;
        high.push(h);
        low.push(l);
        open.push(c + 0.1);
        vol.push(1.0e6);
        let y = (x * 9301.0 + 49297.0) % 1e9;
        real1.push(40.0 + (y / 1e9) * 10.0);
    }
    (real0, real1, open, high, low, close, vol)
}

#[cfg(feature = "bench-c")]
fn run_adaq(
    real0: &[f64],
    real1: &[f64],
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
    vol: &[f64],
) -> Vec<(String, f64, f64)> {
    use adaq_talib::math_ops::add_with_output;
    use adaq_talib::momentum::macd_with_output;
    use adaq_talib::overlap::{bbands_with_output, sma_with_output, Bbands, MaType};
    use adaq_talib::pattern::cdl_doji_with_output;

    let mut rows = Vec::new();

    // sma (period 30)
    {
        let mut out = vec![0.0f64; N];
        let start = Instant::now();
        let mut ck = 0.0;
        for _ in 0..ITERS {
            sma_with_output(real0, 30, &mut out).unwrap();
            ck += out[N - 1];
        }
        let ns = start.elapsed().as_nanos() as f64 / (ITERS as f64 * N as f64);
        rows.push(("sma".into(), ns, ck));
    }
    // bbands (period 5, dev 2.0/2.0, Sma)
    {
        let mut bb = Bbands {
            upper: vec![0.0; N],
            middle: vec![0.0; N],
            lower: vec![0.0; N],
        };
        let start = Instant::now();
        let mut ck = 0.0;
        for _ in 0..ITERS {
            bbands_with_output(close, 5, 2.0, 2.0, MaType::Sma, &mut bb).unwrap();
            ck += bb.upper[N - 1] + bb.middle[N - 1] + bb.lower[N - 1];
        }
        let ns = start.elapsed().as_nanos() as f64 / (ITERS as f64 * N as f64);
        rows.push(("bbands".into(), ns, ck));
    }
    // macd (12/26/9)
    {
        use adaq_talib::momentum::Macd;
        let mut m = Macd {
            macd: vec![0.0; N],
            signal: vec![0.0; N],
            hist: vec![0.0; N],
        };
        let start = Instant::now();
        let mut ck = 0.0;
        for _ in 0..ITERS {
            macd_with_output(real0, 12, 26, 9, &mut m).unwrap();
            ck += m.macd[N - 1] + m.signal[N - 1] + m.hist[N - 1];
        }
        let ns = start.elapsed().as_nanos() as f64 / (ITERS as f64 * N as f64);
        rows.push(("macd".into(), ns, ck));
    }
    // cdl_doji
    {
        let mut out = vec![0.0f64; N];
        let start = Instant::now();
        let mut ck = 0.0;
        for _ in 0..ITERS {
            cdl_doji_with_output(open, high, low, close, &mut out).unwrap();
            ck += out[N - 1];
        }
        let ns = start.elapsed().as_nanos() as f64 / (ITERS as f64 * N as f64);
        rows.push(("cdl_doji".into(), ns, ck));
    }
    // add
    {
        let mut out = vec![0.0f64; N];
        let start = Instant::now();
        let mut ck = 0.0;
        for _ in 0..ITERS {
            add_with_output(real0, real1, &mut out).unwrap();
            ck += out[N - 1];
        }
        let ns = start.elapsed().as_nanos() as f64 / (ITERS as f64 * N as f64);
        rows.push(("add".into(), ns, ck));
    }
    let _ = (high, low, vol);
    rows
}

#[cfg(feature = "bench-c")]
fn main() {
    unsafe { talib_ffi::TA_Initialize(); }
    // Diagnose exact TA-Lib names.
    let names = talib_ffi::list_names();
    println!("TA-Lib total functions enumerated: {}", names.len());
    for nm in names.iter().filter(|n| n.contains("DOJI") || n.contains("CDL")) {
        println!("  TA name: {}", nm);
    }
    let (real0, real1, open, high, low, close, vol) = make_inputs();
    // Diagnose: do NON-candle Price-input functions also fail at output set?
    for t in ["ATR", "AD", "OBV"] {
        match talib_ffi::c_run(t, N, &real0, &real1, &open, &high, &low, &close, &vol, 1) {
            Some((ns, _)) => println!("C_diag {} ok ns={:.3}", t, ns),
            None => println!("C_diag {} FAILED", t),
        }
    }
    // Direct FFI candle test
    {
        let mut out = vec![0.0f64; N];
        let mut beg = 0i32;
        let mut nb = 0i32;
        let rc = unsafe {
            talib_ffi::TA_CDLDOJI(
                0,
                N as i32 - 1,
                open.as_ptr(),
                high.as_ptr(),
                low.as_ptr(),
                close.as_ptr(),
                &mut beg,
                &mut nb,
                out.as_mut_ptr(),
            )
        };
        println!("C_diag CDLDOJI direct rc={} nb={}", rc, nb);
    }
    let adaq = run_adaq(&real0, &real1, &open, &high, &low, &close, &vol);
    println!(
        "{:<12} {:>14} {:>14} {:>12} {:>12} {:>10}",
        "fn", "adaq_ns/elem", "C_ns/elem", "adaq_ck", "C_ck", "diff"
    );
    for (name, ans, ack) in &adaq {
        let ta_name = name.to_uppercase().replace('_', "");
        if let Some((cns, cck)) = talib_ffi::c_run(
            &ta_name, N, &real0, &real1, &open, &high, &low, &close, &vol, ITERS,
        ) {
            let diff = (ack - cck).abs();
            println!(
                "{:<12} {:>14.3} {:>14.3} {:>12.4} {:>12.4} {:>10.2e}",
                name, ans, cns, ack, cck, diff
            );
        } else {
            println!(
                "{:<12} {:>14.3} {:>14} {:>12.4} {:>12} {:>10}",
                name, ans, "C_MISS", ack, 0.0, 0.0
            );
        }
    }
    unsafe { talib_ffi::TA_Shutdown(); }
}

#[cfg(not(feature = "bench-c"))]
fn main() {
    println!("Enable with --features bench-c");
}
