use std::path::Path;
use tokio::{fs::{self, File}, io::BufReader};
use tokio_tar::Archive;

use crate::{config::file as config, judge};

pub async fn init() {
    _ = fs::create_dir(judge::DIR).await;
}

pub async fn decompress(dir_path: &Path) -> Result<(), ()> {
    let tar_file = format!("{}.tar.gz", dir_path.to_str().unwrap());
    let file = File::open(&tar_file).await;
    let file = if let Ok(f) = file {f} else {
        log::error!("file with name \"{tar_file}\" does not exist");
        return Err(());
    };
    let buf_reader = BufReader::new(file);
    let dec = async_compression::tokio::bufread::GzipDecoder::new(buf_reader);

    let mut archive = Archive::new(dec);

    archive.unpack("tests").await.unwrap(); 
    Ok(())
}

pub async fn decompress_tests() {
    decompress(Path::new(config::TESTS)).await.unwrap();
}