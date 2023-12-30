use std::error::Error;

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
        "./src/cs2dumper/client.rs"
    ).expect("Failed to download build file \"client.dll.rs\"");

    download(
        "https://raw.githubusercontent.com/a2x/cs2-dumper/main/generated/offsets.rs",
        "./src/cs2dumper/offsets.rs"
    ).expect("Failed to download build file \"offsets.rs\"");

    download(
        "https://raw.githubusercontent.com/a2x/cs2-dumper/main/generated/engine2.dll.rs",
        "./src/cs2dumper/engine2.rs"
    ).expect("Failed to download build file \"engine2.dll.rs\"");

    Ok(())
}