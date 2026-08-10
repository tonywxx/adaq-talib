// 构建脚本：仅在 `bench-c` feature 下链接 TA-Lib C 库（原生对照，见 ADR 0004）。
// Build script: link the TA-Lib C library only under the `bench-c` feature (native comparison).
//
// 未启用该 feature 时不做任何链接，普通构建/测试完全零 C 依赖。
// Without the feature, nothing is linked; normal builds/tests stay C-free.

fn main() {
    if std::env::var("CARGO_FEATURE_BENCH_C").is_ok() {
        println!("cargo:rustc-link-lib=ta_lib");
        // 若 C 库不在默认搜索路径，可在此添加，例如：
        // println!("cargo:rustc-link-search=native=/opt/homebrew/lib");
    }
}
