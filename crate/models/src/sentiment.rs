use rust_bert::{
    pipelines::{
        common::ModelType as BertModelType, sentiment::SentimentModel,
        sequence_classification::SequenceClassificationConfig,
    },
    resources::LocalResource,
};
use shared::config::path;

pub fn get_local_ressources() -> (
    LocalResource,
    LocalResource,
    LocalResource,
    Option<LocalResource>,
) {
    let model_path = path::data().join(shared::config::path::model::DISTILBERT_SST2);
    let config = LocalResource {
        local_path: model_path.join("config.json"),
    };
    let vocab = LocalResource {
        local_path: model_path.join("vocab.txt"),
    };
    let model = LocalResource {
        local_path: model_path.join("model.safetensors"),
    };

    (config, vocab, model, None)
}

pub struct Sentiment {
    model: SentimentModel,
}

impl Default for Sentiment {
    fn default() -> Self {
        let ressources = get_local_ressources();
        let config = SequenceClassificationConfig::new(
            BertModelType::DistilBert,
            ressources.2,
            ressources.0,
            ressources.1,
            None,
            true,
            None,
            None,
        );

        Self {
            model: SentimentModel::new(config)
                .map_err(|error| format!("{error}"))
                .ok()
                .unwrap(),
        }
    }
}

impl Sentiment {
    pub fn prediction(&self, input: &str) -> String {
        let input: [&str; 1] = [input];
        let prediction = self.model.predict(&input[..]);
        let message = format!("{prediction:?}");
        message
    }

    pub fn try_prediction(&self, input: &str) -> Result<(), &'static str> {
        let input: [&str; 1] = [input];
        let prediction = self.model.predict(&input[..]);
        let message = format!("{prediction:?}");
        println!("{message:?}");
        Ok(())
    }
}
