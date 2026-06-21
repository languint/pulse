use std::path::{Path, PathBuf};

use gpui::{
    App, AppContext, Context, InteractiveElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Window, div, px,
};
use gpui_component::{
    ActiveTheme, IconName, WindowExt as _,
    button::{Button, ButtonVariants as _},
    dialog::{DialogClose, DialogFooter, DialogTitle},
    h_flex, v_flex,
};
use pulse_library::LibraryConfig;

use crate::library::PulseLibrary;

#[derive(Clone, Copy, Debug)]
enum RootRowKind {
    SystemMusic,
    Custom(usize),
}

#[derive(Clone, Debug)]
struct RootRow {
    kind: RootRowKind,
    path: PathBuf,
}

pub struct LibraryRootsEditor {
    config: LibraryConfig,
}

impl LibraryRootsEditor {
    const fn new(config: LibraryConfig) -> Self {
        Self { config }
    }

    #[must_use]
    pub const fn config(&self) -> &LibraryConfig {
        &self.config
    }

    fn build_rows(config: &LibraryConfig) -> Vec<RootRow> {
        let mut rows = Vec::new();

        if config.include_xdg_music_dir
            && let Some(path) = dirs::audio_dir()
        {
            rows.push(RootRow {
                kind: RootRowKind::SystemMusic,
                path,
            });
        }

        for (index, path) in config.extra_paths.iter().enumerate() {
            rows.push(RootRow {
                kind: RootRowKind::Custom(index),
                path: path.clone(),
            });
        }

        rows
    }

    fn delete_row(&mut self, row_ix: usize, cx: &mut Context<Self>) {
        let Some(kind) = Self::build_rows(&self.config)
            .get(row_ix)
            .map(|row| row.kind)
        else {
            return;
        };

        match kind {
            RootRowKind::SystemMusic => self.config.include_xdg_music_dir = false,
            RootRowKind::Custom(index) => {
                if index < self.config.extra_paths.len() {
                    self.config.extra_paths.remove(index);
                }
            }
        }

        cx.notify();
    }

    fn add_path(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        if path.as_os_str().is_empty() {
            return;
        }

        if self
            .config
            .extra_paths
            .iter()
            .any(|existing| paths_equal(existing, &path))
        {
            return;
        }

        if self.config.include_xdg_music_dir
            && dirs::audio_dir().is_some_and(|music_dir| paths_equal(&music_dir, &path))
        {
            return;
        }

        self.config.extra_paths.push(path);
        cx.notify();
    }

    fn pick_folder(window: &Window, cx: &Context<Self>) {
        let dialog = rfd::AsyncFileDialog::new()
            .set_title("Select music folder")
            .set_parent(window)
            .pick_folder();

        cx.spawn(async move |this, cx| {
            let Some(handle) = dialog.await else {
                return;
            };

            let path = handle.path().to_path_buf();
            this.update(cx, |editor, cx| {
                editor.add_path(path, cx);
            })
            .ok();
        })
        .detach();
    }
}

impl Render for LibraryRootsEditor {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let theme = cx.theme();
        let rows = Self::build_rows(&self.config);

        let mut list_container = div()
            .id("library-roots-list")
            .h(px(240.))
            .w_full()
            .overflow_y_scroll()
            .border_1()
            .border_color(theme.border)
            .rounded(theme.radius);

        if rows.is_empty() {
            list_container = list_container.child(
                div()
                    .size_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .child("No library folders configured."),
            );
        } else {
            list_container = list_container.child(v_flex().children(
                rows.into_iter().enumerate().map(|(row_ix, row)| {
                    h_flex()
                        .id(("library-root-row", row_ix))
                        .w_full()
                        .items_center()
                        .gap_2()
                        .px_3()
                        .py_2()
                        .border_b_1()
                        .border_color(theme.border)
                        .child(
                            div()
                                .id(("library-root-path", row_ix))
                                .flex_1()
                                .min_w_0()
                                .overflow_x_scroll()
                                .text_sm()
                                .whitespace_nowrap()
                                .child(display_path(&row.path)),
                        )
                        .child(
                            Button::new(("delete-root", row_ix))
                                .text()
                                .flex_shrink_0()
                                .icon(IconName::Close)
                                .tooltip("Remove folder")
                                .on_click(cx.listener(move |editor, _, _, cx| {
                                    editor.delete_row(row_ix, cx);
                                })),
                        )
                }),
            ));
        }

        v_flex()
            .gap_3()
            .child(DialogTitle::new().child("Library Roots"))
            .child(list_container)
            .child(
                h_flex().child(
                    Button::new("add-root")
                        .outline()
                        .label("Add Folder…")
                        .on_click(cx.listener(|_, _, window, cx| {
                            Self::pick_folder(window, cx);
                        })),
                ),
            )
    }
}

pub fn open_library_roots_dialog(window: &mut Window, cx: &mut App) {
    let initial_config = cx.global::<PulseLibrary>().inner().config().clone();

    let editor = cx.new(|_| LibraryRootsEditor::new(initial_config));

    window.open_dialog(cx, move |dialog, _, _| {
        let editor_apply = editor.clone();
        dialog
            .w(px(560.))
            .keyboard(true)
            .child(editor.clone())
            .footer(
            DialogFooter::new()
                .gap_2()
                .child(
                    DialogClose::new().child(
                        Button::new("cancel-roots")
                            .label("Cancel")
                            .outline(),
                    ),
                )
                .child(
                    Button::new("apply-roots")
                        .label("Apply & Rescan")
                        .primary()
                        .on_click(move |_, window, cx| {
                            let config = editor_apply.read(cx).config().clone();
                            PulseLibrary::apply_config(cx, config);
                            window.close_dialog(cx);
                        }),
                ),
        )
    });
}

#[must_use]
fn display_path(path: &Path) -> String {
    let raw = path.display().to_string();
    strip_verbatim_prefix(&raw)
}

fn strip_verbatim_prefix(path: &str) -> String {
    path.strip_prefix(r"\\?\UNC\").map_or_else(
        || {
            path.strip_prefix(r"\\?\")
                .map_or_else(|| path.to_string(), std::string::ToString::to_string)
        },
        |rest| format!(r"\\{rest}"),
    )
}

fn paths_equal(a: &Path, b: &Path) -> bool {
    a.canonicalize()
        .ok()
        .zip(b.canonicalize().ok())
        .is_some_and(|(a, b)| a == b)
        || a == b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_verbatim_prefix() {
        assert_eq!(
            display_path(Path::new(r"\\?\C:\Users\user\Music")),
            r"C:\Users\user\Music"
        );
    }

    #[test]
    fn strips_verbatim_unc_prefix() {
        assert_eq!(
            strip_verbatim_prefix(r"\\?\UNC\server\share"),
            r"\\server\share"
        );
    }
}
