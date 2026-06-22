use gpui::actions;

actions!(
    pulse,
    [
        ToggleFullscreen,
        Quit,
        ManageLibraryRoots,
        OpenSettings,
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
