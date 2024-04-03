use std::error::Error;

use serde::{Deserialize, Serialize};
use vergen::EmitBuilder;

#[derive(Clone, Deserialize, Serialize)]
struct InfoJson {
    build_number: usize,
    timestamp: String
}

fn download(url: &str, to: &str) -> Result<(), Box<dyn Error>> {
    let content = reqwest::blocking::get(url)
        .unwrap_or_else(|_| panic!("Downloading \"{to}\""))
        .text()
        .expect("Convert response to text");

    std::fs::write(to, content)
    .expect("Write to file");

    Ok(())
}

fn build_number() -> Result<(), Box<dyn Error>> {
    let content = reqwest::blocking::get("https://raw.githubusercontent.com/a2x/cs2-dumper/main/output/info.json")
        .unwrap_or_else(|_| panic!("Downloading info.json"))
        .text()
        .expect("Convert response to text");

    let info = serde_json::from_str::<InfoJson>(&content)?;
    println!("cargo:rustc-env=CS2_BUILD_NUMBER={}", info.build_number);

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {

    download(
        "https://raw.githubusercontent.com/a2x/cs2-dumper/main/output/client.dll.rs",
        "./src/dma/cs2dumper/client_mod.rs"
    ).expect("Failed to download build file \"client.dll.rs\"");

    download(
        "https://raw.githubusercontent.com/a2x/cs2-dumper/main/output/offsets.rs",
        "./src/dma/cs2dumper/offsets_mod.rs"
    ).expect("Failed to download build file \"offsets.rs\"");

    download(
        "https://raw.githubusercontent.com/a2x/cs2-dumper/main/output/engine2.dll.rs",
        "./src/dma/cs2dumper/engine2_mod.rs"
    ).expect("Failed to download build file \"engine2.dll.rs\"");

    build_number()?;

    EmitBuilder::builder()
        .git_sha(true)
        .git_commit_date()
        .cargo_debug()
        .cargo_target_triple()
        .rustc_semver()
        .rustc_llvm_version()
        .emit()?;

    Ok(())
}