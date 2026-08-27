use std::sync::Arc;
use parking_lot::Mutex;
use rodio::{Sink, OutputStream, OutputStreamHandle, Source};
use std::io::Cursor;
use reqwest::Client;
use std::time::Duration;
use hound::WavSpec;
use serde::{Serialize, Deserialize};

pub struct RadioPlayer {
    sink: Arc<Mutex<Option<Sink>>>,
    stream_handle: OutputStreamHandle,
    current_url: Arc<Mutex<Option<String>>>,
    is_playing: Arc<Mutex<bool>>,
    volume: Arc<Mutex<f32>>,
    eq_gains: Arc<Mutex<[f32; 10]>>,
    metadata: Arc<Mutex<Option<Metadata>>>,
    recorder: Arc<Mutex<Option<Recorder>>>,
}

struct Recorder {
    writer: hound::WavWriter<std::io::BufWriter<std::fs::File>>,
    spec: WavSpec,
    path: std::path::PathBuf,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub artwork_url: String,
}

impl RadioPlayer {
    pub fn new() -> Self {
        let (_stream, stream_handle) = OutputStream::try_default().expect("Failed to create audio output stream");

        Self {
            sink: Arc::new(Mutex::new(None)),
            stream_handle,
            current_url: Arc::new(Mutex::new(None)),
            is_playing: Arc::new(Mutex::new(false)),
            volume: Arc::new(Mutex::new(0.7)),
            eq_gains: Arc::new(Mutex::new([0.0; 10])),
            metadata: Arc::new(Mutex::new(None)),
            recorder: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn play(self: Arc<Self>, url: String) -> Result<(), String> {
        self.stop().await?;

        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("WinRadio/0.1")
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        let response = client.get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to connect to stream: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()));
        }

        let icy_metaint = response.headers()
            .get("icy-metaint")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<usize>().ok());

        let stream = response.bytes_stream();

        let source = HttpStreamSource::new(stream, icy_metaint, self.clone());

        let sink = Sink::try_new(&self.stream_handle).map_err(|e| format!("Failed to create sink: {}", e))?;
        sink.set_volume(*self.volume.lock());

        {
            let mut current_sink = self.sink.lock();
            *current_sink = Some(sink);
        }

        {
            let mut current_url = self.current_url.lock();
            *current_url = Some(url.clone());
        }

        {
            let mut playing = self.is_playing.lock();
            *playing = true;
        }

        let player_sink = self.sink.clone();
        let player_playing = self.is_playing.clone();
        let player_eq = self.eq_gains.clone();

        tokio::task::spawn_blocking(move || {
            let sink_guard = player_sink.lock();
            if let Some(sink) = sink_guard.as_ref() {
                let source = EqSource::new(source, player_eq.clone());
                sink.append(source);
                sink.sleep_until_end();

                let mut playing = player_playing.lock();
                *playing = false;
            }
        });

        Ok(())
    }

    pub async fn stop(&self) -> Result<(), String> {
        let mut sink_guard = self.sink.lock();
        if let Some(sink) = sink_guard.take() {
            sink.stop();
        }

        let mut playing = self.is_playing.lock();
        *playing = false;

        let mut current_url = self.current_url.lock();
        *current_url = None;

        let mut recorder_guard = self.recorder.lock();
        if let Some(recorder) = recorder_guard.take() {
            recorder.writer.finalize().map_err(|e| format!("Failed to finalize recording: {}", e))?;
        }

        Ok(())
    }

    pub async fn set_volume(&self, volume: f32) -> Result<(), String> {
        let vol = volume.clamp(0.0, 1.0);
        {
            let mut v = self.volume.lock();
            *v = vol;
        }
        let sink_guard = self.sink.lock();
        if let Some(sink) = sink_guard.as_ref() {
            sink.set_volume(vol);
        }
        Ok(())
    }

    pub fn get_volume(&self) -> f32 {
        *self.volume.lock()
    }

    pub fn is_playing(&self) -> bool {
        *self.is_playing.lock()
    }

    pub async fn current_url(&self) -> Option<String> {
        self.current_url.lock().clone()
    }

    pub async fn set_eq_band(&self, band: usize, gain_db: f32) -> Result<(), String> {
        if band >= 10 {
            return Err("Invalid band".to_string());
        }
        let mut eq = self.eq_gains.lock();
        eq[band] = gain_db.clamp(-12.0, 12.0);
        Ok(())
    }

    pub fn get_eq(&self) -> [f32; 10] {
        *self.eq_gains.lock()
    }

    pub async fn reset_eq(&self) -> Result<(), String> {
        let mut eq = self.eq_gains.lock();
        *eq = [0.0; 10];
        Ok(())
    }

    pub async fn start_recording(&self, filename: String) -> Result<String, String> {
        let spec = WavSpec {
            channels: 2,
            sample_rate: 44100,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let path = std::path::PathBuf::from(&filename);
        let writer = hound::WavWriter::create(&path, spec)
            .map_err(|e| format!("Failed to create WAV writer: {}", e))?;

        let recorder = Recorder { writer, spec, path: path.clone() };

        {
            let mut rec = self.recorder.lock();
            *rec = Some(recorder);
        }

        Ok(filename)
    }

    pub async fn stop_recording(&self) -> Result<std::path::PathBuf, String> {
        let mut rec = self.recorder.lock();
        if let Some(recorder) = rec.take() {
            recorder.writer.finalize().map_err(|e| format!("Failed to finalize recording: {}", e))?;
            Ok(recorder.path)
        } else {
            Err("Not recording".to_string())
        }
    }

    pub fn get_metadata(&self) -> Option<Metadata> {
        self.metadata.lock().clone()
    }

    fn update_metadata(&self, metadata: Metadata) {
        *self.metadata.lock() = Some(metadata);
    }
}

use futures::StreamExt;
use std::pin::Pin;

struct HttpStreamSource {
    stream: Pin<Box<dyn futures::Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Send>>,
    icy_metaint: Option<usize>,
    buffer: Vec<u8>,
    metadata_buffer: Vec<u8>,
    bytes_since_metadata: usize,
    in_metadata: bool,
    metadata_length: usize,
    player: Option<Arc<RadioPlayer>>,
}

impl HttpStreamSource {
    fn new(
        stream: impl futures::Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Send + 'static,
        icy_metaint: Option<usize>,
        player: Arc<RadioPlayer>,
    ) -> Self {
        Self {
            stream: Box::pin(stream),
            icy_metaint,
            buffer: Vec::with_capacity(8192),
            metadata_buffer: Vec::new(),
            bytes_since_metadata: 0,
            in_metadata: false,
            metadata_length: 0,
            player: Some(player),
        }
    }
}

impl Source for HttpStreamSource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        2
    }

    fn sample_rate(&self) -> u32 {
        44100
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

impl Iterator for HttpStreamSource {
    type Item = i16;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if !self.buffer.is_empty() {
                return Some(self.buffer.remove(0) as i16);
            }

            // This is a blocking call in an iterator, which is not ideal
            // but works for our use case. In production, you'd want async.
            let rt = tokio::runtime::Handle::current();
            let chunk = rt.block_on(async {
                self.stream.next().await
            });

            match chunk {
                Some(Ok(bytes)) => {
                    let data = bytes.as_ref();

                    if let Some(metaint) = self.icy_metaint {
                        for &byte in data {
                            if self.in_metadata {
                                self.metadata_buffer.push(byte);
                                if self.metadata_buffer.len() >= self.metadata_length {
                                    self.parse_icy_metadata();
                                    self.in_metadata = false;
                                    self.metadata_buffer.clear();
                                }
                            } else {
                                self.bytes_since_metadata += 1;
                                if self.bytes_since_metadata >= metaint {
                                    self.in_metadata = true;
                                    self.metadata_length = byte as usize * 16;
                                    self.bytes_since_metadata = 0;
                                    if self.metadata_length == 0 {
                                        self.in_metadata = false;
                                    }
                                } else {
                                    self.buffer.push(byte);
                                }
                            }
                        }
                    } else {
                        self.buffer.extend_from_slice(data);
                    }

                    if !self.buffer.is_empty() {
                        return Some(self.buffer.remove(0) as i16);
                    }
                }
                Some(Err(_)) => return None,
                None => return None,
            }
        }
    }
}

impl HttpStreamSource {
    fn parse_icy_metadata(&mut self) {
        if let Ok(metadata_str) = String::from_utf8(self.metadata_buffer.clone()) {
            let mut title = String::new();
            let mut artist = String::new();

            for part in metadata_str.split(';') {
                if let Some((key, value)) = part.split_once('=') {
                    let value = value.trim_matches('\'').trim_matches('"');
                    match key.trim() {
                        "StreamTitle" => {
                            if let Some((a, t)) = value.split_once(" - ") {
                                artist = a.trim().to_string();
                                title = t.trim().to_string();
                            } else {
                                title = value.to_string();
                            }
                        }
                        "StreamUrl" => { /* ignore */ }
                        _ => {}
                    }
                }
            }

            if !title.is_empty() || !artist.is_empty() {
                if let Some(player) = &self.player {
                    player.update_metadata(Metadata {
                        title,
                        artist,
                        album: String::new(),
                        artwork_url: String::new(),
                    });
                }
            }
        }
    }
}

struct EqSource<S: Source<Item = i16>> {
    source: S,
    eq_gains: Arc<Mutex<[f32; 10]>>,
    filters: [BiquadFilter; 10],
}

impl<S: Source<Item = i16>> EqSource<S> {
    fn new(source: S, eq_gains: Arc<Mutex<[f32; 10]>>) -> Self {
        let frequencies = [31.0, 62.0, 125.0, 250.0, 500.0, 1000.0, 2000.0, 4000.0, 8000.0, 16000.0];
        let filters = frequencies.map(|f| BiquadFilter::new(f, 44100.0));

        Self {
            source,
            eq_gains,
            filters,
        }
    }
}

impl<S: Source<Item = i16>> Source for EqSource<S> {
    fn current_frame_len(&self) -> Option<usize> {
        self.source.current_frame_len()
    }

    fn channels(&self) -> u16 {
        self.source.channels()
    }

    fn sample_rate(&self) -> u32 {
        self.source.sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        self.source.total_duration()
    }
}

impl<S: Source<Item = i16>> Iterator for EqSource<S> {
    type Item = i16;

    fn next(&mut self) -> Option<Self::Item> {
        let sample = self.source.next()?;
        let gains = self.eq_gains.lock();
        let mut processed = sample as f32 / 32768.0;

        for (i, filter) in self.filters.iter_mut().enumerate() {
            filter.set_gain(gains[i]);
            processed = filter.process(processed);
        }

        Some((processed.clamp(-1.0, 1.0) * 32767.0) as i16)
    }
}

struct BiquadFilter {
    a0: f32, a1: f32, a2: f32,
    b1: f32, b2: f32,
    x1: f32, x2: f32,
    y1: f32, y2: f32,
}

impl BiquadFilter {
    fn new(freq: f32, sample_rate: f32) -> Self {
        let mut filter = Self {
            a0: 1.0, a1: 0.0, a2: 0.0,
            b1: 0.0, b2: 0.0,
            x1: 0.0, x2: 0.0,
            y1: 0.0, y2: 0.0,
        };
        filter.set_frequency(freq, sample_rate);
        filter.set_gain(0.0);
        filter
    }

    fn set_frequency(&mut self, freq: f32, sample_rate: f32) {
        let w0 = 2.0 * std::f32::consts::PI * freq / sample_rate;
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();
        let alpha = sin_w0 / (2.0 * 1.0); // Q = 1.0

        self.b1 = -2.0 * cos_w0;
        self.b2 = 1.0 - alpha;
    }

    fn set_gain(&mut self, gain_db: f32) {
        let a = 10.0_f32.powf(gain_db / 40.0);
        let w0 = 2.0 * std::f32::consts::PI * self.get_frequency() / 44100.0;
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();
        let alpha = sin_w0 / 2.0;

        let a0 = 1.0 + alpha * a;
        self.a0 = (1.0 - cos_w0) * a / a0;
        self.a1 = 2.0 * (1.0 - cos_w0) * a / a0;
        self.a2 = self.a0;
    }

    fn get_frequency(&self) -> f32 {
        1000.0 // placeholder
    }

    fn process(&mut self, input: f32) -> f32 {
        let y = self.a0 * input + self.a1 * self.x1 + self.a2 * self.x2
            - self.b1 * self.y1 - self.b2 * self.y2;

        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = y;

        y
    }
}