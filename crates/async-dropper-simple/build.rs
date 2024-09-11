use rustc_version::{version_meta, Channel};

fn main() {
    // Set cfg flags depending on release channel
    let channel = match version_meta().unwrap().channel {
        Channel::Stable => "stable",
        Channel::Beta => "beta",
        Channel::Nightly => "nightly",
        Channel::Dev => "dev",
    };
    println!("cargo:rustc-check-cfg=cfg(channel, values(\"stable\", \"beta\", \"nightly\", \"dev\"))");
    println!("cargo:rustc-check=channel={}", channel)
}
