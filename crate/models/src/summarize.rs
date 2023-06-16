use rust_bert::{
    pipelines::summarization::{SummarizationConfig, SummarizationModel},
    resources::LocalResource,
};
use shared::config::path;

pub fn get_local_ressources() -> (
    LocalResource,
    LocalResource,
    LocalResource,
    Option<LocalResource>,
) {
    let model_path = path::data().join(shared::config::path::model::DISTILBART_CNN_6_6);
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
        local_path: model_path.join("merges.txt"),
    };

    (config, vocab, model, Some(merges))
}

pub struct Summarize {
    model: SummarizationModel,
}

impl Default for Summarize {
    fn default() -> Self {
        let ressources = get_local_ressources();
        let merges = Box::new(ressources.3.unwrap());
        let config = SummarizationConfig {
            config_resource: Box::new(ressources.0),
            vocab_resource: Box::new(ressources.1),
            merges_resource: Some(merges),
            model_resource: Box::new(ressources.2),
            min_length: 60,
            max_length: None,
            ..Default::default()
        };

        Self {
            model: SummarizationModel::new(config)
                .map_err(|error| format!("{error}"))
                .ok()
                .unwrap(),
        }
    }
}

impl Summarize {
    pub fn prediction(&self, input: &str) -> String {
        let input: [&str; 1] = [input];
        let prediction = self.model.summarize(&input[..]);
        prediction.into_iter().collect::<String>()
    }

    pub fn try_prediction(&self, input: &str) -> Result<(), &'static str> {
        let input: [&str; 1] = [input];
        let prediction = self.model.summarize(&input[..]);
        let message = prediction.into_iter().collect::<String>();
        println!("{message:?}");
        Ok(())
    }
}
