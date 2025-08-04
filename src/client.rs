use crate::{error::*, types::*};
use futures_util::{SinkExt, StreamExt};
use reqwest::Client as HttpClient;
use serde_json::{Value, json};
use tokio::sync::mpsc;
use tokio::time::{Duration, sleep};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};
use tracing::{debug, error, info, warn};

pub struct VideoTaxiClient {
    http_client: HttpClient,
    api_key: String,
    api_url: String,
}

impl VideoTaxiClient {
    pub fn new(api_key: String, api_url: Option<String>) -> Self {
        Self {
            http_client: HttpClient::new(),
            api_key,
            api_url: api_url.unwrap_or("https://service.video.taxi/graphiql".to_string()),
        }
    }

    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("VIDEOTAXI_TOKEN").map_err(|_| {
            SpeechApiError::InvalidConfig(
                "VIDEOTAXI_TOKEN environment variable not set".to_string(),
            )
        })?;
        let api_url = std::env::var("VIDEOTAXI_URL").ok();
        Ok(Self::new(api_key, api_url))
    }

    pub async fn create_session(&self, config: &SessionConfig) -> Result<VideoTaxiSession> {
        let create_query = json!({
            "query": format!(r#"
                mutation {{
                    createRealtimeSession(
                        name: "{}"
                        translationLanguages: {:?}
                    ) {{
                        id
                    }}
                }}
            "#, config.session_name, config.translation_languages)
        });

        info!("Creating new VIDEO.TAXI session...");
        let create_response = self
            .http_client
            .post(&self.api_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&create_query)
            .send()
            .await
            .map_err(|e| SpeechApiError::HttpError(e))?;

        let create_result: Value = create_response
            .json()
            .await
            .map_err(|e| SpeechApiError::HttpError(e))?;

        let session_id = create_result["data"]["createRealtimeSession"]["id"]
            .as_str()
            .ok_or(SpeechApiError::InvalidConfig(
                "Failed to get session ID from response".to_string(),
            ))?
            .to_string();

        info!("Created session with ID: {}", session_id);

        let details_query = json!({
            "query": format!(r#"
                query {{
                    realtimeSession(id: "{}") {{
                        id
                        masterSocketUrl(languageCode: "{}")
                        name
                        translationLanguages
                        viewerSocketUrl(enableVoiceover: {}, languageCode: "{}")
                        viewerWebUrl
                    }}
                }}
            "#, session_id, config.master_language, config.enable_voiceover, config.viewer_language)
        });

        let details_response = self
            .http_client
            .post(&self.api_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&details_query)
            .send()
            .await
            .map_err(|e| SpeechApiError::HttpError(e))?;

        let details_result: Value = details_response
            .json()
            .await
            .map_err(|e| SpeechApiError::HttpError(e))?;

        let session_details = &details_result["data"]["realtimeSession"];

        let master_socket_url = session_details["masterSocketUrl"]
            .as_str()
            .ok_or(SpeechApiError::InvalidConfig(
                "Failed to get master socket URL".to_string(),
            ))?
            .to_string();

        let viewer_socket_url = session_details["viewerSocketUrl"]
            .as_str()
            .ok_or(SpeechApiError::InvalidConfig(
                "Failed to get viewer socket URL".to_string(),
            ))?
            .to_string();

        info!("Session details retrieved successfully");

        Ok(VideoTaxiSession {
            session_id,
            master_socket_url,
            viewer_socket_url,
            config: config.clone(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct VideoTaxiSession {
    pub session_id: String,
    pub master_socket_url: String,
    pub viewer_socket_url: String,
    pub config: SessionConfig,
}

impl VideoTaxiSession {
    pub async fn connect_audio_sender(&self) -> Result<AudioSender> {
        info!("Connecting to master socket: {}", self.master_socket_url);
        let (ws_stream, _) = connect_async(&self.master_socket_url).await?;
        info!("Connected to master socket");

        Ok(AudioSender::new(ws_stream))
    }

    pub async fn connect_event_receiver(&self) -> Result<EventReceiver> {
        self.connect_event_receiver_with_retry(30, Duration::from_secs(2))
            .await
    }

    pub async fn connect_event_receiver_with_retry(
        &self,
        max_attempts: u32,
        retry_delay: Duration,
    ) -> Result<EventReceiver> {
        let mut attempts = 0;

        loop {
            attempts += 1;

            info!(
                "Attempting to connect to viewer socket (attempt {}/{}): {}",
                attempts, max_attempts, self.viewer_socket_url
            );

            match connect_async(&self.viewer_socket_url).await {
                Ok((ws_stream, _)) => {
                    info!(
                        "Successfully connected to viewer socket on attempt {}",
                        attempts
                    );
                    return Ok(EventReceiver::new(ws_stream));
                }
                Err(e) => {
                    if attempts >= max_attempts {
                        error!(
                            "Failed to connect to viewer socket after {} attempts. Last error: {}",
                            max_attempts, e
                        );
                        return Err(SpeechApiError::WebSocketError(e));
                    }

                    warn!(
                        "Failed to connect to viewer socket (attempt {}): {}. Retrying in {:?}...",
                        attempts, e, retry_delay
                    );

                    sleep(retry_delay).await;
                }
            }
        }
    }
}

pub struct AudioSender {
    ws_stream: WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>,
}

impl AudioSender {
    fn new(ws_stream: WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>) -> Self {
        Self { ws_stream }
    }

    pub async fn send_audio(&mut self, audio_data: Vec<u8>) -> Result<()> {
        self.ws_stream
            .send(Message::Binary(audio_data.into()))
            .await?;
        debug!("Sent audio chunk");
        Ok(())
    }

    pub async fn close(&mut self) -> Result<()> {
        self.ws_stream.send(Message::Close(None)).await?;
        info!("Audio sender connection closed");
        Ok(())
    }
}

pub struct EventReceiver {
    event_receiver: mpsc::UnboundedReceiver<Event>,
    _handle: tokio::task::JoinHandle<()>,
}

impl EventReceiver {
    fn new(ws_stream: WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        let handle = tokio::spawn(async move {
            let (mut _ws_sender, mut ws_receiver) = ws_stream.split();

            while let Some(msg) = ws_receiver.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        match serde_json::from_str::<WebSocketMessage>(&text) {
                            Ok(ws_message) => {
                                for event in ws_message.events {
                                    if let Err(_) = event_sender.send(event) {
                                        warn!("Event receiver channel closed");
                                        return;
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Failed to parse WebSocket message: {}", e);
                                error!("Raw message: {}", text);
                            }
                        }
                    }
                    Ok(Message::Close(_)) => {
                        info!("Viewer WebSocket connection closed");
                        break;
                    }
                    Ok(Message::Binary(data)) => {
                        debug!("Received binary data: {} bytes", data.len());
                    }
                    Err(e) => {
                        error!("WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        });

        Self {
            event_receiver,
            _handle: handle,
        }
    }

    pub async fn next_event(&mut self) -> Option<Event> {
        self.event_receiver.recv().await
    }
}
