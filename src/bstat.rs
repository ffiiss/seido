use std::*;

pub struct BeatSaberStatus {
    socket: tungstenite::WebSocket<tungstenite::stream::MaybeTlsStream<net::TcpStream>>,
}

#[derive(Debug, miniserde::Deserialize)]
pub struct NoteCut {
    #[serde(rename = "noteCutDirection")]
    pub note_cut_direction: String,
    #[serde(rename = "cutDistanceScore")]
    pub cut_distance_score: Option<usize>,
    #[serde(rename = "saberType")]
    pub saber_type: Option<String>,
    #[serde(rename = "cutPoint")]
    pub cut_point: [f64; 3],
    #[serde(rename = "cutNormal")]
    pub cut_normal: [f64; 3],
}

#[derive(miniserde::Deserialize)]
struct Event {
    event: String,
    #[serde(rename = "noteCut")]
    note_cut: Option<NoteCut>,
}

impl BeatSaberStatus {
    pub fn new() -> tungstenite::Result<Self> {
        let (socket, _) = tungstenite::connect("ws://localhost:6557/socket")?;
        Ok(BeatSaberStatus { socket: socket })
    }

    pub fn wait_note_cut(&mut self) -> Result<NoteCut, Box<dyn error::Error>> {
        loop {
            let data = self.socket.read()?.into_text()?;
            if data.is_empty() {
                continue;
            }
            let event: Event = miniserde::json::from_str(&data)?;
            if event.event == "noteCut" {
                return event.note_cut.ok_or("".into());
            }
        }
    }
}
