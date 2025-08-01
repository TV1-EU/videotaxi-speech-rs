use tokio::io::AsyncReadExt;
use videotaxi_speech_rs::{
    AudioSender, EventReceiver, SessionConfig, VideoTaxiClient, VideoTaxiSession,
};

/// Example usage:
/// ffmpeg -f alsa -i default -ac 2 -f adts -  | VIDEOTAXI_TOKEN=<insert_token> cargo run
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // can set two env vars:
    // - VIDEOTAXI_TOKEN (mandatory)
    // - VIDEOTAXI_URL (optional)
    let client = VideoTaxiClient::from_env()?;

    // Configure session
    let config = SessionConfig {
        session_name: "My Rust Session".to_string(),
        translation_languages: vec!["en-US".to_string(), "nb".to_string()],
        master_language: "de".to_string(),
        viewer_language: "en-US".to_string(),
        enable_voiceover: true,
    };

    // Create session: provisions the required VIDEO.TAXI Resources and fetches the session details
    let session: VideoTaxiSession = client.create_session(&config).await?;

    // Connect audio sender and event receiver
    let mut audio_sender: AudioSender = session.connect_audio_sender().await?;
    let mut event_receiver: EventReceiver = session.connect_event_receiver().await?;

    // Spawn task to send audio from stdin (we pipe encoded frames into stdin)
    let audio_task = tokio::spawn(async move {
        let mut stdin = tokio::io::stdin();
        let mut buffer = vec![0u8; 4096];

        loop {
            match stdin.read(&mut buffer).await {
                Ok(0) => break, // EOF
                Ok(n) => {
                    if let Err(e) = audio_sender.send_audio(buffer[..n].to_vec()).await {
                        eprintln!("Error sending audio: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Error reading stdin: {}", e);
                    break;
                }
            }
        }
    });

    // Handle events sent from VIDEO.TAXI, we could filter on the EventType
    let event_task = tokio::spawn(async move {
        while let Some(event) = event_receiver.next_event().await {
            println!("Received event: {:#?}", event);
        }
    });

    // Wait for tasks (forever)
    tokio::select! {
        _ = audio_task => println!("Audio task completed"),
        _ = event_task => println!("Event task completed"),
        _ = tokio::signal::ctrl_c() => println!("Shutting down..."),
    }

    Ok(())
}
