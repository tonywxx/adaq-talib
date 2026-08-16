//! `adaq-talib` 交互式调用示例入口 / Interactive example entry point.
//!
//! 用法 / Usage:
//! ```text
//! cargo run --example demo -- sma
//! cargo run --example demo -- ema
//! cargo run --example demo -- wma
//! cargo run --example demo -- dema
//! cargo run --example demo -- tema
//! cargo run --example demo -- midpoint
//! cargo run --example demo -- midprice
//! cargo run --example demo -- rsi
//! cargo run --example demo -- macd
//! cargo run --example demo -- cmo
//! cargo run --example demo -- trix
//! cargo run --example demo -- mom
//! cargo run --example demo -- cci
//! cargo run --example demo -- willr
//! cargo run --example demo -- bop
//! cargo run --example demo -- ultosc
//! cargo run --example demo -- adx
//! cargo run --example demo -- aroon
//! cargo run --example demo -- stoch
//! cargo run --example demo -- mfi
//! cargo run --example demo -- trange
//! cargo run --example demo -- atr
//! cargo run --example demo -- natr
//! cargo run --example demo -- ad
//! cargo run --example demo -- adosc
//! cargo run --example demo -- obv
//! cargo run --example demo -- avgprice
//! cargo run --example demo -- medprice
//! cargo run --example demo -- typprice
//! cargo run --example demo -- wclprice
//! cargo run --example demo -- stddev
//! cargo run --example demo -- var
//! cargo run --example demo -- linearreg
//! cargo run --example demo -- linearreg_angle
//! cargo run --example demo -- linearreg_intercept
//! cargo run --example demo -- linearreg_slope
//! cargo run --example demo -- tsf
//! cargo run --example demo -- beta
//! cargo run --example demo -- correl
//! cargo run --example demo -- bbands
//! cargo run --example demo -- trima
//! cargo run --example demo -- t3
//! cargo run --example demo -- ma
//! cargo run --example demo -- mavp
//! cargo run --example demo -- kama
//! cargo run --example demo -- sar
//! cargo run --example demo -- sarext
//! cargo run --example demo -- mama
//! cargo run --example demo -- ht_trendline
//! ```
//! 当前仅演示已实现的指标；后续指标按里程碑补齐后在此注册（见 ADR 0002）。
//! Currently demonstrates implemented indicators only; more are registered here as the
//! milestone plan lands additional functions (ADR 0002).

use adaq_talib::momentum::{
    adx_default, aroon_default, bop, cci_default, cmo_default, macd_default, mfi_default, mom,
    rsi_default, stoch_default, trix_default, ultosc_default, willr_default,
};
use adaq_talib::overlap::{
    bbands_default, dema, ema, kama_default, ma, mavp_default, midpoint, midprice, sar, sarext,
    sma, tema, t3, trima, wma, MaType,
};
use adaq_talib::price_transform::{avgprice, medprice, typprice, wclprice};
use adaq_talib::stat::{
    beta_default, correl_default, linear_reg_angle_default, linear_reg_default,
    linear_reg_intercept_default, linear_reg_slope_default, stddev_default, tsf_default,
    var_default,
};
use adaq_talib::volume::{ad, adosc_default, obv};
use adaq_talib::volatility::{atr_default, natr_default, trange};
use adaq_talib::cycle::{ht_trendline_default, mama_default};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let indicator = args.get(1).map(|s| s.as_str()).unwrap_or("sma");

    match indicator {
        // ---- 重叠研究 / Overlap Studies ----
        "sma" => demo_single("SMA", 3, &|p| sma(p, 3).expect("sma")),
        "ema" => demo_single("EMA", 3, &|p| ema(p, 3).expect("ema")),
        "wma" => demo_single("WMA", 3, &|p| wma(p, 3).expect("wma")),
        "dema" => demo_single("DEMA", 3, &|p| dema(p, 3).expect("dema")),
        "tema" => demo_single("TEMA", 3, &|p| tema(p, 3).expect("tema")),
        "midpoint" => demo_single("MIDPOINT", 3, &|p| midpoint(p, 3).expect("midpoint")),
        "midprice" => demo_midprice(),

        // ---- 重叠研究（第二批）/ Overlap Studies (batch 2) ----
        "bbands" => demo_bbands(),
        "trima" => demo_single("TRIMA", 30, &|p| trima(p, 30).expect("trima")),
        "t3" => demo_t3(),
        "ma" => demo_single("MA", 30, &|p| ma(p, 30, MaType::Sma).expect("ma")),
        "mavp" => demo_mavp(),
        "kama" => demo_single("KAMA", 30, &|p| kama_default(p).expect("kama")),
        "sar" => demo_ohlc("SAR", &|h, l, _c, _o| sar(h, l, 0.02, 0.2).expect("sar")),
        "sarext" => demo_ohlc("SAREXT", &|h, l, _c, _o| {
            sarext(h, l, 0.0, 0.0, 0.02, 0.02, 0.2, 0.02, 0.02, 0.2).expect("sarext")
        }),

        // ---- 动量指标 / Momentum Indicators ----
        "rsi" => demo_single("RSI", 14, &|p| rsi_default(p).expect("rsi")),
        "cmo" => demo_single("CMO", 14, &|p| cmo_default(p).expect("cmo")),
        "trix" => demo_single("TRIX", 30, &|p| trix_default(p).expect("trix")),
        "mom" => demo_single("MOM", 10, &|p| mom(p, 10).expect("mom")),
        "macd" => demo_macd(),
        "cci" => demo_ohlc("CCI", &|h, l, c, _| cci_default(h, l, c).expect("cci")),
        "willr" => demo_ohlc("WILLR", &|h, l, c, _| willr_default(h, l, c).expect("willr")),
        "bop" => demo_ohlc("BOP", &|h, l, c, o| bop(o, h, l, c).expect("bop")),
        "ultosc" => demo_ohlc("ULTOSC", &|h, l, c, _| ultosc_default(h, l, c).expect("ultosc")),
        "adx" => demo_ohlc("ADX", &|h, l, c, _| adx_default(h, l, c).expect("adx")),
        "aroon" => demo_aroon(),
        "stoch" => demo_ohlc("STOCH", &|h, l, c, _| {
            let s = stoch_default(h, l, c).expect("stoch");
            s.slow_k
        }),
        "mfi" => demo_ohlc_vol("MFI"),

        // ---- 波动率 / Volatility ----
        "trange" => demo_ohlc("TRANGE", &|h, l, c, _| trange(h, l, c).expect("trange")),
        "atr" => demo_ohlc("ATR", &|h, l, c, _| atr_default(h, l, c).expect("atr")),
        "natr" => demo_ohlc("NATR", &|h, l, c, _| natr_default(h, l, c).expect("natr")),

        // ---- 成交量 / Volume ----
        "ad" => demo_ohlc_vol_ind("AD", &|h, l, c, v| ad(h, l, c, v).expect("ad")),
        "adosc" => {
            demo_ohlc_vol_ind("ADOSC", &|h, l, c, v| adosc_default(h, l, c, v).expect("adosc"))
        }
        "obv" => demo_ohlc_vol_ind("OBV", &|_h, _l, c, v| obv(c, v).expect("obv")),

        // ---- 价格变换 / Price Transform ----
        "avgprice" => {
            demo_ohlc("AVGPRICE", &|h, l, c, o| avgprice(h, l, c, o).expect("avgprice"))
        }
        "medprice" => demo_ohlc("MEDPRICE", &|h, l, _c, _o| medprice(h, l).expect("medprice")),
        "typprice" => {
            demo_ohlc("TYPPRICE", &|h, l, c, _o| typprice(h, l, c).expect("typprice"))
        }
        "wclprice" => {
            demo_ohlc("WCLPRICE", &|h, l, c, _o| wclprice(h, l, c).expect("wclprice"))
        }

        // ---- 统计 / Statistic ----
        "stddev" => demo_single("STDDEV", 5, &|p| stddev_default(p).expect("stddev")),
        "var" => demo_single("VAR", 5, &|p| var_default(p).expect("var")),
        "linearreg" => demo_single("LINEARREG", 14, &|p| linear_reg_default(p).expect("linear_reg")),
        "linearreg_angle" => demo_single("LINEARREG_ANGLE", 14, &|p| {
            linear_reg_angle_default(p).expect("linear_reg_angle")
        }),
        "linearreg_intercept" => demo_single("LINEARREG_INTERCEPT", 14, &|p| {
            linear_reg_intercept_default(p).expect("linear_reg_intercept")
        }),
        "linearreg_slope" => demo_single("LINEARREG_SLOPE", 14, &|p| {
            linear_reg_slope_default(p).expect("linear_reg_slope")
        }),
        "tsf" => demo_single("TSF", 14, &|p| tsf_default(p).expect("tsf")),
        "beta" => demo_beta_corr(),
        "correl" => demo_beta_corr(),

        // ---- 周期类 / Cycle ----
        "mama" => demo_mama(),
        "ht_trendline" => demo_ht_trendline(),

        other => {
            eprintln!("未知指标 / unknown indicator: {other}");
            eprintln!(
                "当前支持 / supported: sma, ema, wma, dema, tema, midpoint, midprice, \
                 bbands, trima, t3, ma, mavp, kama, sar, sarext, mama, ht_trendline, \
                 rsi, cmo, trix, mom, macd, cci, willr, bop, ultosc, adx, aroon, stoch, mfi, \
                 trange, atr, natr, ad, adosc, obv, avgprice, medprice, typprice, wclprice, \
                 stddev, var, linearreg, linearreg_angle, linearreg_intercept, linearreg_slope, \
                 tsf, beta, correl"
            );
            eprintln!("用法 / usage: cargo run --example demo -- <indicator>");
            std::process::exit(2);
        }
    }
}

/// 单输入指标的演示骨架：用内置样本数据（2 的幂）以给定周期计算并打印。
/// Demo skeleton for single-input indicators: compute on built-in sample data
/// (powers of two) with the given period and print.
fn demo_single(name: &str, period: usize, f: &dyn Fn(&[f64]) -> Vec<f64>) {
    // 仅重叠研究示例用 12 点数据；动量默认周期较长，用更长的样本以保证有有效输出。
    // Overlap demos use 12 points; momentum defaults need a longer series for valid output.
    let prices: Vec<f64> = if period <= 12 {
        (0..12)
            .map(|i| 2f64.powi(i))
            .collect()
    } else {
        (0..120)
            .map(|i| 100.0 + 10.0 * (i as f64 * 0.3).sin() + i as f64 * 0.05)
            .collect()
    };
    let out = f(&prices);

    println!("{name}(prices, period = {period})");
    println!("-----------------------------------");
    let start = if period <= 12 { 0 } else { prices.len() - 12 };
    for i in start..prices.len() {
        let s = if out[i].is_nan() {
            "NaN (不稳定期 / unstable)".to_string()
        } else {
            format!("{:.4}", out[i])
        };
        println!("  [{i:2}] input = {:9.2}  ->  {s}", prices[i]);
    }
}

/// 演示 MIDPRICE：用内置 high/low 样本数据以周期 3 计算并打印。
/// Demo MIDPRICE on built-in high/low sample data with period 3.
fn demo_midprice() {
    let high: [f64; 12] = [
        2.0, 3.0, 5.0, 9.0, 17.0, 33.0, 65.0, 129.0, 257.0, 513.0, 1025.0, 2049.0,
    ];
    let low: [f64; 12] = [
        0.0, 1.0, 3.0, 7.0, 15.0, 31.0, 63.0, 127.0, 255.0, 511.0, 1023.0, 2047.0,
    ];
    let period = 3;
    let out = midprice(&high, &low, period).expect("midprice");

    println!("MIDPRICE(high, low, period = {period})");
    println!("----------------------------------------");
    for (i, v) in out.iter().enumerate() {
        let s = if v.is_nan() {
            "NaN (不稳定期 / unstable)".to_string()
        } else {
            format!("{v:.4}")
        };
        println!(
            "  [{i:2}] high = {:6.2}  low = {:6.2}  ->  {s}",
            high[i], low[i]
        );
    }
}

/// 演示 MACD（默认 12/26/9），打印 macd / signal / hist 三列。
/// Demo MACD (defaults 12/26/9), printing macd / signal / hist.
fn demo_macd() {
    let close: Vec<f64> = (0..60).map(|i| 100.0 + 10.0 * (i as f64 * 0.3).sin()).collect();
    let m = macd_default(&close).expect("macd");
    println!("MACD(close, 12/26/9)");
    println!("--------------------------------------------------");
    for i in 40..close.len() {
        let fmt = |v: f64| {
            if v.is_nan() {
                "    NaN".to_string()
            } else {
                format!("{v:8.4}")
            }
        };
        println!(
            "  [{i:2}] close = {:7.2}  macd = {}  signal = {}  hist = {}",
            close[i],
            fmt(m.macd[i]),
            fmt(m.signal[i]),
            fmt(m.hist[i])
        );
    }
}

/// 样本 OHLC 数据（内置确定性序列）。/ Built-in deterministic OHLC sample data.
fn sample_ohlc() -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
    let n = 40;
    let close: Vec<f64> = (0..n).map(|i| 100.0 + 10.0 * (i as f64 * 0.3).sin() + i as f64 * 0.05).collect();
    let open: Vec<f64> = std::iter::once(close[0]).chain(close.iter().take(n - 1).copied()).collect();
    let high: Vec<f64> = (0..n)
        .map(|i| open[i].max(close[i]) + 1.0 + 0.5 * (i as f64).sin())
        .collect();
    let low: Vec<f64> = (0..n)
        .map(|i| open[i].min(close[i]) - 1.0 - 0.5 * (i as f64 + 1.0).sin())
        .collect();
    (open, high, low, close)
}

/// 多输入（OHLC）指标的演示骨架。/ Demo skeleton for OHLC-input indicators.
fn demo_ohlc(name: &str, f: &dyn Fn(&[f64], &[f64], &[f64], &[f64]) -> Vec<f64>) {
    let (open, high, low, close) = sample_ohlc();
    let out = f(&high, &low, &close, &open);
    println!("{name}(high, low, close, open)");
    println!("--------------------------------------------------");
    for i in 0..close.len() {
        let s = if out[i].is_nan() {
            "    NaN".to_string()
        } else {
            format!("{:8.4}", out[i])
        };
        println!(
            "  [{i:2}] close = {:7.2}  high = {:7.2}  low = {:7.2}  ->  {s}",
            close[i], high[i], low[i]
        );
    }
}

/// 演示 AROON（输出 up/down 两列）。/ Demo AROON (up/down columns).
fn demo_aroon() {
    let (_, high, low, _) = sample_ohlc();
    let a = aroon_default(&high, &low).expect("aroon");
    println!("AROON(high, low, period = 14)");
    println!("--------------------------------------------------");
    for i in 14..high.len() {
        let fmt = |v: f64| {
            if v.is_nan() {
                "    NaN".to_string()
            } else {
                format!("{v:8.4}")
            }
        };
        println!(
            "  [{i:2}]  up = {}  down = {}",
            fmt(a.up[i]),
            fmt(a.down[i])
        );
    }
}

/// 演示 MFI（需要成交量）。/ Demo MFI (requires volume).
fn demo_ohlc_vol(name: &str) {
    let (_, high, low, close) = sample_ohlc();
    let volume: Vec<f64> = (0..close.len())
        .map(|i| 1000.0 + 100.0 * (i as f64 * 0.7).sin())
        .collect();
    let out = mfi_default(&high, &low, &close, &volume).expect("mfi");
    println!("{name}(high, low, close, volume, period = 14)");
    println!("--------------------------------------------------");
    for i in 14..close.len() {
        let s = if out[i].is_nan() {
            "    NaN".to_string()
        } else {
            format!("{:8.4}", out[i])
        };
        println!("  [{i:2}] close = {:7.2}  ->  {s}", close[i]);
    }
}

/// 多输入（OHLC + 成交量）指标的演示骨架。/ Demo skeleton for OHLC+volume indicators.
fn demo_ohlc_vol_ind(name: &str, f: &dyn Fn(&[f64], &[f64], &[f64], &[f64]) -> Vec<f64>) {
    let (_, high, low, close) = sample_ohlc();
    let volume: Vec<f64> = (0..close.len())
        .map(|i| 1000.0 + 100.0 * (i as f64 * 0.7).sin())
        .collect();
    let out = f(&high, &low, &close, &volume);
    println!("{name}(high, low, close, volume)");
    println!("--------------------------------------------------");
    for i in 0..close.len() {
        let s = if out[i].is_nan() {
            "    NaN".to_string()
        } else {
            format!("{:8.4}", out[i])
        };
        println!("  [{i:2}] close = {:7.2}  ->  {s}", close[i]);
    }
}

/// 演示 BETA / CORREL（使用 close 与 high 两序列，默认周期 5）。
/// Demo BETA / CORREL (close vs high, default period 5).
fn demo_beta_corr() {
    let (_, high, _, close) = sample_ohlc();
    let b = beta_default(&close, &high).expect("beta");
    let c = correl_default(&close, &high).expect("correl");
    println!("BETA / CORREL(close, high, period = 5)");
    println!("--------------------------------------------------");
    for i in 0..close.len() {
        let fmt = |v: f64| {
            if v.is_nan() {
                "    NaN".to_string()
            } else {
                format!("{v:8.4}")
            }
        };
        println!("  [{i:2}]  beta = {}  correl = {}", fmt(b[i]), fmt(c[i]));
    }
}

/// 演示 BBANDS（默认 20/2.0/2.0/SMA），打印上/中/下三轨。
/// Demo BBANDS (defaults 20/2.0/2.0/SMA), printing upper/middle/lower bands.
fn demo_bbands() {
    let (_, _, _, close) = sample_ohlc();
    let b = bbands_default(&close).expect("bbands");
    println!("BBANDS(close, 20, 2.0/2.0, SMA)");
    println!("--------------------------------------------------");
    let fmt = |v: f64| if v.is_nan() { "    NaN".to_string() } else { format!("{v:8.4}") };
    for i in 0..close.len() {
        println!(
            "  [{i:2}] close = {:7.2}  upper = {}  middle = {}  lower = {}",
            close[i],
            fmt(b.upper[i]),
            fmt(b.middle[i]),
            fmt(b.lower[i])
        );
    }
}

/// 演示 T3（默认周期 5、v 因子 0.7），用较长样本以保证有有效输出（lookback = 25）。
/// Demo T3 (defaults period 5, v-factor 0.7); longer series so output is valid
/// (lookback = 6*(period-1)+1 = 25).
fn demo_t3() {
    let prices: Vec<f64> = (0..80)
        .map(|i| 100.0 + 10.0 * (i as f64 * 0.3).sin() + i as f64 * 0.05)
        .collect();
    let out = t3(&prices, 5, 0.7).expect("t3");
    println!("T3(prices, period = 5, vfactor = 0.7)");
    println!("--------------------------------------------------");
    let start = prices.len() - 20;
    for i in start..prices.len() {
        let s = if out[i].is_nan() {
            "NaN (不稳定期 / unstable)".to_string()
        } else {
            format!("{:.4}", out[i])
        };
        println!("  [{i:2}] input = {:9.2}  ->  {s}", prices[i]);
    }
}

/// 演示 MAVP（默认 min 2 / max 30、SMA），用变周期数组覆盖多种周期。
/// Demo MAVP (defaults min 2 / max 30, SMA) with a varying-period array.
fn demo_mavp() {
    let (_, _, _, close) = sample_ohlc();
    let periods: Vec<f64> = (0..close.len()).map(|i| 2.0 + (i % 29) as f64).collect();
    let out = mavp_default(&close, &periods).expect("mavp");
    println!("MAVP(close, periods, min = 2, max = 30, SMA)");
    println!("--------------------------------------------------");
    for i in 29..close.len() {
        let s = if out[i].is_nan() {
            "NaN (不稳定期 / unstable)".to_string()
        } else {
            format!("{:.4}", out[i])
        };
        println!(
            "  [{i:2}] period = {:2.0}  close = {:7.2}  ->  {s}",
            periods[i], close[i]
        );
    }
}

/// 演示 MAMA / FAMA（默认 0.5 / 0.05），用较长样本（lookback = 32）。
/// Demo MAMA / FAMA (defaults 0.5 / 0.05); longer series (lookback = 32).
fn demo_mama() {
    let close: Vec<f64> = (0..80)
        .map(|i| 100.0 + 10.0 * (i as f64 * 0.3).sin() + i as f64 * 0.05)
        .collect();
    let m = mama_default(&close).expect("mama");
    println!("MAMA(close, fast = 0.5, slow = 0.05)");
    println!("--------------------------------------------------");
    let fmt = |v: f64| if v.is_nan() { "    NaN".to_string() } else { format!("{v:8.4}") };
    for i in 32..close.len() {
        println!(
            "  [{i:2}] close = {:7.2}  mama = {}  fama = {}",
            close[i],
            fmt(m.mama[i]),
            fmt(m.fama[i])
        );
    }
}

/// 演示 HT_TRENDLINE（默认无参数），用较长样本（lookback = 63）。
/// Demo HT_TRENDLINE (no optional inputs); longer series (lookback = 63).
fn demo_ht_trendline() {
    let close: Vec<f64> = (0..80)
        .map(|i| 100.0 + 10.0 * (i as f64 * 0.3).sin() + i as f64 * 0.05)
        .collect();
    let out = ht_trendline_default(&close).expect("ht_trendline");
    println!("HT_TRENDLINE(close)");
    println!("--------------------------------------------------");
    for i in 63..close.len() {
        let s = if out[i].is_nan() {
            "    NaN".to_string()
        } else {
            format!("{:.4}", out[i])
        };
        println!("  [{i:2}] close = {:7.2}  ->  {s}", close[i]);
    }
}
