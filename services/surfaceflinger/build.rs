fn main() {
    let target = std::env::var("TARGET").unwrap();
    let is_cross = target.contains("aarch64");
    
    let mut build = cc::Build::new();
    build.cpp(true)
         .std("c++17")
         .file("wl/wl_compositor.cpp")
         .include("include");

    // Wayland support is only enabled when not cross-compiling
    let mut use_wayland = false;
    if !is_cross {
        if let Ok(lib) = pkg_config::Config::new().atleast_version("0.18").probe("wlroots-0.18") {
            use_wayland = true;
            for path in lib.include_paths {
                build.include(path);
            }
        }
        if use_wayland {
            if let Ok(lib) = pkg_config::probe_library("wayland-server") {
                for path in lib.include_paths {
                    build.include(path);
                }
            }
        }
    }

    if use_wayland {
        build.define("TRACK_WAYLAND", None);
        build.define("WLR_USE_UNSTABLE", None);
        
        // Link Wayland and wlroots libraries
        println!("cargo:rustc-link-lib=wayland-server");
        println!("cargo:rustc-link-lib=wlroots-0.18");
        
        let protocol_src = std::path::Path::new("src/xdg-shell-protocol.c");
        if protocol_src.exists() {
            let mut build_c = cc::Build::new();
            build_c.file("src/xdg-shell-protocol.c")
                   .include("include");
            if let Ok(lib) = pkg_config::probe_library("wayland-server") {
                for path in lib.include_paths {
                    build_c.include(path);
                }
            }
            build_c.compile("xdg_shell_protocol");
        }
    }
    
    build.compile("wl_compositor");
    
    // Rerun triggers
    println!("cargo:rerun-if-changed=wl/wl_compositor.cpp");
    println!("cargo:rerun-if-changed=wl/wl_compositor.h");
}
