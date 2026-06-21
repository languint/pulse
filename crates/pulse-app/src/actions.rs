use gpui::actions;

actions!(
    pulse,
    [
        ToggleFullscreen,
        Quit,
        ManageLibraryRoots,
        MediaPlayPause,
        MediaNextTrack,
        MediaPreviousTrack
    ]
);
