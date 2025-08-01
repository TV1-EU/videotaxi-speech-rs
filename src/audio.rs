use crate::{error::*, types::*};
use base64::{Engine as _, engine::general_purpose};

pub struct AudioUtils;

impl AudioUtils {
    /// Decode base64 audio data from voice events
    pub fn decode_voice_audio(voice_payload: &VoicePayload) -> Result<Vec<i16>> {
        let decoded_bytes = general_purpose::STANDARD
            .decode(&voice_payload.audio)
            .map_err(|_| SpeechApiError::InvalidAudioFormat)?;

        // Convert bytes to i16 samples (signed linear 16-bit PCM)
        let samples: Vec<i16> = decoded_bytes
            .chunks_exact(2)
            .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();

        Ok(samples)
    }

    /// Create audio frame from raw PCM data
    pub fn create_audio_frame(data: Vec<u8>, format: AudioFormat) -> AudioFrame {
        AudioFrame { data, format }
    }

    /// Convert i16 samples to bytes for transmission
    pub fn samples_to_bytes(samples: &[i16]) -> Vec<u8> {
        samples
            .iter()
            .flat_map(|&sample| sample.to_le_bytes().to_vec())
            .collect()
    }
}
