pub mod http1;
pub mod http2;

use shared::config::path;
use shared::model::Models;
use tokio::task;

pub async fn clean() -> Result<(), &'static str> {
    tokio::fs::remove_dir_all(&path::data())
        .await
        .map_err(|_| "Cleaning ai-path failed")
}

pub async fn list() -> Result<(), &'static str> {
    for model in Models::load() {
        println!("Model: {}, resume:", model.name);
        let mut exist = true;
        for ressource in model.ressources.iter().filter(|res| !res.url.is_empty()) {
            let path = path::data().join(&model.name).join(&ressource.name);
            if let Ok(status) = tokio::fs::try_exists(path.as_path()).await {
                println!("\tRessource: {}, status: {}", ressource.name, status);
                exist = exist && status;
            }
        }
        if exist {
            println!("Model: {}, is complet\n", model.name);
        } else {
            println!("Model: {}, is not complet\n", model.name);
        }
    }

    Ok(())
}

pub async fn remove(
    model_name: String,
    ressource_name: Option<String>,
) -> Result<(), &'static str> {
    let models = Models::load();
    assert!(models.is_valid_model(&model_name));

    match ressource_name {
        None => tokio::fs::remove_dir_all(path::data().join(model_name))
            .await
            .map_err(|_| "Cannot delete the folder"),
        Some(ressource_name) => {
            assert!(models
                .clone()
                .into_iter()
                .find(|model| model.name == model_name)
                .unwrap()
                .is_valid_ressource(&ressource_name));

            let model = models
                .clone()
                .into_iter()
                .find(|model| model.name == model_name)
                .unwrap();

            let ressource = model
                .ressources
                .iter()
                .find(|ressource| ressource.name == ressource_name)
                .unwrap();
            if ressource.url.is_empty() {
                Ok(())
            } else {
                tokio::fs::remove_file(path::data().join(model_name).join(ressource_name))
                    .await
                    .map_err(|_| "Cannot delete the file")
            }
        }
    }
}

pub async fn add(model_name: String, ressource_name: Option<String>) -> Result<(), &'static str> {
    let models = Models::load();
    assert!(models.is_valid_model(&model_name));

    let model = models
        .into_iter()
        .find(|model| model.name == model_name)
        .unwrap();

    let multibar = indicatif::MultiProgress::new();

    if !path::data().exists() {
        std::fs::create_dir(path::data()).ok();
    }

    let mut set = task::JoinSet::new();

    match ressource_name {
        None => {
            for ressource in model.ressources {
                if !ressource.url.is_empty() {
                    let multibar = multibar.to_owned();
                    let name = model.name.clone();
                    if ressource.h2 {
                        set.spawn(async move { http2::download(multibar, &name, ressource).await });
                    } else {
                        set.spawn(async move { http1::download(multibar, &name, ressource).await });
                    }
                }
            }
        }
        Some(ressource_name) => {
            assert!(model.is_valid_ressource(&ressource_name));
            let ressource = model
                .ressources
                .iter()
                .find(|ressource| ressource.name == ressource_name)
                .unwrap();

            if !ressource.url.is_empty() {
                let multibar = multibar.to_owned();
                let name = model.name.clone();
                let ressource = (*ressource).clone();
                if ressource.h2 {
                    set.spawn(async move { http2::download(multibar, &name, ressource).await });
                } else {
                    set.spawn(async move { http1::download(multibar, &name, ressource).await });
                }
            }
        }
    };

    while let Some(task) = set.join_next().await {
        if let Err(error) = task {
            eprintln!("task on error: {error:?}");
        }
    }

    multibar.clear().map_err(|_| "Multibar clear failed")
}

pub async fn load() -> Result<(), &'static str> {
    let multibar = indicatif::MultiProgress::new();

    if !path::data().exists() {
        std::fs::create_dir(path::data()).ok();
    }

    let mut set = task::JoinSet::new();

    for model in Models::load() {
        let model_dir = path::data().join(model.name.clone());
        if model_dir.exists() {
            std::fs::remove_dir_all(&model_dir).ok().unwrap();
        }
        std::fs::create_dir(&model_dir).ok().unwrap();

        for ressource in model.ressources.iter().filter(|res| !res.url.is_empty()) {
            let multibar = multibar.to_owned();
            let name = model.name.clone();
            let ressource = ressource.clone();
            if ressource.h2 {
                set.spawn(async move { http2::download(multibar, &name, ressource).await });
            } else {
                set.spawn(async move { http1::download(multibar, &name, ressource).await });
            }
        }
    }

    while let Some(_res) = set.join_next().await {}

    multibar.clear().map_err(|_| "Multibar clear failed")
}
