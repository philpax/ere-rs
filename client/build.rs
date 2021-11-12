use std::env;
use std::path::Path;

fn compile_protobuf(protobuf_build_path: &Path) -> anyhow::Result<()> {
    println!("cargo:warning=compiling protobuf");
    cmake::Config::new("../external/protobuf/cmake")
        .out_dir(protobuf_build_path)
        .profile("Release")
        .build();

    Ok(())
}

fn compile_proto_defs(our_path: &Path, protoc_path: &Path) -> anyhow::Result<()> {
    use std::process::Command;

    println!("cargo:warning=compiling proto files");
    let parent_path = our_path.parent().unwrap();
    {
        env::set_current_dir(parent_path)?;
        let output = Command::new(protoc_path)
            .args(["--cpp_out=client/cpp/src", "proto/rocktree.proto"])
            .output()?;
        println!("cargo:warning={}", String::from_utf8_lossy(&output.stdout));
        env::set_current_dir(our_path)?;
    }

    Ok(())
}

fn compile_proto(our_path: &Path) -> anyhow::Result<()> {
    let protobuf_build_path = dunce::canonicalize(our_path.join("../build/protobuf"))?;

    let protoc_path = protobuf_build_path.join("bin").join("protoc.exe");
    let protobuf_lib_dir = protobuf_build_path.join("lib");
    let protobuf_lib_path = protobuf_lib_dir.join("libprotobuf.lib");

    if !(protoc_path.is_file() && protobuf_lib_path.is_file()) {
        compile_protobuf(&protobuf_build_path)?;
    }

    if !our_path.join("cpp").join("src").join("proto").is_dir() {
        compile_proto_defs(&our_path, &protoc_path)?;
    }

    println!(
        "cargo:rustc-link-search=native={}",
        protobuf_lib_dir.display()
    );
    println!("cargo:rustc-link-lib=static=libprotobuf");

    Ok(())
}

fn compile_sdl(our_path: &Path) -> anyhow::Result<()> {
    let sdl_build_path = dunce::canonicalize(our_path.join("../build/sdl"))?;
    let sdl_lib_dir = sdl_build_path.join("lib");
    let sdl_lib_path = sdl_lib_dir.join("SDL2.lib");

    if !sdl_lib_path.is_file() {
        println!("cargo:warning=compiling sdl");
        cmake::Config::new("../external/sdl")
            .out_dir(&sdl_build_path)
            .profile("Release")
            .build();
    }

    println!("cargo:rustc-link-search=native={}", sdl_lib_dir.display());
    println!("cargo:rustc-link-lib=static=SDL2");

    let sdl_dll_path = sdl_build_path.join("bin").join("SDL2.dll");
    let out_dir = dunce::canonicalize(env::var("OUT_DIR")?)?;
    let sdl_dll_out_path = out_dir.join("SDL2.dll");
    std::fs::copy(sdl_dll_path, sdl_dll_out_path)?;

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let our_path = env::current_dir()?;

    std::fs::create_dir_all("../build/protobuf")?;
    std::fs::create_dir_all("../build/sdl")?;

    compile_proto(&our_path)?;
    compile_sdl(&our_path)?;

    cc::Build::new()
        .static_crt(true)
        .shared_flag(false)
        .cpp(true)
        .file("cpp/src/crn/crn.cc")
        .file("cpp/src/main.cpp")
        .include("cpp/src/crn")
        .include("cpp/src")
        .include("cpp/include")
        .include("../external")
        .include("../external/eigen")
        .include("../external/protobuf/src")
        .include("../external/gl2/include")
        .include("../external/sdl/include")
        .flag("/std:c++14")
        .flag("/EHsc")
        .define("_CRT_SECURE_NO_WARNINGS", None)
        .define("WIN32_LEAN_AND_MEAN", None)
        .compile("ere");

    // println!("cargo:warning=hello!");

    Ok(())
}
