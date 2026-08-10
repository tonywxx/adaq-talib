// 构建脚本：仅在 `bench-c` feature 下链接 TA-Lib C 库（原生对照，见 ADR 0004）。
// Build script: link the TA-Lib C library only under the `bench-c` feature (native comparison).
//
// - 库名在 macOS(Homebrew) 为 `ta-lib`（文件 `libta-lib.dylib`）；其它平台多为 `ta_lib`
//   （文件 `libta_lib.*`）。本脚本自动探测存在的文件名并发出正确的 `-l` 名。
//   (On macOS/Homebrew the lib is `ta-lib`; elsewhere usually `ta_lib`. We detect which.)
// - 路径可用环境变量 `TA_LIB_LIB_DIR` 显式覆盖。
// - 未启用该 feature 时不做任何链接，普通构建/测试完全零 C 依赖（Zero-FFI 不变）。
//   (Without the feature this script does nothing; normal builds/tests stay C-free.)

fn main() {
    if std::env::var("CARGO_FEATURE_BENCH_C").is_ok() {
        // 候选搜索路径；TA_LIB_LIB_DIR 可显式覆盖。
        // Candidate search paths; TA_LIB_LIB_DIR overrides.
        let mut candidates: Vec<String> = Vec::new();
        if let Ok(dir) = std::env::var("TA_LIB_LIB_DIR") {
            candidates.push(dir);
        }
        candidates.push("/opt/homebrew/opt/ta-lib/lib".into());
        candidates.push("/opt/homebrew/Cellar/ta-lib/0.7.1/lib".into());
        candidates.push("/opt/homebrew/lib".into());
        candidates.push("/usr/local/lib".into());
        candidates.push("/usr/lib".into());

        let lib_names: &[&str] = if cfg!(target_os = "macos") {
            &["libta-lib.dylib", "libta_lib.dylib"]
        } else if cfg!(target_os = "windows") {
            &["ta_lib.dll", "ta-lib.dll"]
        } else {
            &["libta-lib.so", "libta_lib.so"]
        };

        let mut found: Option<(String, &'static str)> = None;
        for dir in candidates.iter().filter(|d| !d.is_empty()) {
            for name in lib_names.iter() {
                if std::path::Path::new(dir).join(name).exists() {
                    let link_name: &str = if name.contains("ta-lib") {
                        "ta-lib"
                    } else {
                        "ta_lib"
                    };
                    found = Some((dir.clone(), link_name));
                    break;
                }
            }
            if found.is_some() {
                break;
            }
        }

        match found {
            Some((dir, link_name)) => {
                println!("cargo:rustc-link-search=native={dir}");
                println!("cargo:rustc-link-lib={link_name}");
            }
            None => {
                // 兜底：仍加一个常见路径并给出明确警告，便于定位。
                println!("cargo:rustc-link-search=native=/opt/homebrew/lib");
                println!("cargo:rustc-link-lib=ta-lib");
                println!(
                    "cargo:warning=bench-c enabled but TA-Lib C lib not found in candidate \
                     paths; set TA_LIB_LIB_DIR to its directory"
                );
            }
        }
    }
}
