use std::cell::RefCell;
use std::ffi::c_void;
use std::sync::Mutex;
use std::time::Duration;

use gpui::{App, Global, Window};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use souvlaki::{
    MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, MediaPosition, PlatformConfig,
};

use crate::player::PulsePlayer;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaCommand {
    TogglePlayback,
    Play,
    Pause,
    Next,
    Previous,
}

static PENDING_COMMANDS: Mutex<Vec<MediaCommand>> = Mutex::new(Vec::new());

pub fn enqueue(command: MediaCommand) {
    let Ok(mut pending) = PENDING_COMMANDS.lock() else {
        return;
    };
    pending.push(command);
}

pub struct PulseMediaControls {
    state: RefCell<Option<MediaControlsState>>,
}

struct MediaControlsState {
    controls: MediaControls,
    event_rx: std::sync::mpsc::Receiver<MediaControlEvent>,
    last_sync: PlaybackSyncState,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct PlaybackSyncState {
    track_index: Option<usize>,
    playing: bool,
    position_seconds: u64,
}

impl Global for PulseMediaControls {}

pub fn init(window: &Window, cx: &mut App) {
    let hwnd = window_hwnd(window);
    let config = PlatformConfig {
        dbus_name: "pulse",
        display_name: "Pulse",
        hwnd,
    };

    let (tx, rx) = std::sync::mpsc::channel();

    let mut controls = match MediaControls::new(config) {
        Ok(controls) => Some(controls),
        Err(error) => {
            tracing::warn!(%error, "failed to create OS media controls");
            None
        }
    };

    let event_rx = controls.as_mut().and_then(|controls| {
        match controls.attach(move |event| {
            let _ = tx.send(event);
        }) {
            Ok(()) => Some(rx),
            Err(error) => {
                tracing::warn!(%error, "failed to attach OS media controls");
                None
            }
        }
    });

    let souvlaki_active = event_rx.is_some();

    #[cfg(windows)]
    if !souvlaki_active && let Some(hwnd) = hwnd {
        windows_keys::install(hwnd);
    }

    let state = controls
        .zip(event_rx)
        .map(|(controls, event_rx)| MediaControlsState {
            controls,
            event_rx,
            last_sync: PlaybackSyncState::default(),
        });

    cx.set_global(PulseMediaControls {
        state: RefCell::new(state),
    });
}

pub fn poll(cx: &mut App) {
    drain_souvlaki_events(cx);
    drain_pending_commands(cx);
    sync_playback(cx);
}

fn drain_souvlaki_events(cx: &App) {
    let global = cx.global::<PulseMediaControls>();
    let mut state = global.state.borrow_mut();
    let Some(state) = state.as_mut() else {
        return;
    };

    while let Ok(event) = state.event_rx.try_recv() {
        match event {
            MediaControlEvent::Play => enqueue(MediaCommand::Play),
            MediaControlEvent::Pause => enqueue(MediaCommand::Pause),
            MediaControlEvent::Toggle => enqueue(MediaCommand::TogglePlayback),
            MediaControlEvent::Next => enqueue(MediaCommand::Next),
            MediaControlEvent::Previous => enqueue(MediaCommand::Previous),
            _ => {}
        }
    }
}

fn drain_pending_commands(cx: &mut App) {
    let commands = {
        let Ok(mut pending) = PENDING_COMMANDS.lock() else {
            return;
        };
        std::mem::take(&mut *pending)
    };

    for command in commands {
        dispatch(command, cx);
    }
}

pub fn dispatch(command: MediaCommand, cx: &mut App) {
    match command {
        MediaCommand::TogglePlayback => PulsePlayer::toggle_pause(cx),
        MediaCommand::Play => PulsePlayer::play(cx),
        MediaCommand::Pause => PulsePlayer::pause(cx),
        MediaCommand::Next => PulsePlayer::next(cx),
        MediaCommand::Previous => PulsePlayer::previous(cx),
    }
}

fn sync_playback(cx: &App) {
    let global = cx.global::<PulseMediaControls>();
    let mut state = global.state.borrow_mut();
    let Some(state) = state.as_mut() else {
        return;
    };

    let snapshot = PulsePlayer::snapshot(cx);
    let sync = PlaybackSyncState {
        track_index: snapshot.current_index,
        playing: PulsePlayer::is_playing(cx),
        position_seconds: snapshot.position_ms / 1000,
    };

    if sync == state.last_sync {
        return;
    }

    let playing = sync.playing;
    state.last_sync = sync;

    let progress = MediaPosition(Duration::from_millis(snapshot.position_ms));
    let playback = if snapshot.current_index.is_none() {
        MediaPlayback::Stopped
    } else if playing {
        MediaPlayback::Playing {
            progress: Some(progress),
        }
    } else {
        MediaPlayback::Paused {
            progress: Some(progress),
        }
    };

    if let Err(error) = state.controls.set_playback(playback) {
        tracing::debug!(%error, "failed to sync media playback state");
    }

    if snapshot.current_index.is_some() {
        let title = PulsePlayer::current_track_title(cx).map(|value| value.to_string());
        let artist = PulsePlayer::current_track_subtitle(cx).map(|value| value.to_string());
        let metadata = MediaMetadata {
            title: title.as_deref(),
            artist: artist.as_deref(),
            album: None,
            cover_url: None,
            duration: None,
        };

        if let Err(error) = state.controls.set_metadata(metadata) {
            tracing::debug!(%error, "failed to sync media metadata");
        }
    }
}

fn window_hwnd(window: &Window) -> Option<*mut c_void> {
    #[cfg(windows)]
    {
        let handle = HasWindowHandle::window_handle(window).ok()?;
        match handle.as_raw() {
            RawWindowHandle::Win32(handle) => {
                #[allow(clippy::as_conversions)]
                {
                    Some(handle.hwnd.get() as *mut c_void)
                }
            }
            _ => None,
        }
    }

    #[cfg(not(windows))]
    {
        let _ = window;
        None
    }
}

#[cfg(windows)]
mod windows_keys {
    use std::sync::OnceLock;

    use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
    use windows::Win32::UI::Shell::{DefSubclassProc, SetWindowSubclass};
    use windows::Win32::UI::WindowsAndMessaging::WM_APPCOMMAND;

    use super::{MediaCommand, enqueue};

    static INSTALLED: OnceLock<()> = OnceLock::new();

    pub fn install(hwnd: *mut core::ffi::c_void) {
        INSTALLED.get_or_init(|| unsafe {
            let hwnd = HWND(hwnd);
            let _ = SetWindowSubclass(hwnd, Some(subclass_proc), 0, 0);
        });
    }

    unsafe extern "system" fn subclass_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
        _id: usize,
        _data: usize,
    ) -> LRESULT {
        let _ = lparam;

        if msg == WM_APPCOMMAND {
            let command = (wparam.0 >> 16) & 0xFFF;
            match command {
                14 => enqueue(MediaCommand::TogglePlayback),
                11 => enqueue(MediaCommand::Previous),
                12 => enqueue(MediaCommand::Next),
                _ => return unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) },
            }
            return LRESULT(1);
        }

        unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) }
    }
}
