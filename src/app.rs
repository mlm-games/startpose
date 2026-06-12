#![allow(non_snake_case)]

use std::rc::Rc;

use repose_core::{ColorScheme, CursorIcon, PaddingValues, prelude::*, set_theme_default};
use repose_material::material3::{self, OutlinedTextFieldConfig};
use repose_ui::overlay::{OverlayHandle, SnackbarAction, SnackbarController, SnackbarRequest};
use repose_ui::scroll::{ScrollArea, remember_scroll_state};
use repose_ui::*;

use crate::storage::{self, Bookmark};

fn open_url(url: &str) {
    if let Some(w) = web_sys::window() {
        if w.open_with_url_and_target(url, "_blank").is_ok() {
            return;
        }
        let _ = w.location().set_href(url);
    }
}

fn normalize_url(s: &str) -> Option<String> {
    let t = s.trim();
    if t.is_empty() {
        return None;
    }
    if t.starts_with("http://") || t.starts_with("https://") {
        return Some(t.to_string());
    }
    if !t.contains(' ') && t.contains('.') {
        return Some(format!("https://{t}"));
    }
    None
}

fn truncate_url(url: &str) -> String {
    url.replace("https://", "")
        .replace("http://", "")
        .replace("www.", "")
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SearchEngine {
    DuckDuckGo,
    Google,
    Brave,
}

impl SearchEngine {
    const ALL: [SearchEngine; 3] = [Self::DuckDuckGo, Self::Google, Self::Brave];

    fn label(self) -> &'static str {
        match self {
            Self::DuckDuckGo => "DuckDuckGo",
            Self::Google => "Google",
            Self::Brave => "Brave",
        }
    }

    fn url(self, query: &str) -> String {
        let q = urlencoding::encode(query.trim());
        match self {
            Self::DuckDuckGo => format!("https://duckduckgo.com/?q={q}"),
            Self::Google => format!("https://www.google.com/search?q={q}"),
            Self::Brave => format!("https://search.brave.com/search?q={q}"),
        }
    }
}

fn search_or_open(engine: SearchEngine, input: &str) {
    if let Some(url) = normalize_url(input) {
        open_url(&url);
    } else {
        open_url(&engine.url(input));
    }
}

fn make_theme() -> Theme {
    Theme {
        colors: ColorScheme {
            background: Color::from_hex("#0B0F14"),
            surface: Color::from_hex("#111827"),
            surface_variant: Color::from_hex("#1F2937"),
            surface_container: Color::from_hex("#1A2332"),
            surface_container_high: Color::from_hex("#243041"),
            surface_container_highest: Color::from_hex("#1E293B"),
            surface_bright: Color::from_hex("#1E293B"),
            surface_dim: Color::from_hex("#0B0F14"),
            surface_tint: Color::from_hex("#3B82F6"),
            on_surface: Color::from_hex("#E5E7EB"),
            on_surface_variant: Color::from_hex("#9CA3AF"),
            primary: Color::from_hex("#3B82F6"),
            on_primary: Color::WHITE,
            primary_container: Color::from_hex("#1E3A5F"),
            on_primary_container: Color::from_hex("#93C5FD"),
            secondary: Color::from_hex("#6B7280"),
            on_secondary: Color::WHITE,
            secondary_container: Color::from_hex("#374151"),
            on_secondary_container: Color::from_hex("#D1D5DB"),
            on_background: Color::from_hex("#E5E7EB"),
            tertiary: Color::from_hex("#8B5CF6"),
            on_tertiary: Color::WHITE,
            tertiary_container: Color::from_hex("#3B0764"),
            on_tertiary_container: Color::from_hex("#D8B4FE"),
            error: Color::from_hex("#EF4444"),
            on_error: Color::WHITE,
            error_container: Color::from_hex("#7F1D1D"),
            on_error_container: Color::from_hex("#FCA5A5"),
            outline: Color::from_hex("#374151"),
            outline_variant: Color::from_hex("#243041"),
            inverse_surface: Color::from_hex("#E5E7EB"),
            inverse_on_surface: Color::from_hex("#0B0F14"),
            inverse_primary: Color::from_hex("#3B82F6"),
            scrim: Color(0, 0, 0, 82),
            shadow: Color::BLACK,
            focus: Color::from_hex("#60A5FA"),
            surface_container_lowest: Color::from_hex("#070A0F"),
            surface_container_low: Color::from_hex("#0F172A"),
        },
        focus: Color::from_hex("#60A5FA"),
        button_bg: Color::from_hex("#1F2937"),
        button_bg_hover: Color::from_hex("#243041"),
        button_bg_pressed: Color::from_hex("#2B3A52"),
        scrollbar_track: Color(0xFF, 0xFF, 0xFF, 16),
        scrollbar_thumb: Color(0xFF, 0xFF, 0xFF, 80),
        ..Default::default()
    }
}

const SP_2: f32 = 8.0;
const SP_3: f32 = 12.0;
const SP_4: f32 = 16.0;
const SP_5: f32 = 20.0;
const SP_6: f32 = 24.0;
const SP_8: f32 = 32.0;
const SP_10: f32 = 40.0;

fn show_snackbar(sb: &SnackbarController, message: &str, duration_ms: u32) {
    let sb = sb.clone();
    let msg = message.to_string();
    let sb_builder = sb.clone();
    sb.show(SnackbarRequest {
        message: msg.clone(),
        action: Some(SnackbarAction {
            label: "Dismiss".to_string(),
            on_click: Rc::new({
                let sb = sb.clone();
                move || sb.dismiss()
            }),
        }),
        duration_ms,
        builder: Rc::new(move || {
            material3::Snackbar(
                msg.clone(),
                Some(SnackbarAction {
                    label: "Dismiss".to_string(),
                    on_click: Rc::new({
                        let sb = sb_builder.clone();
                        move || sb.dismiss()
                    }),
                }),
                Modifier::new()
                    .absolute()
                    .offset(Some(SP_4), None, Some(SP_4), None),
            )
        }),
    });
}

fn BookmarkCard(
    bm: Bookmark,
    bookmarks: Rc<Signal<Vec<Bookmark>>>,
    snackbar: Rc<SnackbarController>,
) -> View {
    let url_open = bm.url.clone();
    let url_remove = bm.url.clone();

    let inner = Row(Modifier::new()
        .fill_max_width()
        .padding_values(PaddingValues {
            left: SP_4,
            right: SP_4,
            top: SP_3,
            bottom: SP_3,
        })
        .align_items(AlignItems::Center))
    .child((
        Column(Modifier::new().weight(1.0).min_width(0.0)).child((
            Text(&bm.title)
                .size(16.0)
                .single_line()
                .overflow_ellipsize()
                .color(theme().on_surface),
            Text(&truncate_url(&bm.url))
                .size(12.0)
                .single_line()
                .overflow_ellipsize()
                .color(theme().on_surface_variant),
        )),
        Box(Modifier::new().width(SP_2).height(1.0)),
        material3::IconButton(Text("×").size(18.0), {
            let bms = bookmarks.clone();
            let snackbar = snackbar.clone();
            move || {
                bms.update(|v| v.retain(|b| b.url != url_remove));
                storage::save_bookmarks(&bms.get());
                show_snackbar(&snackbar, "Bookmark removed", 3000);
            }
        }),
    ));

    material3::ClickableOutlinedCard(
        move || open_url(&url_open),
        Modifier::new().fill_max_width().cursor(CursorIcon::Pointer),
        inner,
    )
}

fn AddBookmarkForm(
    bookmarks: Rc<Signal<Vec<Bookmark>>>,
    new_title: Rc<Signal<String>>,
    new_url: Rc<Signal<String>>,
    snackbar: Rc<SnackbarController>,
    on_dismiss: impl Fn() + 'static + Clone,
) -> View {
    let on_dismiss_clone = on_dismiss.clone();

    Column(Modifier::new().fill_max_width().padding(SP_5)).child((
        Row(Modifier::new()
            .fill_max_width()
            .align_items(AlignItems::Center)
            .padding_values(PaddingValues {
                bottom: SP_3,
                ..Default::default()
            }))
        .child((
            Text("Add bookmark").size(18.0).color(theme().on_surface),
            Spacer(),
            material3::IconButton(Text("×").size(18.0), {
                let on_dismiss = on_dismiss.clone();
                move || on_dismiss()
            }),
        )),
        material3::OutlinedTextField(
            Modifier::new().fill_max_width(),
            new_title.get(),
            {
                let nt = new_title.clone();
                move |s| nt.set(s)
            },
            OutlinedTextFieldConfig {
                label: Some("Title".to_string()),
                placeholder: None,
                leading_icon: None,
                trailing_icon: None,
                single_line: true,
                is_error: false,
                enabled: true,
                on_submit: None,
            },
        ),
        Box(Modifier::new().height(SP_3).width(1.0)),
        material3::OutlinedTextField(
            Modifier::new().fill_max_width(),
            new_url.get(),
            {
                let nu = new_url.clone();
                move |s| nu.set(s)
            },
            OutlinedTextFieldConfig {
                label: Some("URL".to_string()),
                placeholder: Some("https://example.com".to_string()),
                leading_icon: None,
                trailing_icon: None,
                single_line: true,
                is_error: false,
                enabled: true,
                on_submit: None,
            },
        ),
        Box(Modifier::new().height(SP_4).width(1.0)),
        Row(Modifier::new()
            .fill_max_width()
            .align_items(AlignItems::Center))
        .child((
            Spacer(),
            material3::TextButton(
                Modifier::new(),
                {
                    let on_dismiss = on_dismiss.clone();
                    move || on_dismiss()
                },
                || Text("Cancel"),
            ),
            Box(Modifier::new().width(SP_2).height(1.0)),
            material3::FilledButton(
                Modifier::new(),
                {
                    let bookmarks = bookmarks.clone();
                    let new_title = new_title.clone();
                    let new_url = new_url.clone();
                    let snackbar = snackbar.clone();
                    let on_dismiss = on_dismiss_clone.clone();
                    move || {
                        let title = new_title.get().trim().to_string();
                        let url_raw = new_url.get().trim().to_string();

                        if title.is_empty() || url_raw.is_empty() {
                            show_snackbar(&snackbar, "Title and URL are required", 4000);
                            return;
                        }
                        let Some(url) = normalize_url(&url_raw) else {
                            show_snackbar(&snackbar, "Invalid URL format", 4000);
                            return;
                        };

                        bookmarks.update(|v| v.push(Bookmark { title, url }));
                        storage::save_bookmarks(&bookmarks.get());

                        new_title.set(String::new());
                        new_url.set(String::new());
                        on_dismiss();
                        show_snackbar(&snackbar, "Bookmark added", 3000);
                    }
                },
                || Text("Add").color(theme().on_primary),
            ),
        )),
    ))
}

fn EmptyState() -> View {
    Column(
        Modifier::new()
            .fill_max_width()
            .padding_values(PaddingValues {
                top: SP_8,
                bottom: SP_8,
                ..Default::default()
            })
            .align_items(AlignItems::Center),
    )
    .child((
        Text("No bookmarks yet")
            .size(22.0)
            .color(theme().on_surface),
        Text("Tap '+ Add bookmark' to save a link.")
            .size(14.0)
            .color(theme().on_surface_variant),
    ))
}

pub fn app(s: &mut Scheduler) -> View {
    set_theme_default(make_theme());

    let bookmarks = remember(|| signal(storage::load_bookmarks()));
    let query = remember(|| signal(String::new()));
    let engine = remember(|| signal(SearchEngine::DuckDuckGo));
    let new_title = remember(|| signal(String::new()));
    let new_url = remember(|| signal(String::new()));
    let show_add = remember(|| signal(false));
    let root_scroll = remember_scroll_state("root_scroll");

    let overlay = remember(OverlayHandle::new);
    let snackbar = remember(|| SnackbarController::new((*overlay).clone()));

    let px_w = s.size.0 as f32;
    let scale = repose_core::locals::density().scale * repose_core::locals::ui_scale().0;
    let dp_w = if scale > 0.0 { px_w / scale } else { px_w };
    let cols = if dp_w < 520.0 {
        1
    } else if dp_w < 860.0 {
        2
    } else if dp_w < 1180.0 {
        3
    } else {
        4
    };

    // Clones for Scaffold closures
    let bms = bookmarks.clone();
    let q = query.clone();
    let eng = engine.clone();
    let nt = new_title.clone();
    let nu = new_url.clone();
    let show = show_add.clone();
    let sb = snackbar.clone();
    let rs = root_scroll.clone();

    let content = material3::Scaffold(None, None, None, move |padding| {
        let bms = bms.clone();
        let q = q.clone();
        let eng = eng.clone();
        let nt = nt.clone();
        let nu = nu.clone();
        let show = show.clone();
        let sb = sb.clone();
        let rs = rs.clone();

        Surface(
            Modifier::new()
                .fill_max_size()
                .padding_values(padding)
                .background(theme().background),
            ScrollArea(
                Modifier::new().fill_max_size(),
                rs,
                Column(
                    Modifier::new()
                        .fill_max_width()
                        .padding(SP_6)
                        .align_items(AlignItems::Center),
                )
                .child(
                    Box(Modifier::new()
                        .fill_max_width()
                        .max_width(900.0)
                        .min_width(0.0))
                    .child(
                        Column(
                            Modifier::new()
                                .fill_max_width()
                                .align_items(AlignItems::Center),
                        )
                        .child((
                            Text("Startpage")
                                .size(32.0)
                                .color(theme().on_surface)
                                .modifier(Modifier::new().padding_values(PaddingValues {
                                    top: SP_10,
                                    bottom: SP_10,
                                    ..Default::default()
                                })),
                            Box(Modifier::new().fill_max_width().max_width(600.0)).child(
                                Column(
                                    Modifier::new()
                                        .fill_max_width()
                                        .align_items(AlignItems::Center),
                                )
                                .child((
                                    material3::OutlinedTextField(
                                        Modifier::new().fill_max_width().height(56.0),
                                        q.get(),
                                        {
                                            let q = q.clone();
                                            move |s| q.set(s)
                                        },
                                        OutlinedTextFieldConfig {
                                            label: None,
                                            placeholder: Some("Search or type a URL…".to_string()),
                                            leading_icon: None,
                                            trailing_icon: None,
                                            single_line: true,
                                            is_error: false,
                                            enabled: true,
                                            on_submit: Some(Rc::new({
                                                let eng = eng.clone();
                                                move |submitted| {
                                                    search_or_open(eng.get(), &submitted)
                                                }
                                            })),
                                        },
                                    ),
                                    Row(Modifier::new().padding_values(PaddingValues {
                                        top: SP_3,
                                        ..Default::default()
                                    }))
                                    .child(
                                        SearchEngine::ALL
                                            .iter()
                                            .map(|&e| {
                                                let eng = eng.clone();
                                                material3::FilterChip(
                                                    eng.get() == e,
                                                    move || eng.set(e),
                                                    Text(e.label()),
                                                    None,
                                                    None,
                                                )
                                            })
                                            .collect::<Vec<_>>(),
                                    ),
                                )),
                            ),
                            if bms.get().is_empty() {
                                EmptyState()
                            } else {
                                Box(Modifier::new().fill_max_width().padding_values(
                                    PaddingValues {
                                        top: SP_6,
                                        bottom: SP_6,
                                        ..Default::default()
                                    },
                                ))
                                .child(Grid(
                                    cols,
                                    Modifier::new().fill_max_width(),
                                    bms.get()
                                        .iter()
                                        .map(|bm| {
                                            let bm = bm.clone();
                                            BookmarkCard(bm, bms.clone(), sb.clone())
                                        })
                                        .collect::<Vec<_>>(),
                                    SP_3,
                                    SP_3,
                                ))
                            },
                            if show.get() {
                                Box(Modifier::new().fill_max_width().max_width(500.0)).child(
                                    AddBookmarkForm(
                                        bms.clone(),
                                        nt.clone(),
                                        nu.clone(),
                                        sb.clone(),
                                        {
                                            let show = show.clone();
                                            move || show.set(false)
                                        },
                                    ),
                                )
                            } else {
                                Box(Modifier::new().padding_values(PaddingValues {
                                    top: SP_4,
                                    ..Default::default()
                                }))
                                .child(
                                    material3::OutlinedButton(
                                        Modifier::new(),
                                        {
                                            let show = show.clone();
                                            move || show.set(true)
                                        },
                                        || Text("+ Add bookmark").color(theme().on_surface_variant),
                                    ),
                                )
                            },
                        )),
                    ),
                ),
            ),
        )
    });

    overlay.host(Modifier::new().fill_max_size(), content)
}
