#[derive(Debug, Clone, Copy, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum CommandType {
    Kill,
    Pause,
    Spawn,
    Resume,
    Process,
    // MoveTo,
    // ReConfig,
    // StreamTo
}
