use rust_bert::{
    m2m_100::{M2M100SourceLanguages, M2M100TargetLanguages},
    pipelines::{
        common::ModelType,
        translation::{Language, TranslationConfig, TranslationModel},
    },
    resources::LocalResource,
};
use tch::Device;

use shared::config::path;

pub fn get_local_ressources() -> (
    LocalResource,
    LocalResource,
    LocalResource,
    Option<LocalResource>,
) {
    let model_path = path::data().join(shared::config::path::model::M2M100_418M);
    let config = LocalResource {
        local_path: model_path.join("config.json"),
    };
    let vocab = LocalResource {
        local_path: model_path.join("vocab.json"),
    };

    let model = LocalResource {
        local_path: model_path.join("rust_model.ot"),
    };

    let merges = LocalResource {
        local_path: model_path.join("sentencepiece.bpe.model"),
    };

    (config, vocab, model, Some(merges))
}

pub struct Translation {
    model: TranslationModel,
}

impl Default for Translation {
    fn default() -> Self {
        let source_languages = M2M100SourceLanguages::M2M100_418M;
        let target_languages = M2M100TargetLanguages::M2M100_418M;
        let ressources = get_local_ressources();
        let merges = ressources.3.unwrap();

        let config = TranslationConfig::new(
            ModelType::M2M100,
            ressources.2,
            ressources.0,
            ressources.1,
            Some(merges),
            source_languages,
            target_languages,
            Device::cuda_if_available(),
        );

        Self {
            model: TranslationModel::new(config)
                .map_err(|error| format!("{error}"))
                .ok()
                .unwrap(),
        }
    }
}

fn str_to_lang(value: &str) -> Language {
    match value {
        "en" => Language::English,
        "fr" => Language::French,
        "da" => Language::Danish,
        "de" => Language::German,
        "no" => Language::Norwegian,
        _ => Language::English,
    }
}

impl Translation {
    pub fn prediction(&self, input: &str, source_lang: &str, target_lang: &str) -> String {
        let input: [&str; 1] = [input];
        let prediction = self
            .model
            .translate(
                &input[..],
                str_to_lang(source_lang),
                str_to_lang(target_lang),
            )
            .map_err(|error| format!("{error}"))
            .ok()
            .unwrap();

        prediction.into_iter().collect::<String>()
    }

    pub fn try_prediction(
        &self,
        input: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<(), &'static str> {
        let input: [&str; 1] = [input];
        let prediction = self
            .model
            .translate(
                &input[..],
                str_to_lang(source_lang),
                str_to_lang(target_lang),
            )
            .map_err(|error| format!("{error}"))
            .ok()
            .unwrap();

        let message = prediction.into_iter().collect::<String>();
        println!("{message:?}");
        Ok(())
    }
}
