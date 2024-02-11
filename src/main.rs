extern crate serde;
extern crate serde_json;
use std::{fs, io::Read, net::TcpStream, process::exit};
use serde::Deserialize;

#[derive(Deserialize)]
struct Metadata {
    filename: String
}
fn main() -> std::io::Result<()> {
    let mut client = TcpStream::connect("127.0.0.1:8000")?;
    let mut buffer: Vec<u8> = Vec::new();
    if let Err(e) = client.read_to_end(&mut buffer) {
        eprintln!("Could not read contents easy: {e}");
        std::process::exit(1);
    }
    let mut split = buffer.splitn(2, |e| e == &b'\0');
    let json_part = split.next().unwrap_or(&[]);
    let data_part = split.next().unwrap_or(&[]);
    let deserialized_metadata: Metadata = match serde_json::from_slice(json_part) {
        Ok(e) => e,
        Err(error) => {
            eprintln!("Error when deserializing json data: {error}");
            exit(1);
        }
    };
    if let Err(e) = fs::write(&deserialized_metadata.filename, data_part) {
        eprintln!("Could not dump data to file: {e}");
        exit(1);
    }
    if is_binary(data_part) && !cfg!(target_os="windows") {
        println!("File is binary and therefore must be chmodded (sudo chmod +x '{}') - do this if you trust the source!", deserialized_metadata.filename);
    }

    let file_description = std::process::Command::new("file")
    .arg(&deserialized_metadata.filename)
    .output()?;
    println!("file name: {}", deserialized_metadata.filename);
    println!("file description: \"{}\"", String::from_utf8_lossy(&file_description.stdout).trim_end());
    println!("path to file: {}", fs::canonicalize(deserialized_metadata.filename)?.display());


    Ok(())
}
fn is_binary(data: &[u8]) -> bool {
    for each in data.chunks(1024) {
        if each.contains(&b'\0') {
            return true;
        }
    }
    false
}