use std::{fmt::Debug, time::Duration};

use rodio::{source::SineWave, OutputStream, OutputStreamBuilder, Sink, Source};

/// Cross-platform audio wrapper for CHIP-8 beeps
pub struct Speaker {
    /// This must be held as long as [`Self::sink`] lives
    _stream: OutputStream,
    /// The audio stream used for playing beeps
    sink: Sink,
    /// Whether or not the stream is currently playing
    is_playing: bool,
}

impl Debug for Speaker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Speaker")
            .field("is_playing", &self.is_playing)
            .finish()
    }
}

impl Speaker {
    /// Attempts to create a new sink with a preloaded 1000 second long sine wave (440Hz)
    pub fn new() -> Option<Self> {
        match OutputStreamBuilder::open_default_stream() {
            Ok(mut stream_handle) => {
                // dont log warnings on exit if in release mode
                if !cfg!(debug_assertions) {
                    stream_handle.log_on_drop(false);
                }

                let sink = Sink::connect_new(stream_handle.mixer());

                // creates a new 1000 second long A note and attaches it to the sink
                let source = SineWave::new(440.0)
                    .take_duration(Duration::from_secs(1000))
                    .amplify(0.20);
                sink.append(source);
                sink.pause();

                Some(Self {
                    _stream: stream_handle,
                    sink,
                    is_playing: false,
                })
            }
            Err(e) => {
                log::error!("audio error when opening stream: {:?}", e);
                None
            }
        }
    }

    /// Starts a beep's playing
    pub fn start(&mut self) {
        log::debug!("Starting sound");
        self.sink.play();
        self.is_playing = true;
    }

    /// Stops a beep's playing
    pub fn stop(&mut self) {
        log::debug!("Stopping sound");
        self.sink.pause();
        self.is_playing = false;
    }

    /// Whether or not the beep is currently playing
    pub fn is_playing(&self) -> bool {
        self.is_playing
    }
}
