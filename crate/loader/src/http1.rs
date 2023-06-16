use flate2::bufread;
use reqwest::header;
use shared::config::path;
use std::{fs::File, io::copy, io::BufReader, path::PathBuf};
use tokio::io::AsyncWriteExt;

use shared::model::Ressource;

async fn content_length(client: &reqwest::Client, url: &str) -> u64 {
    if let Ok(resp) = client.head(url).send().await {
        if resp.status().is_success() {
            resp.headers()
                .get(header::CONTENT_LENGTH)
                .and_then(|ct_len| ct_len.to_str().ok())
                .and_then(|ct_len| ct_len.parse::<u64>().ok())
                .unwrap()
        } else {
            panic!("send request is failing.")
        }
    } else {
        panic!("cannot send head request.")
    }
}

async fn prepare_outfile(
    model_name: &str,
    ressource_name: &str,
    defalte: bool,
) -> (tokio::fs::File, PathBuf) {
    let filedir = path::data().join(model_name);
    if !filedir.exists() {
        tokio::fs::create_dir(&filedir).await.ok().unwrap();
    }

    let mut filepath = path::data().join(model_name).join(ressource_name);
    if filepath.exists() {
        tokio::fs::remove_file(&filepath).await.ok().unwrap();
    }
    if defalte {
        filepath.set_extension("txt.gz");
        if filepath.exists() {
            tokio::fs::remove_file(&filepath).await.ok().unwrap();
        }
    }
    let file = tokio::fs::File::create(&filepath).await.ok().unwrap();
    (file, filepath)
}

pub async fn download(
    multibar: indicatif::MultiProgress,
    model_name: &str,
    ressource: Ressource,
) -> Result<(), &'static str> {
    let client = reqwest::Client::builder()
        .build()
        .map_err(|_| "cannot build req client")?;

    let download_size = content_length(&client, &ressource.url).await;

    let (mut file, filepath) =
        prepare_outfile(model_name, &ressource.name, ressource.deflate).await;

    let progress_bar = multibar.add(indicatif::ProgressBar::new(download_size));

    progress_bar.set_style(
        indicatif::ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
        )
        .unwrap()
        .progress_chars("##-"),
    );

    let set_fmt = |downloaded: u64| {
        format!(
            "{:?}, downloaded: {downloaded}, ratio: {}",
            ressource.name,
            downloaded / download_size * 100u64
        )
    };

    let mut stream = match client.get(&ressource.url).send().await {
        Err(error) => {
            eprintln!("{error:?}");
            panic!("failed")
        }
        Ok(stream) => stream,
    };

    let mut counter = 0u64;
    let mut downloaded = 0u64;
    while let Some(chunk) = stream.chunk().await.map_err(|_| "chunk read failed")? {
        file.write_all(&chunk)
            .await
            .map_err(|_| "chunk write failed")?;

        downloaded += chunk.len() as u64;
        counter += 1;
        if counter % 10 == 9 {
            progress_bar.set_message(set_fmt(downloaded));
        }
        progress_bar.inc(chunk.len() as u64);
    }
    file.flush().await.ok().unwrap();

    progress_bar.println(format!("Loading of URL: {:?} is done!", ressource.url));

    if ressource.deflate {
        unzip_and_rename(filepath).await?;
    };

    Ok(())
}

async fn unzip_and_rename(mut path: PathBuf) -> Result<(), &'static str> {
    let input = BufReader::new(File::open(path.as_path()).unwrap());

    path.set_extension("");
    let mut output = File::create(path.as_path()).unwrap();

    let mut decoder = bufread::GzDecoder::new(input);
    copy(&mut decoder, &mut output).unwrap();

    path.set_extension("txt.gz");
    std::fs::remove_file(path.as_path()).ok().unwrap();

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn ext() {
        let model_name = "stable_diffusion_2_1";
        let ressource_name = "vocab.txt";
        let mut ressource_path = shared::config::path::data()
            .join(model_name)
            .join(ressource_name);
        println!("{ressource_path:?}");

        ressource_path.set_extension("txt.gz");
        println!("{ressource_path:?}");

        ressource_path.set_extension("");
        println!("{ressource_path:?}");

        ressource_path.set_extension("txt.gz");
        println!("{ressource_path:?}");
    }
}
