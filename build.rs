use std::process::Command;
use std::{
    env,
    error::Error,
    path::{Path, PathBuf},
};

const SOURCES: &[&str] = &["flanterm.c", "backends/fb.c"];

fn init_submodule(flanterm_path: &Path) {
    if !flanterm_path.join("README.md").exists() {
        Command::new("git")
            .args(["submodule", "update", "--init"])
            .current_dir(flanterm_path)
            .status()
            .expect("failed to retrieve flanterm sources with git");
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let project_dir = env::var_os("CARGO_MANIFEST_DIR").unwrap();
    let flanterm_path = Path::new(&project_dir).join("flanterm");

    init_submodule(&flanterm_path);

    let flanterm_path_str = flanterm_path.to_str().unwrap();

    let sources = SOURCES
        .iter()
        .map(|file| format!("{flanterm_path_str}/{file}"));

    let mut cc = cc::Build::new();

    cc.files(sources)
        .define("FLANTERM_FB_DISABLE_BUMP_ALLOC", "") // reduces binary size but needs memory allocator
        // .flag("-std=c11")
        // .flag("-mgeneral-regs-only")
        .flag("-nostdlib")
        .flag("-ffreestanding")
        .flag("-fno-stack-protector")
        .flag("-fno-stack-check")
        .flag("-mno-red-zone")
        .flag("-mcmodel=kernel")
        .flag("-fno-PIC")
        .flag("-fno-PIE");

    cc.compile("flanterm");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .clang_args(&["-ffreestanding"])
        .prepend_enum_name(false)
        .use_core()
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    Ok(())
}
