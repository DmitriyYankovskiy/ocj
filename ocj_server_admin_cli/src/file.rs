use std::io::BufWriter;
use flate2::{write::GzEncoder, Compression};
use ocj_config::{contest::File, file as config};

pub fn compress(dir_path: &str) -> std::io::Result<Box<crate::config::contest::File>> {
    let mut w = Vec::new();
    {
        let buf = BufWriter::new(&mut w);
        let encoder = GzEncoder::new(buf, Compression::default());
        let mut builder = tar::Builder::new(encoder);
        builder.append_dir_all("", dir_path)?;
        builder.finish()?;
    }
    Ok(Box::from(w))
}

pub fn get_compressed_tests() -> std::io::Result<Box<File>> {
    compress(config::TESTS)
}

pub fn get_compressed_statements() -> std::io::Result<Box<File>> {
    compress(config::STATEMENTS)
}
