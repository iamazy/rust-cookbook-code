extern crate flate2;
extern crate tar;

use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::File;
use tar::Archive;

pub fn decompress_tarball() -> Result<(), std::io::Error> {
    let path = "/Users/iamazy/Downloads/curl-7.68.0.tar.gz";
    let tar_gz = File::open(&path)?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    archive.unpack(".")?;

    Ok(())
}

pub fn compress_dir() -> Result<(), std::io::Error> {
    let tar_gz = File::create("archive.tar.gz")?;
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = tar::Builder::new(enc);
    let mut file = File::open("./Cargo.toml").unwrap();
    // 压缩包里的路径
    tar.append_file("./bar/Cargo.toml", &mut file)?;
    Ok(())
}

// pub fn decompress_tarball2() -> Result<(), std::io::Error> {
//     let file = File::open("archive.tar.gz")?;
//     let mut archive = Archive::new(GzDecoder::new(file));
//     let prefix = "bundle/logs";

//     println!("Extracted the following files");
//     archive
//         .entries()?
//         .filter_map(|e| e.ok())
//         .map(|mut entry| -> Result<PathBuf, std::io::Error> {
//             let path = entry.path()?.strip_prefix(prefix).to_owned();
//             entry.unpack(&path)?;
//             Ok(path)
//         })
//         .filter_map(|e| e.ok())
//         .for_each(|x| println!("> {}", x.display()));
//     Ok(())
// }
