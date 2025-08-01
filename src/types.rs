use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct SessionConfig {
    pub session_name: String,
    pub translation_languages: Vec<String>,
    pub master_language: String,
    pub viewer_language: String,
    pub enable_voiceover: bool,
    // pub remove_disfluencies: bool,
    // pub prefer_current_speaker: bool,
    // pub speaker_sensitivity: f32,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            session_name: "Rust SDK Session".to_string(),
            translation_languages: vec!["en-US".to_string(), "nb".to_string()],
            master_language: "de".to_string(),
            viewer_language: "en-US".to_string(),
            enable_voiceover: true,
            // remove_disfluencies: false,
            // prefer_current_speaker: false,
            // speaker_sensitivity: 0.5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketMessage {
    pub events: Vec<Event>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum Event {
    #[serde(rename = "partial")]
    Partial(PartialPayload),
    #[serde(rename = "transcript")]
    Transcript(TranscriptPayload),
    #[serde(rename = "translation")]
    Translation(TranslationPayload),
    #[serde(rename = "voiceover")]
    Voiceover(VoiceoverPayload),
    #[serde(rename = "voice")]
    Voice(VoicePayload),
    #[serde(rename = "start_of_stream")]
    StartOfStream(StartOfStreamPayload),
    #[serde(rename = "end_of_stream")]
    EndOfStream(EndOfStreamPayload),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialPayload {
    pub id: Option<String>,
    pub text: String,
    pub latency: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptPayload {
    pub sentence_id: String,
    pub text: String,
    pub latency: f64,
    pub speaker: String,
    pub created_at: i64,
    pub from_ms: f64,
    pub to_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationPayload {
    pub id: String,
    pub sentence_id: String,
    pub text: String,
    pub original: String,
    pub latency: f64,
    pub speaker: String,
    pub created_at: i64,
    pub from_ms: f64,
    pub to_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceoverPayload {
    pub id: String,
    pub text: String,
    pub original: String,
    pub latency: f64,
    pub speaker: String,
    pub created_at: i64,
    pub playback_uri: String,
    pub from_ms: f64,
    pub to_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoicePayload {
    pub id: String,
    pub sentence_id: String,
    pub text: String,
    pub latency: f64,
    pub speaker: String,
    pub created_at: i64,
    pub audio: String, // base64 encoded audio data
    pub seq: u32,
    pub from_ms: f64,
    pub to_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndOfStreamPayload {
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartOfStreamPayload {}

#[derive(Debug, Clone)]
pub struct AudioFrame {
    pub data: Vec<u8>,
    pub format: AudioFormat,
}

#[derive(Debug, Clone)]
pub enum AudioFormat {
    WebM,
    MpegTs,
    Raw,
}
