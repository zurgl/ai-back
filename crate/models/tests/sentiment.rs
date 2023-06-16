use models::sentiment::Sentiment;

pub const SENTIMENT_INPUT: &str = "Probably my all-time favorite movie, a story of selflessness, sacrifice and dedication to a noble cause, but it's not preachy or boring.";

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Input {
    prompt: String,
}

impl Default for Input {
    fn default() -> Self {
        Self {
            prompt: SENTIMENT_INPUT.to_string(),
        }
    }
}

#[test]
fn test_sentiment() {
    let model = Sentiment::default();
    let Input { prompt } = &Input::default();
    let prediction = model.prediction(prompt);
    println!("{prediction:?}");
}
