use models::translation::Translation;

pub const TRANSLATION_INPUT: &str = "This sentence will be translated in multiple languages.";

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Input {
    prompt: String,
    source_lang: String,
    target_lang: String,
}

impl Default for Input {
    fn default() -> Self {
        Self {
            prompt: TRANSLATION_INPUT.to_string(),
            source_lang: "en".to_string(),
            target_lang: "fr".to_string(),
        }
    }
}

#[test]
fn test_summarize() {
    let model = Translation::default();
    let Input {
        prompt,
        source_lang,
        target_lang,
    } = &Input::default();
    let prediction = model.prediction(prompt, source_lang, target_lang);
    println!("{prediction:?}");
}
