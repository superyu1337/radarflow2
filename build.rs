use std::error::Error;
use vergen::EmitBuilder;

fn download(url: &str, to: &str) -> Result<(), Box<dyn Error>> {
    let content = reqwest::blocking::get(url)
        .unwrap_or_else(|_| panic!("Downloading \"{to}\""))
        .text()
        .expect("Convert response to text");

    std::fs::write(to, content)
    .expect("Write to file");

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {

    download(
        "https://raw.githubusercontent.com/a2x/cs2-dumper/main/generated/client.dll.rs",
        "./src/sdk/cs2dumper/client.rs"
    ).expect("Failed to download build file \"client.dll.rs\"");

    download(
        "https://raw.githubusercontent.com/a2x/cs2-dumper/main/generated/offsets.rs",
        "./src/sdk/cs2dumper/offsets.rs"
    ).expect("Failed to download build file \"offsets.rs\"");

    download(
        "https://raw.githubusercontent.com/a2x/cs2-dumper/main/generated/engine2.dll.rs",
        "./src/sdk/cs2dumper/engine2.rs"
    ).expect("Failed to download build file \"engine2.dll.rs\"");

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