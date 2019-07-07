#[allow(warnings)]
extern crate gcc;
use std::env;
use std::process::Command;


fn main(){
    let mut cfg = gcc::Config::new();
    let env = env::var("TARGET").unwrap();

    cfg.file("osdialog/osdialog.c");

    if env.contains("darwin") {
        cfg.file("osdialog/osdialog_mac.m");
        cfg.compile("libnfd.a");
        println!("cargo:rustc-link-lib=framework=AppKit");
    }else if env.contains("windows") {
        cfg.file("osdialog/osdialog_win.c");
        cfg.compile("libnfd.a");
        
        println!("cargo:rustc-link-lib=comdlg32");
        // println!("cargo:rustc-link-lib=ole32");
        println!("cargo:rustc-link-lib=shell32");
        // MinGW doesn't link it by default
        // println!("cargo:rustc-link-lib=uuid");

    }else{
        let pkg_output = Command::new("pkg-config")
            .arg("--cflags")
            .arg("gtk+-3.0")
            .output();

        match pkg_output {
            Ok(output) => {
                let t = String::from_utf8(output.stdout).unwrap();
                let flags = t.split(" ");
                for flag in flags {
                    if flag != "\n" && flag != "" {
                        cfg.flag(flag);
                    }
                }
            }
            _ => (),
        }

        cfg.file("osdialog/osdialog_gtk3.c");
        cfg.compile("libnfd.a");
        println!("cargo:rustc-link-lib=gdk-3");
        println!("cargo:rustc-link-lib=gtk-3");
        println!("cargo:rustc-link-lib=glib-2.0");
        println!("cargo:rustc-link-lib=gobject-2.0");
    }
}