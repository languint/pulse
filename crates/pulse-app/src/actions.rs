use gpui::actions;

actions!(
    pulse,
    [
        ToggleFullscreen,
        Quit,
        ManageLibraryRoots,
        OpenSettings,
        OpenVisualizerSettings,
        ShowSpectrumVisualizer,
        ShowOscilloscopeVisualizer,
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
