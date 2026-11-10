//! Null backend that discards all rendered audio without ever touching an
//! audio output device.
//!
//! This is intended for offline rendering where audio is mixed separately and
//! must never be played through speakers. The mixer loop is kept alive so the
//! music and sfx renderers keep consuming their command channels.

use crate::mixer::Mixer;
use crate::Backend;
use anyhow::Result;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use super::{BackendSetup, BackendStreamInfo};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NullStreamInfo;

impl std::fmt::Display for NullStreamInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "null backend (no audio output device)")?;
        Ok(())
    }
}

pub struct NullBackend {
    thread: Option<JoinHandle<()>>,
}

impl NullBackend {
    pub fn new() -> Self {
        Self { thread: None }
    }
}

impl Default for NullBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl Backend for NullBackend {
    fn setup(&mut self, setup: BackendSetup) {
        let BackendSetup {
            mixer_cons,
            latency_rec,
        } = setup;
        drop(latency_rec);

        let mut mixer = Mixer::new(48_000, mixer_cons);
        self.thread = Some(
            thread::Builder::new()
                .name("sasa-null-mixer".to_owned())
                .spawn(move || {
                    const SAMPLE_RATE: u32 = 48_000;
                    let samples_per_tick = (SAMPLE_RATE / 100) as usize;
                    let mut buffer = vec![0.0f32; samples_per_tick * 2];
                    let tick = Duration::from_millis(10);
                    let mut next = Instant::now() + tick;
                    loop {
                        mixer.render_stereo(&mut buffer);
                        let now = Instant::now();
                        if next > now {
                            thread::sleep(next - now);
                            next += tick;
                        } else {
                            next = now + tick;
                        }
                    }
                })
                .expect("spawn null mixer thread"),
        );
    }

    fn start(&mut self) -> Result<()> {
        Ok(())
    }

    fn close(&mut self) -> Result<()> {
        Ok(())
    }

    fn consume_broken(&self) -> bool {
        false
    }

    fn stream_info(&mut self) -> BackendStreamInfo {
        BackendStreamInfo::Null(NullStreamInfo)
    }
}
