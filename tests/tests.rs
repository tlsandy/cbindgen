extern crate cbindgen;

use cbindgen::*;
use std::path::Path;
use std::process::Command;
use std::{env, fs, str};

fn run_cbindgen(
    cbindgen_path: &'static str,
    path: &Path,
    output: &Path,
    language: Language,
    cpp_compat: bool,
    style: Option<Style>,
) {
    let program = Path::new(cbindgen_path);
    let mut command = Command::new(&program);
    match language {
        Language::Cxx => {}
        Language::C => {
            command.arg("--lang").arg("c");

            if cpp_compat {
                command.arg("--cpp-compat");
            }
        },
        Language::CS => {
            command.arg("--lang").arg("c#");
        }
    }

    if let Some(style) = style {
        command.arg("--style");
        command.arg(match style {
            Style::Both => "both",
            Style::Tag => "tag",
            Style::Type => "type",
        });
    }

    command.arg("-o").arg(output);

    if env::var("CBINDGEN_TEST_VERIFY").is_ok() {
        command.arg("--verify");
    }

    let mut config = path.clone().to_path_buf();
    config.set_extension("toml");
    if config.exists() {
        command.arg("--config").arg(config);
    }

    command.arg(path);

    println!("Running: {:?}", command);
    let cbindgen_output = command.output().expect("failed to execute process");
    assert!(
        cbindgen_output.status.success(),
        "cbindgen failed: {:?} with error: {}",
        output,
        str::from_utf8(&cbindgen_output.stderr).unwrap_or_default()
    );
}

fn compile(cbindgen_output: &Path, language: Language) {
    let cc = match language {
        Language::Cxx => env::var("CXX").unwrap_or_else(|_| "g++".to_owned()),
        Language::C => env::var("CC").unwrap_or_else(|_| "gcc".to_owned()),
        Language::CS => "C:/Program Files (x86)/Microsoft Visual Studio/2019/Community/MSBuild/Current/Bin/Roslyn/csc.exe".to_owned()
    };

    let mut object = cbindgen_output.to_path_buf();
    match language {
        Language::Cxx | Language::C => object.set_extension("o"),
        Language::CS => object.set_extension("dll")
    };

    let mut command = Command::new(cc);
    match language {
        Language::Cxx | Language::C => {
            command.arg("-D").arg("DEFINED");
            command.arg("-c").arg(cbindgen_output);
            command.arg("-o").arg(&object);
            if let Language::Cxx = language {
                // enum class is a c++11 extension which makes g++ on macos 10.14 error out
                command.arg("-std=c++11");
            }
        },
        Language::CS => {
            command.arg("-target:library");
            command.arg(format!("-out:{}", object.to_str().unwrap()));
            command.arg(cbindgen_output);
        }
    }
    

    println!("Running: {:?}", command);
    let out = command.output().expect("failed to compile");
    assert!(out.status.success(), "Output failed to compile: {:?}", out);

    if object.exists() {
        fs::remove_file(object).unwrap();
    }
}

fn run_compile_test(
    cbindgen_path: &'static str,
    name: &'static str,
    path: &Path,
    language: Language,
    cpp_compat: bool,
    style: Option<Style>,
) {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut output = Path::new(&crate_dir).join("tests").join("expectations");
    if let Some(style) = style {
        match style {
            Style::Both => {
                output.push("both");
            }
            Style::Tag => {
                output.push("tag");
            }
            Style::Type => {}
        }
    }

    let ext = match language {
        Language::Cxx => "cpp",
        Language::C => {
            if cpp_compat {
                "compat.c"
            } else {
                "c"
            }
        },
        Language::CS => "cs"
    };

    output.push(format!("{}.{}", name, ext));

    run_cbindgen(cbindgen_path, path, &output, language, cpp_compat, style);
    compile(&output, language);

    if language == Language::C && cpp_compat {
        compile(&output, Language::Cxx)
    }
}

fn test_file(cbindgen_path: &'static str, name: &'static str, filename: &'static str) {
    let test = Path::new(filename);
    for style in &[Style::Type, Style::Tag, Style::Both] {
        for cpp_compat in &[true, false] {
            run_compile_test(
                cbindgen_path,
                name,
                &test,
                Language::C,
                *cpp_compat,
                Some(*style),
            );
        }
    }
    run_compile_test(
        cbindgen_path,
        name,
        &test,
        Language::Cxx,
        /* cpp_compat = */ false,
        None,
    );
    run_compile_test(
        cbindgen_path,
        name,
        &test,
        Language::CS,
        /* cpp_compat = */ false,
        None
    );
}

macro_rules! test_file {
    ($cbindgen_path:expr, $test_function_name:ident, $name:expr, $file:tt) => {
        #[test]
        fn $test_function_name() {
            test_file($cbindgen_path, $name, $file);
        }
    };
}

// This file is generated by build.rs
include!(concat!(env!("OUT_DIR"), "/tests.rs"));
