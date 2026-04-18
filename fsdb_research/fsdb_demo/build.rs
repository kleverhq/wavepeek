use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-env-changed=VERDI_HOME");
    println!("cargo:rerun-if-env-changed=CXX");
    println!("cargo:rerun-if-env-changed=FSDB_DEMO_EXTRA_CXXFLAGS");
    println!("cargo:rerun-if-env-changed=FSDB_DEMO_EXTRA_LDFLAGS");
    println!("cargo:rerun-if-changed=native/bridge_api.h");
    println!("cargo:rerun-if-changed=native/mock_bridge.cpp");
    println!("cargo:rerun-if-changed=native/verdi_bridge.cpp");

    let manifest_dir =
        PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set"));
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR must be set"));
    let native_dir = manifest_dir.join("native");

    let mock_bridge_path = out_dir.join("libfsdb_mock_bridge.so");
    compile_mock_bridge(&native_dir, &mock_bridge_path);
    println!(
        "cargo:rustc-env=FSDB_DEMO_MOCK_BRIDGE_PATH={}",
        mock_bridge_path.display()
    );

    match env::var_os("VERDI_HOME") {
        None => emit_verdi_bridge_env("skipped-no-verdi-home", None, None),
        Some(verdi_home) => build_verdi_bridge(&native_dir, &out_dir, &PathBuf::from(verdi_home)),
    }
}

fn build_verdi_bridge(native_dir: &Path, out_dir: &Path, verdi_home: &Path) {
    let include_dir = verdi_home.join("share/FsdbReader");
    let library_dir = include_dir.join("linux64");
    let header_path = include_dir.join("ffrAPI.h");
    let nffr_path = library_dir.join("libnffr.so");
    let nsys_path = library_dir.join("libnsys.so");

    if !header_path.exists() {
        panic!(
            "VERDI_HOME is set to '{}' but '{}' is missing",
            verdi_home.display(),
            header_path.display()
        );
    }
    if !nffr_path.exists() {
        panic!(
            "VERDI_HOME is set to '{}' but '{}' is missing",
            verdi_home.display(),
            nffr_path.display()
        );
    }
    if !nsys_path.exists() {
        panic!(
            "VERDI_HOME is set to '{}' but '{}' is missing",
            verdi_home.display(),
            nsys_path.display()
        );
    }

    let bridge_path = out_dir.join("libfsdb_verdi_bridge.so");
    let compiler = cxx_compiler();
    let mut command = Command::new(&compiler);
    command
        .arg("-shared")
        .arg("-fPIC")
        .arg("-std=c++11")
        .arg("-I")
        .arg(native_dir)
        .arg("-I")
        .arg(&include_dir)
        .arg("-o")
        .arg(&bridge_path)
        .arg(native_dir.join("verdi_bridge.cpp"))
        .arg("-L")
        .arg(&library_dir)
        .arg("-lnffr")
        .arg("-lnsys")
        .arg(format!("-Wl,-rpath,{}", library_dir.display()));

    push_extra_flags(&mut command, "FSDB_DEMO_EXTRA_CXXFLAGS");
    push_extra_flags(&mut command, "FSDB_DEMO_EXTRA_LDFLAGS");

    let output = command.output().unwrap_or_else(|error| {
        panic!(
            "failed to spawn '{}' while building the Verdi bridge: {error}",
            compiler.display()
        )
    });

    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!(
            "failed to build the Verdi bridge with '{}'.\nstdout:\n{}\nstderr:\n{}\nHint: adjust FSDB_DEMO_EXTRA_CXXFLAGS / FSDB_DEMO_EXTRA_LDFLAGS if your Verdi toolchain needs extra ABI flags.",
            compiler.display(),
            stdout.trim_end(),
            stderr.trim_end()
        );
    }

    emit_verdi_bridge_env("built", Some(&bridge_path), Some(verdi_home));
}

fn compile_mock_bridge(native_dir: &Path, bridge_path: &Path) {
    let compiler = cxx_compiler();
    let output = Command::new(&compiler)
        .arg("-shared")
        .arg("-fPIC")
        .arg("-std=c++11")
        .arg("-I")
        .arg(native_dir)
        .arg("-o")
        .arg(bridge_path)
        .arg(native_dir.join("mock_bridge.cpp"))
        .output()
        .unwrap_or_else(|error| {
            panic!(
                "failed to spawn '{}' while building the mock bridge: {error}",
                compiler.display()
            )
        });

    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!(
            "failed to build the mock bridge with '{}'.\nstdout:\n{}\nstderr:\n{}",
            compiler.display(),
            stdout.trim_end(),
            stderr.trim_end()
        );
    }
}

fn emit_verdi_bridge_env(status: &str, bridge_path: Option<&Path>, verdi_home: Option<&Path>) {
    println!("cargo:rustc-env=FSDB_DEMO_VERDI_BRIDGE_STATUS={status}");
    println!(
        "cargo:rustc-env=FSDB_DEMO_VERDI_BRIDGE_PATH={}",
        bridge_path.map_or_else(String::new, |path| path.display().to_string())
    );
    println!(
        "cargo:rustc-env=FSDB_DEMO_VERDI_HOME={}",
        verdi_home.map_or_else(String::new, |path| path.display().to_string())
    );
}

fn cxx_compiler() -> PathBuf {
    env::var_os("CXX")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("c++"))
}

fn push_extra_flags(command: &mut Command, env_name: &str) {
    if let Some(raw_flags) = env::var_os(env_name) {
        for flag in raw_flags.to_string_lossy().split_whitespace() {
            command.arg(flag);
        }
    }
}
