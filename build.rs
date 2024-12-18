use std::error::Error;

use serde::{Deserialize, Serialize};
use vergen_gitcl::{Emitter, GitclBuilder};

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
        "https://raw.githubusercontent.com/a2x/cs2-dumper/refs/heads/main/output/client_dll.rs",
        "./src/dma/cs2dumper/client_mod.rs"
    ).expect("Failed to download build file \"client.dll.rs\"");

    download(
        "https://raw.githubusercontent.com/a2x/cs2-dumper/refs/heads/main/output/offsets.rs",
        "./src/dma/cs2dumper/offsets_mod.rs"
    ).expect("Failed to download build file \"offsets.rs\"");

    download(
        "https://raw.githubusercontent.com/a2x/cs2-dumper/refs/heads/main/output/engine2_dll.rs",
        "./src/dma/cs2dumper/engine2_mod.rs"
    ).expect("Failed to download build file \"engine2.dll.rs\"");

    build_number()?;

    let gitcl = GitclBuilder::all_git()?;


    Emitter::new()
        .add_instructions(&gitcl)?
        .emit()?;

    Ok(())
}