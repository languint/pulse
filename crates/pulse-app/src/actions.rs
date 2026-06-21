use gpui::actions;

actions!(
    pulse,
    [
        ToggleFullscreen,
        Quit,
        ManageLibraryRoots,
        MediaPlayPause,
        MediaNextTrack,
        MediaPreviousTrack,
        ToggleCommandPalette,
        CommandPaletteSelectUp,
        CommandPaletteSelectDown,
        CommandPaletteConfirm,
        CommandPaletteDismiss,
        CommandPaletteTab
    ]
);
