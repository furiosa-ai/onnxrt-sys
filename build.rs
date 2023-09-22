#![allow(dead_code)]

use std::{
    borrow::Cow,
    env, fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    str::FromStr,
};

/// ONNX Runtime version
///
/// WARNING: If version is changed, bindings for all platforms will have to be re-generated.
///          To do so, run this:
///              cargo build --features generate-bindings
const ORT_VERSION: &str = "1.16.0";

/// Base Url from which to download pre-built releases/
const ORT_RELEASE_BASE_URL: &str = "https://github.com/microsoft/onnxruntime/releases/download";

/// Environment variable selecting which strategy to use for finding the library
/// Possibilities:
/// * "download": Download a pre-built library from upstream. This is the default if `ORT_STRATEGY` is not set.
/// * "system": Use installed library. Use `ORT_LIB_DIR` to point to proper location.
/// * "compile": Download source and compile (TODO).
const ORT_ENV_STRATEGY: &str = "ORT_STRATEGY";

/// Name of environment variable that, if present, contains the location of the ONNX Runtime header
/// files. Only used if `ORT_STRATEGY=system`.
const ORT_ENV_SYSTEM_INCLUDE_DIR: &str = "ORT_INCLUDE_DIR";
/// Name of environment variable that, if present, contains the location of the ONNX Runtime
/// library.  Only used if `ORT_STRATEGY=system`.
const ORT_ENV_SYSTEM_LIB_DIR: &str = "ORT_LIB_DIR";
/// Name of environment variable that, if present, controls wether to use CUDA or not.
const ORT_ENV_GPU: &str = "ORT_USE_CUDA";

/// Subdirectory (of the 'target' directory) into which to extract the prebuilt library.
const ORT_PREBUILT_EXTRACT_DIR: &str = "onnxruntime";

fn main() {
    let (include_dir, lib_dir) = prepare_libort_dir();

    println!("Include directory: {include_dir:?}");
    println!("Lib directory: {lib_dir:?}");

    // Tell cargo to tell rustc to link onnxruntime shared library.
    println!("cargo:rustc-link-lib=onnxruntime");
    println!("cargo:rustc-link-search=native={}", lib_dir.display());

    println!("cargo:rerun-if-env-changed={ORT_ENV_STRATEGY}");
    println!("cargo:rerun-if-env-changed={ORT_ENV_GPU}");
    println!("cargo:rerun-if-env-changed={ORT_ENV_SYSTEM_INCLUDE_DIR}");
    println!("cargo:rerun-if-env-changed={ORT_ENV_SYSTEM_LIB_DIR}");

    generate_bindings(&include_dir);
}

#[cfg(not(feature = "generate-bindings"))]
fn generate_bindings(_include_dir: &Path) {
    println!("Bindings not generated automatically, using committed files instead.");
    println!("Enable with the 'generate-bindings' cargo feature.");
}

#[cfg(feature = "generate-bindings")]
fn generate_bindings(include_dir: &Path) {
    let clang_args = &[
        format!("-I{}", include_dir.display()),
        format!("-I{}", include_dir.join("onnxruntime").join("core").join("session").display()),
    ];

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed=src/generated/bindings.rs");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        // The current working directory is 'onnxruntime-sys'
        .clang_args(clang_args)
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // Set `size_t` to be translated to `usize` for win32 compatibility.
        .size_t_is_usize(true)
        // Format using rustfmt
        .formatter(bindgen::Formatter::Rustfmt)
        .rustified_enum(".*")
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to (source controlled) src/generated/<os>/<arch>/bindings.rs
    let generated_file = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("src")
        .join("generated")
        .join(env::var("CARGO_CFG_TARGET_OS").unwrap())
        .join(env::var("CARGO_CFG_TARGET_ARCH").unwrap())
        .join("bindings.rs");
    println!("cargo:rerun-if-changed={generated_file:?}");
    bindings.write_to_file(&generated_file).expect("Couldn't write bindings!");
}

fn download<P>(source_url: &str, target_file: P)
where
    P: AsRef<Path>,
{
    let resp = ureq::get(source_url)
        .timeout(std::time::Duration::from_secs(300))
        .call()
        .unwrap_or_else(|err| panic!("ERROR: Failed to download {source_url}: {err:?}"));

    let len = resp.header("Content-Length").and_then(|s| s.parse::<usize>().ok()).unwrap();
    let mut reader = resp.into_reader();
    // FIXME: Save directly to the file
    let mut buffer = vec![];
    let read_len = reader.read_to_end(&mut buffer).unwrap();
    assert_eq!(buffer.len(), len);
    assert_eq!(buffer.len(), read_len);

    let f = fs::File::create(&target_file).unwrap();
    let mut writer = io::BufWriter::new(f);
    writer.write_all(&buffer).unwrap();
}

fn extract_archive(filename: &Path, output: &Path) {
    match filename.extension().map(|e| e.to_str()) {
        #[cfg(windows)]
        Some(Some("zip")) => extract_zip(filename, output),
        #[cfg(unix)]
        Some(Some("tgz")) => extract_tgz(filename, output),
        _ => unimplemented!(),
    }
}

#[cfg(unix)]
fn extract_tgz(filename: &Path, output: &Path) {
    let file = fs::File::open(filename).unwrap();
    let buf = io::BufReader::new(file);
    let tar = flate2::read::GzDecoder::new(buf);
    let mut archive = tar::Archive::new(tar);
    archive.unpack(output).unwrap();
}

#[cfg(windows)]
fn extract_zip(filename: &Path, outpath: &Path) {
    let file = fs::File::open(filename).unwrap();
    let buf = io::BufReader::new(file);
    let mut archive = zip::ZipArchive::new(buf).unwrap();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        #[allow(deprecated)]
        let outpath = outpath.join(file.sanitized_name());
        if !(&*file.name()).ends_with('/') {
            println!(
                "File {} extracted to \"{}\" ({} bytes)",
                i,
                outpath.as_path().display(),
                file.size()
            );
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(&p).unwrap();
                }
            }
            let mut outfile = fs::File::create(&outpath).unwrap();
            io::copy(&mut file, &mut outfile).unwrap();
        }
    }
}

trait OnnxPrebuiltArchive {
    fn as_onnx_str(&self) -> Cow<str>;
}

#[derive(Debug)]
enum Architecture {
    X86,
    X86_64,
    Arm,
    Arm64,
}

impl FromStr for Architecture {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "x86" => Ok(Architecture::X86),
            "x86_64" => Ok(Architecture::X86_64),
            "arm" => Ok(Architecture::Arm),
            "aarch64" => Ok(Architecture::Arm64),
            _ => Err(format!("Unsupported architecture: {s}")),
        }
    }
}

impl OnnxPrebuiltArchive for Architecture {
    fn as_onnx_str(&self) -> Cow<str> {
        match self {
            Architecture::X86 => Cow::from("x86"),
            Architecture::X86_64 => Cow::from("x64"),
            Architecture::Arm => Cow::from("arm"),
            Architecture::Arm64 => Cow::from("arm64"),
        }
    }
}

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
enum Os {
    Windows,
    Linux,
    MacOs,
}

impl Os {
    fn archive_extension(&self) -> &'static str {
        match self {
            Os::Windows => "zip",
            Os::Linux => "tgz",
            Os::MacOs => "tgz",
        }
    }
}

impl FromStr for Os {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "windows" => Ok(Os::Windows),
            "macos" => Ok(Os::MacOs),
            "linux" => Ok(Os::Linux),
            _ => Err(format!("Unsupported os: {s}")),
        }
    }
}

impl OnnxPrebuiltArchive for Os {
    fn as_onnx_str(&self) -> Cow<str> {
        match self {
            Os::Windows => Cow::from("win"),
            Os::Linux => Cow::from("linux"),
            Os::MacOs => Cow::from("osx"),
        }
    }
}

#[derive(Debug)]
enum Accelerator {
    None,
    Gpu,
}

impl FromStr for Accelerator {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "1" | "yes" | "true" | "on" => Ok(Accelerator::Gpu),
            _ => Ok(Accelerator::None),
        }
    }
}

impl OnnxPrebuiltArchive for Accelerator {
    fn as_onnx_str(&self) -> Cow<str> {
        match self {
            Accelerator::None => Cow::from(""),
            Accelerator::Gpu => Cow::from("gpu"),
        }
    }
}

#[derive(Debug)]
struct Triplet {
    os: Os,
    arch: Architecture,
    accelerator: Accelerator,
}

impl OnnxPrebuiltArchive for Triplet {
    fn as_onnx_str(&self) -> Cow<str> {
        match (&self.os, &self.arch, &self.accelerator) {
            // onnxruntime-win-x86-1.10.0.zip
            // onnxruntime-win-x64-1.10.0.zip
            // onnxruntime-win-arm-1.10.0.zip
            // onnxruntime-win-arm64-1.10.0.zip
            // onnxruntime-linux-x64-1.10.0.tgz
            // onnxruntime-osx-arm64-1.10.0.tgz
            (Os::Windows, Architecture::X86, Accelerator::None)
            | (Os::Windows, Architecture::X86_64, Accelerator::None)
            | (Os::Windows, Architecture::Arm, Accelerator::None)
            | (Os::Windows, Architecture::Arm64, Accelerator::None)
            | (Os::Linux, Architecture::X86_64, Accelerator::None)
            | (Os::MacOs, Architecture::Arm64, Accelerator::None) => {
                Cow::from(format!("{}-{}", self.os.as_onnx_str(), self.arch.as_onnx_str()))
            }
            // onnxruntime-linux-aarch64-1.10.0.tgz
            (Os::Linux, Architecture::Arm64, Accelerator::None) => {
                Cow::from(format!("{}-{}", self.os.as_onnx_str(), "aarch64"))
            }
            // onnxruntime-osx-x86_64-1.10.0.tgz
            (Os::MacOs, Architecture::X86_64, Accelerator::None) => {
                Cow::from(format!("{}-{}", self.os.as_onnx_str(), "x86_64"))
            }
            // onnxruntime-win-x64-gpu-1.10.0.zip
            // onnxruntime-linux-x64-gpu-1.10.0.tgz
            (Os::Windows, Architecture::X86_64, Accelerator::Gpu)
            | (Os::Linux, Architecture::X86_64, Accelerator::Gpu) => Cow::from(format!(
                "{}-{}-{}",
                self.os.as_onnx_str(),
                self.arch.as_onnx_str(),
                self.accelerator.as_onnx_str(),
            )),
            _ => {
                panic!(
                    "Unsupported prebuilt triplet: {:?}, {:?}, {:?}. Please use {}=system, {}=/path/to/onnxruntime_include_dir, and {}=/path/to/onnxruntime_lib_dir",
                    self.os,
                    self.arch,
                    self.accelerator,
                    ORT_ENV_STRATEGY,
                    ORT_ENV_SYSTEM_INCLUDE_DIR,
                    ORT_ENV_SYSTEM_LIB_DIR,
                );
            }
        }
    }
}

fn prebuilt_archive_url() -> (PathBuf, String) {
    let triplet = Triplet {
        os: env::var("CARGO_CFG_TARGET_OS").expect("Unable to get TARGET_OS").parse().unwrap(),
        arch: env::var("CARGO_CFG_TARGET_ARCH")
            .expect("Unable to get TARGET_ARCH")
            .parse()
            .unwrap(),
        accelerator: env::var(ORT_ENV_GPU).unwrap_or_default().parse().unwrap(),
    };

    let prebuilt_archive = format!(
        "onnxruntime-{}-{}.{}",
        triplet.as_onnx_str(),
        ORT_VERSION,
        triplet.os.archive_extension()
    );
    let prebuilt_url = format!("{ORT_RELEASE_BASE_URL}/v{ORT_VERSION}/{prebuilt_archive}");

    (PathBuf::from(prebuilt_archive), prebuilt_url)
}

fn prepare_libort_dir_prebuilt() -> PathBuf {
    let (prebuilt_archive, prebuilt_url) = prebuilt_archive_url();

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let extract_dir = out_dir.join(ORT_PREBUILT_EXTRACT_DIR);
    let downloaded_file = out_dir.join(&prebuilt_archive);

    if !downloaded_file.exists() {
        println!("Creating directory {out_dir:?}");
        fs::create_dir_all(&out_dir).unwrap();

        println!("Downloading {} into {}", prebuilt_url, downloaded_file.display());
        download(&prebuilt_url, &downloaded_file);
    }

    if !extract_dir.exists() {
        println!("Extracting to {}...", extract_dir.display());
        extract_archive(&downloaded_file, &extract_dir);
    }

    extract_dir.join(prebuilt_archive.file_stem().unwrap())
}

fn prepare_libort_dir() -> (PathBuf, PathBuf) {
    let strategy = env::var(ORT_ENV_STRATEGY);
    println!("strategy: {:?}", strategy.as_ref().map(String::as_str).unwrap_or_else(|_| "unknown"));
    match strategy.as_ref().map(String::as_str) {
        Ok("download") | Err(_) => {
            let libort_install_dir = prepare_libort_dir_prebuilt();
            let include_dir = libort_install_dir.join("include");
            let lib_dir = libort_install_dir.join("lib");
            (include_dir, lib_dir)
        }
        Ok("system") => {
            let include_dir = match env::var(ORT_ENV_SYSTEM_INCLUDE_DIR) {
                Ok(p) => PathBuf::from(p),
                Err(e) => {
                    panic!("Could not get value of environment variable {ORT_ENV_SYSTEM_INCLUDE_DIR:?}: {e:?}");
                }
            };
            let lib_dir = match env::var(ORT_ENV_SYSTEM_LIB_DIR) {
                Ok(p) => PathBuf::from(p),
                Err(e) => {
                    panic!("Could not get value of environment variable {ORT_ENV_SYSTEM_LIB_DIR:?}: {e:?}");
                }
            };
            (include_dir, lib_dir)
        }
        Ok("compile") => unimplemented!(),
        _ => panic!("Unknown value for {ORT_ENV_STRATEGY:?}"),
    }
}
