use tokio::{fs::File, io::AsyncWriteExt};

use crate::{config::file as config, Result};

pub async fn update_tests(data: &[u8]) -> Result<()>{
    let mut file= File::create(format!("{}.tar.gz", config::TESTS)).await?;
    file.write_all(data).await?;
    Ok(())
}

pub async fn get_tests() -> Result<File> {
    Ok(File::open(format!("{}.tar.gz", config::TESTS)).await?)
}

pub async fn get_statements() -> Result<File> {
    Ok(File::open(format!("{}.tar.gz", config::STATEMENTS)).await?)
}



