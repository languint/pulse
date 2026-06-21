use std::fs::File;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player};

#[derive(Debug, thiserror::Error)]
pub enum PlayerError {
    #[error("failed to open audio output: {0}")]
    Output(String),
    #[error("failed to open audio file: {path}")]
    OpenFile { path: PathBuf },
    #[error("unsupported or corrupt audio file: {path}")]
    Decode { path: PathBuf },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PlaybackState {
    #[default]
    Stopped,
    Playing,
    Paused,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrackInfo {
    pub path: PathBuf,
    pub duration_ms: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PlayerSnapshot {
    pub state: PlaybackState,
    pub queue: Vec<TrackInfo>,
    pub current_index: Option<usize>,
    pub position_ms: u64,
    pub duration_ms: u64,
}

enum Command {
    PlayQueue {
        tracks: Vec<TrackInfo>,
        index: usize,
    },
    TogglePause,
    Pause,
    Play,
    Next,
    Previous,
    Seek {
        position_ms: u64,
    },
    Stop,
}

pub struct PlayerHandle {
    command_tx: Sender<Command>,
    snapshot: Arc<Mutex<PlayerSnapshot>>,
    _thread: JoinHandle<()>,
}

impl PlayerHandle {
    /// # Errors
    ///
    /// Returns an error when the audio output device cannot be opened.
    pub fn new() -> Result<Self, PlayerError> {
        let snapshot = Arc::new(Mutex::new(PlayerSnapshot::default()));
        let (command_tx, command_rx) = mpsc::channel();
        let thread_snapshot = snapshot.clone();

        let thread = thread::Builder::new()
            .name("pulse-audio".into())
            .spawn(move || audio_thread_main(command_rx, thread_snapshot))
            .map_err(|error| PlayerError::Output(error.to_string()))?;

        Ok(Self {
            command_tx,
            snapshot,
            _thread: thread,
        })
    }

    #[must_use]
    pub fn snapshot(&self) -> PlayerSnapshot {
        self.snapshot
            .lock()
            .map(|snapshot| snapshot.clone())
            .unwrap_or_default()
    }

    pub fn play_queue(&self, tracks: Vec<TrackInfo>, index: usize) {
        let _ = self.command_tx.send(Command::PlayQueue { tracks, index });
    }

    pub fn toggle_pause(&self) {
        let _ = self.command_tx.send(Command::TogglePause);
    }

    pub fn pause(&self) {
        let _ = self.command_tx.send(Command::Pause);
    }

    pub fn play(&self) {
        let _ = self.command_tx.send(Command::Play);
    }

    pub fn next(&self) {
        let _ = self.command_tx.send(Command::Next);
    }

    pub fn previous(&self) {
        let _ = self.command_tx.send(Command::Previous);
    }

    pub fn seek(&self, position_ms: u64) {
        let _ = self.command_tx.send(Command::Seek { position_ms });
    }

    pub fn stop(&self) {
        let _ = self.command_tx.send(Command::Stop);
    }
}

const SEEK_BACK_TOLERANCE_MS: u64 = 100;

struct AudioThread {
    _device_sink: MixerDeviceSink,
    player: Player,
    queue: Vec<TrackInfo>,
    current_index: Option<usize>,
    state: PlaybackState,
    playhead_offset_ms: u64,
}

impl AudioThread {
    fn playback_position_ms(&mut self) -> u64 {
        if self.state == PlaybackState::Stopped {
            self.playhead_offset_ms = 0;
            return 0;
        }

        let player_pos = u64::try_from(self.player.get_pos().as_millis()).unwrap_or(0);

        if self.playhead_offset_ms > 0 {
            if player_pos.saturating_add(SEEK_BACK_TOLERANCE_MS) >= self.playhead_offset_ms {
                self.playhead_offset_ms = 0;
                player_pos
            } else {
                self.playhead_offset_ms.saturating_add(player_pos)
            }
        } else {
            player_pos
        }
    }

    fn publish_snapshot(&mut self, snapshot: &Arc<Mutex<PlayerSnapshot>>) {
        let duration_ms = self
            .current_index
            .and_then(|index| self.queue.get(index))
            .map_or(0, |track| u64::from(track.duration_ms));

        let position_ms = self.playback_position_ms();
        let Ok(mut shared) = snapshot.lock() else {
            return;
        };

        shared.state = self.state;
        shared.queue.clone_from(&self.queue);
        shared.current_index = self.current_index;
        shared.duration_ms = duration_ms;
        shared.position_ms = position_ms.min(duration_ms);
    }

    fn stop_playback(&mut self) {
        self.player.stop();
        self.player.clear();
        self.state = PlaybackState::Stopped;
        self.current_index = None;
        self.playhead_offset_ms = 0;
    }

    fn load_track(
        &mut self,
        index: usize,
        seek_ms: u64,
        autoplay: bool,
    ) -> Result<(), PlayerError> {
        let Some(track) = self.queue.get(index) else {
            return Ok(());
        };

        let volume = self.player.volume();
        self.player.set_volume(0.0);
        self.player.clear();

        let file = File::open(&track.path).map_err(|_| PlayerError::OpenFile {
            path: track.path.clone(),
        })?;
        let decoder = Decoder::try_from(file).map_err(|_| PlayerError::Decode {
            path: track.path.clone(),
        })?;

        let seek_ms = seek_ms.min(u64::from(track.duration_ms));

        self.player.append(decoder);
        self.player.pause();

        if seek_ms > 0
            && let Err(error) = self.player.try_seek(Duration::from_millis(seek_ms))
        {
            tracing::warn!(
                %error,
                seek_ms,
                path = ?track.path,
                "player seek failed after reload"
            );
        }

        self.playhead_offset_ms = seek_ms;
        self.player.set_volume(volume);

        if autoplay {
            self.player.play();
            self.state = PlaybackState::Playing;
        } else {
            self.player.pause();
            self.state = PlaybackState::Paused;
        }

        self.current_index = Some(index);
        Ok(())
    }

    fn play_queue(&mut self, tracks: Vec<TrackInfo>, index: usize) {
        self.queue = tracks;
        if self.queue.is_empty() {
            self.stop_playback();
            return;
        }

        let index = index.min(self.queue.len().saturating_sub(1));
        if let Err(error) = self.load_track(index, 0, true) {
            tracing::error!(%error, "failed to start playback");
            self.stop_playback();
        }
    }

    fn toggle_pause(&mut self) {
        if self.current_index.is_none() {
            return;
        }

        if self.player.is_paused() {
            self.player.play();
            self.state = PlaybackState::Playing;
        } else {
            self.player.pause();
            self.state = PlaybackState::Paused;
        }
    }

    fn pause(&mut self) {
        if self.current_index.is_none() || self.state != PlaybackState::Playing {
            return;
        }

        self.player.pause();
        self.state = PlaybackState::Paused;
    }

    fn play(&mut self) {
        if self.current_index.is_none() || self.state != PlaybackState::Paused {
            return;
        }

        self.player.play();
        self.state = PlaybackState::Playing;
    }

    fn seek(&mut self, position_ms: u64) {
        let Some(index) = self.current_index else {
            return;
        };

        let Some(track) = self.queue.get(index) else {
            return;
        };

        let position_ms = position_ms.min(u64::from(track.duration_ms));

        if let Err(error) = self.load_track(index, position_ms, false) {
            tracing::error!(%error, "failed to seek");
        }
    }

    fn play_relative(&mut self, delta: i32) {
        let Some(current) = self.current_index else {
            return;
        };

        let step = usize::try_from(delta.unsigned_abs()).unwrap_or(usize::MAX);
        let next = if delta.is_negative() {
            current.checked_sub(step)
        } else {
            current.checked_add(step)
        };

        let Some(next) = next.filter(|index| *index < self.queue.len()) else {
            self.stop_playback();
            return;
        };

        if let Err(error) = self.load_track(next, 0, true) {
            tracing::error!(%error, "failed to change track");
            self.stop_playback();
        }
    }

    fn handle_command(&mut self, command: Command) {
        match command {
            Command::PlayQueue { tracks, index } => self.play_queue(tracks, index),
            Command::TogglePause => self.toggle_pause(),
            Command::Pause => self.pause(),
            Command::Play => self.play(),
            Command::Next => self.play_relative(1),
            Command::Previous => {
                if self.player.get_pos() > Duration::from_secs(3) {
                    let Some(index) = self.current_index else {
                        return;
                    };
                    let autoplay = self.state == PlaybackState::Playing;
                    if let Err(error) = self.load_track(index, 0, autoplay) {
                        tracing::error!(%error, "failed to restart track");
                    }
                } else {
                    self.play_relative(-1);
                }
            }
            Command::Seek { position_ms } => self.seek(position_ms),
            Command::Stop => self.stop_playback(),
        }
    }

    fn poll_track_end(&mut self) {
        if self.state != PlaybackState::Playing || !self.player.empty() {
            return;
        }

        self.play_relative(1);
    }
}

#[allow(clippy::needless_pass_by_value)]
fn audio_thread_main(command_rx: Receiver<Command>, snapshot: Arc<Mutex<PlayerSnapshot>>) {
    let device_sink = match DeviceSinkBuilder::open_default_sink() {
        Ok(sink) => sink,
        Err(error) => {
            tracing::error!(%error, "failed to open default audio output device");
            return;
        }
    };

    let player = Player::connect_new(device_sink.mixer());
    let mut audio = AudioThread {
        _device_sink: device_sink,
        player,
        queue: Vec::new(),
        current_index: None,
        state: PlaybackState::Stopped,
        playhead_offset_ms: 0,
    };

    loop {
        let mut pending_seek = None;

        while let Ok(command) = command_rx.try_recv() {
            match command {
                Command::Seek { position_ms } => pending_seek = Some(position_ms),
                command => {
                    if let Some(position_ms) = pending_seek.take() {
                        audio.seek(position_ms);
                        audio.publish_snapshot(&snapshot);
                    }
                    audio.handle_command(command);
                    audio.publish_snapshot(&snapshot);
                }
            }
        }

        if let Some(position_ms) = pending_seek.take() {
            audio.seek(position_ms);
            audio.publish_snapshot(&snapshot);
        }

        audio.poll_track_end();
        if audio.state != PlaybackState::Stopped {
            audio.publish_snapshot(&snapshot);
        }

        thread::sleep(Duration::from_millis(50));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_defaults_to_stopped() {
        let player = PlayerHandle::new().expect("audio output");
        let snapshot = player.snapshot();
        assert_eq!(snapshot.state, PlaybackState::Stopped);
        assert!(snapshot.queue.is_empty());
    }
}
