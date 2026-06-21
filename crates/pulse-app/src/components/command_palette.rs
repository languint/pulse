use std::rc::Rc;

use gpui::{
    App, AppContext, Context, Entity, FocusHandle, Focusable, InteractiveElement, IntoElement,
    Keystroke, KeyBinding, ParentElement, Render, ScrollStrategy, Size, Styled, Window, div, px,
    prelude::FluentBuilder, size,
};
use gpui_component::{
    ActiveTheme, StyledExt as _, VirtualListScrollHandle,
    h_flex,
    input::{Input, InputEvent, InputState},
    kbd::Kbd,
    v_virtual_list,
};
use pulse_keymap::KeymapAction;

use crate::actions::{
    CommandPaletteConfirm, CommandPaletteDismiss, CommandPaletteSelectDown,
    CommandPaletteSelectUp, CommandPaletteTab,
};
use crate::components::library_roots_dialog::open_library_roots_dialog;
use crate::components::theme_picker_dialog::open_themes_folder;
use crate::config::PulseConfig;
use crate::pulse;
use crate::theme_list::selectable_themes;

const CONTEXT: &str = "CommandPalette";
const VISIBLE_ROWS: usize = 5;
const ROW_HEIGHT: f32 = 36.;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PaletteMode {
    Commands,
    Themes,
}

#[derive(Clone, Debug)]
enum CommandKind {
    ChangeTheme,
    SetTheme(String),
    ManageLibraryRoots,
    OpenThemesFolder,
}

#[derive(Clone, Debug)]
struct CommandItem {
    label: String,
    keywords: String,
    kind: CommandKind,
}

const INPUT_CONTEXT: &str = "Input";

pub struct CommandPalette {
    open: bool,
    mode: PaletteMode,
    list_focused: bool,
    list_focus_handle: FocusHandle,
    restore_focus: FocusHandle,
    query_input: Entity<InputState>,
    items: Vec<CommandItem>,
    theme_items: Vec<CommandItem>,
    filtered: Vec<usize>,
    list_sizes: Rc<Vec<Size<gpui::Pixels>>>,
    list_scroll: VirtualListScrollHandle,
    selected: usize,
    committed_theme: String,
}

impl CommandPalette {
    pub fn new(window: &mut Window, restore_focus: FocusHandle, cx: &mut Context<Self>) -> Self {
        let list_focus_handle = cx.focus_handle();
        let query_input = cx.new(|cx| InputState::new(window, cx).placeholder("Type a command…"));

        cx.subscribe_in(&query_input, window, |this, input, event, window, cx| match event {
            InputEvent::Change => this.on_query_changed(input, cx),
            InputEvent::PressEnter {
                secondary: false,
                shift: false,
            } if !this.list_focused => {
                this.confirm(window, cx);
            }
            InputEvent::PressEnter { .. } | InputEvent::Blur | InputEvent::Focus => {}
        })
        .detach();

        Self {
            open: false,
            mode: PaletteMode::Commands,
            list_focused: false,
            list_focus_handle,
            restore_focus,
            query_input,
            items: Vec::new(),
            theme_items: Vec::new(),
            filtered: Vec::new(),
            list_sizes: list_sizes_for(0),
            list_scroll: VirtualListScrollHandle::new(),
            selected: 0,
            committed_theme: String::new(),
        }
    }

    #[must_use]
    pub const fn is_open(&self) -> bool {
        self.open
    }

    pub fn toggle(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.open {
            self.dismiss(window, cx);
        } else {
            self.open_palette(window, cx);
        }
    }

    pub fn handle_tab(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.open {
            self.open_palette(window, cx);
            return;
        }

        if self.list_focused {
            self.focus_query_input(window, cx);
        } else {
            self.focus_list(window, cx);
        }
    }

    fn focus_query_input(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.list_focused = false;
        self.query_input.update(cx, |input, cx| input.focus(window, cx));
        cx.notify();
    }

    fn focus_list(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.list_focused = true;
        self.list_focus_handle.focus(window, cx);
        cx.notify();
    }

    fn close_palette(&mut self, window: &mut Window, cx: &mut Context<Self>, revert_preview: bool) {
        if self.open && revert_preview {
            pulse::preview_theme(cx, &self.committed_theme);
        }
        self.open = false;
        self.mode = PaletteMode::Commands;
        self.list_focused = false;
        self.restore_focus.focus(window, cx);
        cx.notify();
    }

    fn open_palette(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.committed_theme = cx.global::<PulseConfig>().theme.clone();
        self.mode = PaletteMode::Commands;
        self.list_focused = false;
        self.items = build_commands();
        self.theme_items = build_theme_commands(cx);
        self.query_input.update(cx, |input, cx| {
            input.set_placeholder("Type a command…", window, cx);
            input.set_value("", window, cx);
        });
        self.refilter("", cx);
        self.open = true;
        self.focus_query_input(window, cx);
    }

    fn dismiss(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.open {
            self.close_palette(window, cx, true);
        }
    }

    fn enter_theme_mode(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.mode = PaletteMode::Themes;
        self.list_focused = false;
        self.theme_items = build_theme_commands(cx);
        self.query_input.update(cx, |input, cx| {
            input.set_placeholder("Search themes…", window, cx);
            input.set_value("", window, cx);
        });
        self.refilter("", cx);
        self.focus_query_input(window, cx);
    }

    fn on_query_changed(&mut self, input: &Entity<InputState>, cx: &mut Context<Self>) {
        let query = input.read(cx).text().to_string();
        self.refilter(&query, cx);
    }

    fn active_items(&self) -> &[CommandItem] {
        match self.mode {
            PaletteMode::Commands => &self.items,
            PaletteMode::Themes => &self.theme_items,
        }
    }

    fn refilter(&mut self, query: &str, cx: &mut Context<Self>) {
        let query = query.trim().to_ascii_lowercase();
        let items = self.active_items();

        self.filtered = items
            .iter()
            .enumerate()
            .filter(|(_, item)| {
                query.is_empty()
                    || item.label.to_ascii_lowercase().contains(&query)
                    || item.keywords.to_ascii_lowercase().contains(&query)
            })
            .map(|(index, _)| index)
            .collect();

        self.list_sizes = list_sizes_for(self.filtered.len());
        self.selected = 0;
        if query.is_empty() && self.mode == PaletteMode::Themes {
            self.select_current_theme();
        }
        self.list_scroll.scroll_to_item(self.selected, ScrollStrategy::Top);
        self.preview_selection(cx);
        cx.notify();
    }

    fn select_current_theme(&mut self) {
        let current = &self.committed_theme;
        if let Some(filtered_ix) = self.filtered.iter().position(|item_ix| {
            self.theme_items.get(*item_ix).is_some_and(|item| {
                matches!(&item.kind, CommandKind::SetTheme(name) if name == current)
            })
        }) {
            self.selected = filtered_ix;
        }
    }

    fn selected_item(&self) -> Option<&CommandItem> {
        self.filtered
            .get(self.selected)
            .and_then(|index| self.active_items().get(*index))
    }

    fn preview_selection(&mut self, cx: &mut Context<Self>) {
        match self.mode {
            PaletteMode::Themes => match self.selected_item().map(|item| &item.kind) {
                Some(CommandKind::SetTheme(name)) => pulse::preview_theme(cx, name),
                _ => pulse::preview_theme(cx, &self.committed_theme),
            },
            PaletteMode::Commands => {
                pulse::preview_theme(cx, &self.committed_theme);
            }
        }
    }

    fn move_selection(&mut self, delta: isize, cx: &mut Context<Self>) {
        if self.filtered.is_empty() {
            return;
        }

        let len = self.filtered.len() as isize;
        self.selected = (self.selected as isize + delta).rem_euclid(len) as usize;
        self.list_scroll
            .scroll_to_item(self.selected, ScrollStrategy::Top);
        self.preview_selection(cx);
        cx.notify();
    }

    fn confirm(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(item) = self.selected_item().cloned() else {
            self.dismiss(window, cx);
            return;
        };

        match (&self.mode, item.kind) {
            (PaletteMode::Commands, CommandKind::ChangeTheme) => {
                self.enter_theme_mode(window, cx);
            }
            (PaletteMode::Themes, CommandKind::SetTheme(name)) => {
                pulse::set_theme(cx, &name);
                self.committed_theme = name;
                self.close_palette(window, cx, false);
            }
            (_, kind) => {
                if matches!(kind, CommandKind::ManageLibraryRoots) {
                    self.open = false;
                    self.mode = PaletteMode::Commands;
                    self.list_focused = false;
                    cx.notify();
                    execute_command(kind, window, cx);
                } else {
                    self.close_palette(window, cx, true);
                    execute_command(kind, window, cx);
                }
            }
        }
    }

    fn on_select_up(&mut self, _: &CommandPaletteSelectUp, window: &mut Window, cx: &mut Context<Self>) {
        cx.stop_propagation();
        if !self.list_focused {
            self.focus_list(window, cx);
        }
        self.move_selection(-1, cx);
    }

    fn on_select_down(
        &mut self,
        _: &CommandPaletteSelectDown,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        cx.stop_propagation();
        if !self.list_focused {
            self.focus_list(window, cx);
        }
        self.move_selection(1, cx);
    }

    fn on_tab(&mut self, _: &CommandPaletteTab, window: &mut Window, cx: &mut Context<Self>) {
        cx.stop_propagation();
        self.handle_tab(window, cx);
    }

    fn on_confirm(&mut self, _: &CommandPaletteConfirm, window: &mut Window, cx: &mut Context<Self>) {
        cx.stop_propagation();
        self.confirm(window, cx);
    }

    fn on_cancel(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.mode == PaletteMode::Themes {
            pulse::preview_theme(cx, &self.committed_theme);
            self.mode = PaletteMode::Commands;
            let query = self.query_input.read(cx).text().to_string();
            self.query_input.update(cx, |input, cx| {
                input.set_placeholder("Type a command…", window, cx);
            });
            self.refilter(&query, cx);
            cx.notify();
            return;
        }

        self.dismiss(window, cx);
    }
}

impl Focusable for CommandPalette {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        if self.list_focused {
            self.list_focus_handle.clone()
        } else {
            self.query_input.focus_handle(cx)
        }
    }
}

impl Render for CommandPalette {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.open {
            return div().into_any_element();
        }

        let theme = cx.theme();
        let selected = self.selected;
        let filtered = self.filtered.clone();
        let items = self.active_items().to_vec();
        let list_sizes = self.list_sizes.clone();
        let list_focus_handle = self.list_focus_handle.clone();
        let list_focused = self.list_focused;
        let focus_input_hint = focus_input_kbd_hint(window, cx);
        let entity = cx.entity();

        div()
            .id("command-palette-overlay")
            .absolute()
            .inset_0()
            .bg(theme.overlay)
            .flex()
            .justify_center()
            .items_start()
            .pt(px(72.))
            .on_action(cx.listener(|this, _: &CommandPaletteDismiss, window, cx| {
                this.on_cancel(window, cx);
            }))
            .on_mouse_down_out(cx.listener(|this, _, window, cx| {
                this.dismiss(window, cx);
            }))
            .child(
                div()
                    .id("command-palette")
                    .w(px(560.))
                    .flex_shrink_0()
                    .rounded(theme.radius)
                    .border_1()
                    .border_color(theme.border)
                    .bg(theme.popover)
                    .shadow_lg()
                    .overflow_hidden()
                    .child(
                        h_flex()
                            .items_center()
                            .gap_2()
                            .border_b_1()
                            .border_color(theme.border)
                            .px_3()
                            .py_2()
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .on_action(cx.listener(Self::on_tab))
                                    .child(Input::new(&self.query_input).appearance(false)),
                            )
                            .when(list_focused, |this| {
                                this.when_some(focus_input_hint, |row, hint| {
                                    row.flex_shrink_0().child(hint.outline())
                                })
                            }),
                    )
                    .when(filtered.is_empty(), |this| {
                        this.child(
                            div()
                                .h(px(ROW_HEIGHT * VISIBLE_ROWS as f32))
                                .flex()
                                .items_center()
                                .justify_center()
                                .px_3()
                                .text_sm()
                                .text_color(theme.muted_foreground)
                                .child(if self.mode == PaletteMode::Themes {
                                    "No matching themes"
                                } else {
                                    "No matching commands"
                                }),
                        )
                    })
                    .when(!filtered.is_empty(), |this| {
                        this.child(
                            div()
                                .id("command-palette-list")
                                .key_context(CONTEXT)
                                .track_focus(&list_focus_handle)
                                .on_action(cx.listener(Self::on_select_up))
                                .on_action(cx.listener(Self::on_select_down))
                                .on_action(cx.listener(Self::on_confirm))
                                .on_action(cx.listener(Self::on_tab))
                                .child(
                                    v_virtual_list(
                                        entity,
                                        match self.mode {
                                            PaletteMode::Commands => "command-palette-commands",
                                            PaletteMode::Themes => "command-palette-themes",
                                        },
                                        list_sizes,
                                        move |_, visible_range, _, cx| {
                                            visible_range
                                                .filter_map(|row_ix| {
                                                    let item_ix = *filtered.get(row_ix)?;
                                                    let item = items.get(item_ix)?;
                                                    Some(render_row(
                                                        row_ix,
                                                        item,
                                                        row_ix == selected,
                                                        cx,
                                                    ))
                                                })
                                                .collect()
                                        },
                                    )
                                    .h(px(ROW_HEIGHT * VISIBLE_ROWS as f32))
                                    .track_scroll(&self.list_scroll),
                                ),
                        )
                    }),
            )
            .into_any_element()
    }
}

fn render_row(
    row_ix: usize,
    item: &CommandItem,
    is_selected: bool,
    cx: &Context<CommandPalette>,
) -> gpui::AnyElement {
    let theme = cx.theme();

    h_flex()
        .id(("command-palette-item", row_ix))
        .w_full()
        .h(px(ROW_HEIGHT))
        .px_3()
        .items_center()
        .when(is_selected, |row| {
            row.bg(theme.list_active)
                .border_l_2()
                .border_color(theme.primary)
        })
        .when(!is_selected, |row| row.hover(|element| element.bg(theme.list_hover)))
        .child(
            div()
                .text_sm()
                .when(is_selected, |label| label.font_semibold())
                .child(item.label.clone()),
        )
        .into_any_element()
}

pub fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("up", CommandPaletteSelectUp, Some(CONTEXT)),
        KeyBinding::new("down", CommandPaletteSelectDown, Some(CONTEXT)),
        KeyBinding::new("enter", CommandPaletteConfirm, Some(CONTEXT)),
        KeyBinding::new("escape", CommandPaletteDismiss, Some(CONTEXT)),
        KeyBinding::new("tab", CommandPaletteTab, Some(CONTEXT)),
        KeyBinding::new("tab", CommandPaletteTab, Some(INPUT_CONTEXT)),
    ]);
}

fn focus_input_kbd_hint(window: &Window, cx: &App) -> Option<Kbd> {
    Kbd::binding_for_action(&CommandPaletteTab, Some(CONTEXT), window).or_else(|| {
        cx.global::<PulseConfig>()
            .keymap
            .keystrokes_for(KeymapAction::OpenCommandPalette)
            .first()
            .and_then(|stroke| Keystroke::parse(stroke).ok())
            .map(Kbd::new)
    })
}

fn list_sizes_for(count: usize) -> Rc<Vec<Size<gpui::Pixels>>> {
    Rc::new(vec![size(px(0.), px(ROW_HEIGHT)); count])
}

fn build_commands() -> Vec<CommandItem> {
    vec![
        CommandItem {
            label: "Change Theme".into(),
            keywords: "theme appearance color mode switch".into(),
            kind: CommandKind::ChangeTheme,
        },
        CommandItem {
            label: "Library: Manage Roots…".into(),
            keywords: "library roots folders scan music".into(),
            kind: CommandKind::ManageLibraryRoots,
        },
    ]
}

fn build_theme_commands(cx: &App) -> Vec<CommandItem> {
    let mut items: Vec<CommandItem> = selectable_themes(cx)
        .into_iter()
        .map(|theme| {
            let name = theme.name.to_string();
            CommandItem {
                label: name.clone(),
                keywords: format!("theme appearance color mode {name}"),
                kind: CommandKind::SetTheme(name),
            }
        })
        .collect();

    items.push(CommandItem {
        label: "Open Themes Folder…".into(),
        keywords: "theme custom json folder directory".into(),
        kind: CommandKind::OpenThemesFolder,
    });

    items
}

fn execute_command(kind: CommandKind, window: &mut Window, cx: &mut App) {
    match kind {
        CommandKind::ChangeTheme => {}
        CommandKind::SetTheme(name) => pulse::set_theme(cx, &name),
        CommandKind::ManageLibraryRoots => open_library_roots_dialog(window, cx),
        CommandKind::OpenThemesFolder => open_themes_folder(cx),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn matches_query(item: &CommandItem, query: &str) -> bool {
        let query = query.trim().to_ascii_lowercase();
        query.is_empty()
            || item.label.to_ascii_lowercase().contains(&query)
            || item.keywords.to_ascii_lowercase().contains(&query)
    }

    #[test]
    fn filters_commands_by_label_and_keywords() {
        let items = build_commands();

        assert!(items.iter().any(|item| matches_query(item, "theme")));
        assert!(items.iter().any(|item| matches_query(item, "library")));
        assert!(!items.iter().any(|item| item.label.contains("Pulse Dark")));
    }

    #[test]
    fn theme_list_is_separate_from_commands() {
        let commands = build_commands();
        assert!(commands.iter().all(|item| !matches!(item.kind, CommandKind::SetTheme(_))));
    }
}
