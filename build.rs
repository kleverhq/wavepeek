use std::env;
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rerun-if-env-changed=VERDI_HOME");
    println!("cargo:rerun-if-env-changed=WAVEPEEK_FSDB_READER_LIBDIR");
    println!("cargo:rerun-if-env-changed=WAVEPEEK_FSDB_ABI");
    println!("cargo:rerun-if-changed=native/fsdb/wavepeek_fsdb_shim.cpp");
    println!("cargo:rerun-if-changed=native/fsdb/wavepeek_fsdb_shim.h");

    if env::var_os("CARGO_FEATURE_FSDB").is_none() {
        return;
    }

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    if target_os != "linux" || target_arch != "x86_64" {
        panic!("FSDB support is Linux x86_64 only in this build spike");
    }

    let sdk = resolve_fsdb_sdk();
    compile_fsdb_shim(&sdk);
    emit_fsdb_link_settings(&sdk);
}

#[derive(Debug)]
struct FsdbSdk {
    reader_root: PathBuf,
    libdir: PathBuf,
}

fn resolve_fsdb_sdk() -> FsdbSdk {
    let verdi_home = env::var_os("VERDI_HOME")
        .map(PathBuf::from)
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| {
            panic!(
                "FSDB support requires VERDI_HOME; set VERDI_HOME to a Synopsys Verdi installation containing share/FsdbReader"
            )
        });

    let reader_root = verdi_home.join("share").join("FsdbReader");
    require_file(
        &reader_root.join("ffrAPI.h"),
        "set VERDI_HOME to a Verdi installation containing share/FsdbReader/ffrAPI.h",
    );
    require_file(
        &reader_root.join("ffrKit.h"),
        "set VERDI_HOME to a Verdi installation containing share/FsdbReader/ffrKit.h",
    );
    require_file(
        &reader_root.join("fsdbShr.h"),
        "set VERDI_HOME to a Verdi installation containing share/FsdbReader/fsdbShr.h",
    );

    let libdir = match env::var_os("WAVEPEEK_FSDB_READER_LIBDIR") {
        Some(value) if !value.is_empty() => PathBuf::from(value),
        _ => {
            let abi = env::var_os("WAVEPEEK_FSDB_ABI")
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| "linux64".into());
            reader_root.join(abi)
        }
    };

    require_file(
        &libdir.join("libnffr.so"),
        "set WAVEPEEK_FSDB_READER_LIBDIR to the FSDB Reader library directory or try WAVEPEEK_FSDB_ABI=linux64_gcc950",
    );
    require_file(
        &libdir.join("libnsys.so"),
        "set WAVEPEEK_FSDB_READER_LIBDIR to the FSDB Reader library directory or try WAVEPEEK_FSDB_ABI=linux64_gcc950",
    );

    FsdbSdk {
        reader_root,
        libdir,
    }
}

fn require_file(path: &Path, hint: &str) {
    if !path.is_file() {
        panic!(
            "FSDB Reader SDK check failed: missing {}; {}",
            path.display(),
            hint
        );
    }
}

fn compile_fsdb_shim(sdk: &FsdbSdk) {
    cc::Build::new()
        .cpp(true)
        .flag_if_supported("-std=c++17")
        .flag_if_supported("-fvisibility=hidden")
        .flag_if_supported("-Wno-unused-parameter")
        .include(&sdk.reader_root)
        .include("native/fsdb")
        .file("native/fsdb/wavepeek_fsdb_shim.cpp")
        .compile("wavepeek_fsdb_shim");
}

fn emit_fsdb_link_settings(sdk: &FsdbSdk) {
    let nffr = sdk.libdir.join("libnffr.so");
    let nsys = sdk.libdir.join("libnsys.so");

    println!("cargo:rustc-link-search=native={}", sdk.libdir.display());
    println!("cargo:rustc-link-arg=-Wl,--no-as-needed");
    println!("cargo:rustc-link-arg={}", nffr.display());
    println!("cargo:rustc-link-arg={}", nsys.display());
    println!("cargo:rustc-link-arg=-Wl,--as-needed");
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", sdk.libdir.display());
}
