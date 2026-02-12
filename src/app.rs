#![allow(non_snake_case)]

use std::rc::Rc;

use repose_core::{CursorIcon, PaddingValues, prelude::*, set_theme_default};
use repose_material::material3;
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SearchEngine {
    DuckDuckGo,
    Google,
    Brave,
}

impl SearchEngine {
    fn label(self) -> &'static str {
        match self {
            SearchEngine::DuckDuckGo => "DuckDuckGo",
            SearchEngine::Google => "Google",
            SearchEngine::Brave => "Brave",
        }
    }

    fn url(self, query: &str) -> String {
        let q = urlencoding::encode(query.trim());
        match self {
            SearchEngine::DuckDuckGo => format!("https://duckduckgo.com/?q={q}"),
            SearchEngine::Google => format!("https://www.google.com/search?q={q}"),
            SearchEngine::Brave => format!("https://search.brave.com/search?q={q}"),
        }
    }
}

fn search_or_open(engine: SearchEngine, input: &str) {
    if let Some(url) = normalize_url(input) {
        open_url(&url);
        return;
    }
    open_url(&engine.url(input));
}

fn theme_pro() -> Theme {
    let mut t = Theme::default();
    t.background = Color::from_hex("#0B0F14");
    t.surface = Color::from_hex("#111827");
    t.on_surface = Color::from_hex("#E5E7EB");
    t.primary = Color::from_hex("#3B82F6");
    t.on_primary = Color::WHITE;
    t.outline = Color::from_hex("#243041");
    t.focus = Color::from_hex("#60A5FA");
    t.button_bg = Color::from_hex("#1F2937");
    t.button_bg_hover = Color::from_hex("#243041");
    t.button_bg_pressed = Color::from_hex("#2B3A52");
    t.scrollbar_track = Color(0xFF, 0xFF, 0xFF, 16);
    t.scrollbar_thumb = Color(0xFF, 0xFF, 0xFF, 80);
    t
}

fn EnginePill(label: &str, selected: bool, on_click: impl Fn() + 'static) -> View {
    let bg = if selected {
        Color(theme().primary.0, theme().primary.1, theme().primary.2, 48)
    } else {
        Color(0, 0, 0, 0)
    };

    Button(
        Text(label).size(13.0).single_line().color(if selected {
            theme().primary
        } else {
            Color::from_hex("#9CA3AF")
        }),
        on_click,
    )
    .modifier(
        Modifier::new()
            .padding_values(PaddingValues {
                left: 12.0,
                right: 12.0,
                top: 6.0,
                bottom: 6.0,
            })
            .background(bg)
            .clip_rounded(999.0),
    )
}

fn IconButton(icon: &str, on_click: impl Fn() + 'static) -> View {
    Box(Modifier::new()
        .size(32.0, 32.0)
        .background(Color(0, 0, 0, 0))
        .clip_rounded(8.0)
        .clickable()
        .on_pointer_down(move |_| on_click()))
    .child(Text(icon).size(18.0).color(Color::from_hex("#9CA3AF")))
}

fn BookmarkTile(
    bm: Bookmark,
    bookmarks: Rc<Signal<Vec<Bookmark>>>,
    snackbar: Rc<SnackbarController>,
) -> View {
    let url = bm.url.clone();
    let url_clone = bm.url.clone();
    let title = bm.title.clone();

    let bms = bookmarks.clone();
    let snackbar_remove = snackbar.clone();
    let url_for_remove = bm.url.clone();

    Box(Modifier::new()
        .fill_max_width()
        .background(theme().surface)
        .border(1.0, theme().outline, 10.0)
        .clip_rounded(10.0)
        .padding_values(PaddingValues {
            left: 14.0,
            right: 10.0,
            top: 12.0,
            bottom: 12.0,
        })
        .clickable()
        .on_pointer_down(move |_| open_url(&url))
        .cursor(CursorIcon::Pointer))
    .child(
        Row(Modifier::new()
            .fill_max_width()
            .align_items(AlignItems::Center))
        .child((
            // Content area (title + url)
            Box(Modifier::new().weight(1.0).min_width(0.0)).child(
                Column(Modifier::new()).child((
                    Text(title)
                        .size(15.0)
                        .single_line()
                        .overflow_ellipsize()
                        .color(theme().on_surface)
                        .modifier(Modifier::new().fill_max_width()),
                    Text(truncate_url(&url_clone))
                        .size(12.0)
                        .single_line()
                        .overflow_ellipsize()
                        .color(Color::from_hex("#6B7280"))
                        .modifier(Modifier::new().fill_max_width()),
                )),
            ),
            // Remove button (only visible on hover/interaction)
            IconButton("×", {
                let bms = bms.clone();
                let snackbar_remove = snackbar_remove.clone();
                let url_for_remove = url_for_remove.clone();
                move || {
                    bms.update(|v| {
                        if let Some(pos) = v.iter().position(|x| x.url == url_for_remove) {
                            v.remove(pos);
                        }
                    });
                    storage::save_bookmarks(&bms.get());

                    let sb = snackbar_remove.clone();
                    sb.show(SnackbarRequest {
                        message: "Bookmark removed".to_string(),
                        action: None,
                        duration_ms: 3000,
                        builder: Rc::new({
                            let sb = snackbar_remove.clone();
                            move || {
                                material3::Snackbar(
                                    "Bookmark removed",
                                    Some(SnackbarAction {
                                        label: "Dismiss".to_string(),
                                        on_click: Rc::new({
                                            let sb = sb.clone();
                                            move || sb.dismiss()
                                        }),
                                    }),
                                    Modifier::new().absolute().offset(
                                        Some(16.0),
                                        None,
                                        Some(16.0),
                                        None,
                                    ),
                                )
                            }
                        }),
                    });
                }
            }),
        )),
    )
}

fn truncate_url(url: &str) -> String {
    url.replace("https://", "")
        .replace("http://", "")
        .replace("www.", "")
}

fn hash64(s: &str) -> u64 {
    let mut x: u64 = 14695981039346656037;
    for &b in s.as_bytes() {
        x ^= b as u64;
        x = x.wrapping_mul(1099511628211);
    }
    x
}

pub fn app(s: &mut Scheduler) -> View {
    set_theme_default(theme_pro());

    // State
    let bookmarks = remember(|| signal(storage::load_bookmarks()));
    let query = remember(|| signal(String::new()));
    let engine = remember(|| signal(SearchEngine::DuckDuckGo));
    let new_title = remember(|| signal(String::new()));
    let new_url = remember(|| signal(String::new()));
    let show_add_form = remember(|| signal(false));
    let form_epoch = remember(|| signal(0u64));
    let root_scroll = remember_scroll_state("root_scroll");

    let overlay = remember(|| OverlayHandle::new());
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

    let content = Surface(
        Modifier::new()
            .fill_max_size()
            .background(theme().background),
        ScrollArea(
            Modifier::new().fill_max_size(),
            root_scroll,
            Column(Modifier::new().fill_max_width().padding(24.0)).child(
                Box(Modifier::new()
                    .fill_max_width()
                    .max_width(900.0)
                    .align_self_center()
                    .min_width(0.0))
                .child(
                    Column(
                        Modifier::new()
                            .fill_max_width()
                            .align_items(AlignItems::Center),
                    )
                    .child((
                        // Header - minimal
                        Text("Startpage")
                            .size(32.0)
                            .color(theme().on_surface)
                            .modifier(Modifier::new().padding_values(PaddingValues {
                                top: 40.0,
                                bottom: 32.0,
                                ..Default::default()
                            })),
                        // Search Section - Dominant, centered
                        Box(Modifier::new()
                            .fill_max_width()
                            .max_width(600.0)
                            .padding_values(PaddingValues {
                                bottom: 16.0,
                                ..Default::default()
                            }))
                        .child(
                            Column(
                                Modifier::new()
                                    .fill_max_width()
                                    .align_items(AlignItems::Center),
                            )
                            .child((
                                // Large search input
                                Box(Modifier::new().fill_max_width()).child(TextField(
                                    "Search or type a URL…",
                                    Modifier::new()
                                        .key(0xA11CE_u64)
                                        .height(56.0)
                                        .fill_max_width()
                                        .background(Color::from_hex("#0F172A"))
                                        .border(1.0, theme().outline, 16.0)
                                        .clip_rounded(16.0),
                                    Some({
                                        let query = query.clone();
                                        move |s| query.set(s)
                                    }),
                                    Some({
                                        let engine = engine.clone();
                                        move |submitted: String| {
                                            search_or_open(engine.get(), &submitted)
                                        }
                                    }),
                                )),
                                // Engine pills - subtle, inline
                                Row(Modifier::new().padding_values(PaddingValues {
                                    top: 12.0,
                                    ..Default::default()
                                }))
                                .child((
                                    EnginePill(
                                        SearchEngine::DuckDuckGo.label(),
                                        engine.get() == SearchEngine::DuckDuckGo,
                                        {
                                            let engine = engine.clone();
                                            move || engine.set(SearchEngine::DuckDuckGo)
                                        },
                                    ),
                                    EnginePill(
                                        SearchEngine::Google.label(),
                                        engine.get() == SearchEngine::Google,
                                        {
                                            let engine = engine.clone();
                                            move || engine.set(SearchEngine::Google)
                                        },
                                    ),
                                    EnginePill(
                                        SearchEngine::Brave.label(),
                                        engine.get() == SearchEngine::Brave,
                                        {
                                            let engine = engine.clone();
                                            move || engine.set(SearchEngine::Brave)
                                        },
                                    ),
                                )),
                            )),
                        ),
                        // Bookmarks Grid - Flat tiles
                        if !bookmarks.get().is_empty() {
                            Box(Modifier::new()
                                .fill_max_width()
                                .padding_values(PaddingValues {
                                    top: 24.0,
                                    bottom: 24.0,
                                    ..Default::default()
                                }))
                            .child(Grid(
                                cols,
                                Modifier::new().fill_max_width(),
                                bookmarks
                                    .get()
                                    .iter()
                                    .map(|bm| {
                                        let bm = bm.clone();
                                        BookmarkTile(bm, bookmarks.clone(), snackbar.clone())
                                    })
                                    .collect::<Vec<_>>(),
                                12.0,
                                12.0,
                            ))
                        } else {
                            Box(Modifier::new())
                        },
                        // Add Bookmark Section - Collapsible
                        Box(Modifier::new()
                            .fill_max_width()
                            .padding_values(PaddingValues {
                                top: 16.0,
                                ..Default::default()
                            }))
                        .child(if show_add_form.get() {
                            // Expanded form
                            Box(Modifier::new()
                                .fill_max_width()
                                .max_width(500.0)
                                .background(theme().surface)
                                .border(1.0, theme().outline, 12.0)
                                .clip_rounded(12.0)
                                .padding(16.0))
                            .child(
                                Column(Modifier::new().fill_max_width()).child((
                                    Row(Modifier::new()
                                        .fill_max_width()
                                        .align_items(AlignItems::Center)
                                        .padding_values(PaddingValues {
                                            bottom: 12.0,
                                            ..Default::default()
                                        }))
                                    .child((
                                        Text("Add bookmark")
                                            .size(14.0)
                                            .color(Color::from_hex("#9CA3AF")),
                                        Spacer(),
                                        IconButton("×", {
                                            let show = show_add_form.clone();
                                            move || show.set(false)
                                        }),
                                    )),
                                    Row(Modifier::new().fill_max_width()).child((
                                        TextField(
                                            "Title",
                                            Modifier::new()
                                                .key(hash64("title") ^ form_epoch.get())
                                                .height(40.0)
                                                .weight(1.0)
                                                .min_width(0.0)
                                                .background(Color::from_hex("#0F172A"))
                                                .border(1.0, theme().outline, 10.0)
                                                .clip_rounded(10.0),
                                            Some({
                                                let new_title = new_title.clone();
                                                move |s| new_title.set(s)
                                            }),
                                            None::<fn(String)>,
                                        ),
                                        Box(Modifier::new().width(10.0).height(1.0)),
                                        TextField(
                                            "URL",
                                            Modifier::new()
                                                .key(hash64("url") ^ form_epoch.get())
                                                .height(40.0)
                                                .weight(2.0)
                                                .min_width(0.0)
                                                .background(Color::from_hex("#0F172A"))
                                                .border(1.0, theme().outline, 10.0)
                                                .clip_rounded(10.0),
                                            Some({
                                                let new_url = new_url.clone();
                                                move |s| new_url.set(s)
                                            }),
                                            None::<fn(String)>,
                                        ),
                                    )),
                                    Button(Text("Add Bookmark").color(theme().on_primary), {
                                        let bookmarks = bookmarks.clone();
                                        let new_title = new_title.clone();
                                        let new_url = new_url.clone();
                                        let snackbar = snackbar.clone();
                                        let form_epoch = form_epoch.clone();
                                        let show_form = show_add_form.clone();

                                        move || {
                                            let title = new_title.get().trim().to_string();
                                            let url_raw = new_url.get().trim().to_string();

                                            if title.is_empty() || url_raw.is_empty() {
                                                let sb = snackbar.clone();
                                                sb.show(SnackbarRequest {
                                                    message: "Title and URL are required"
                                                        .to_string(),
                                                    action: None,
                                                    duration_ms: 4000,
                                                    builder: Rc::new({
                                                        let sb = sb.clone();
                                                        move || {
                                                            material3::Snackbar(
                                                                "Title and URL are required",
                                                                Some(SnackbarAction {
                                                                    label: "Dismiss".to_string(),
                                                                    on_click: Rc::new({
                                                                        let sb = sb.clone();
                                                                        move || sb.dismiss()
                                                                    }),
                                                                }),
                                                                Modifier::new().absolute().offset(
                                                                    Some(16.0),
                                                                    None,
                                                                    Some(16.0),
                                                                    None,
                                                                ),
                                                            )
                                                        }
                                                    }),
                                                });
                                                return;
                                            }

                                            let Some(url) = normalize_url(&url_raw) else {
                                                let sb = snackbar.clone();
                                                sb.show(SnackbarRequest {
                                                    message: "Invalid URL format".to_string(),
                                                    action: None,
                                                    duration_ms: 4000,
                                                    builder: Rc::new({
                                                        let sb = sb.clone();
                                                        move || {
                                                            material3::Snackbar(
                                                                "Invalid URL format",
                                                                Some(SnackbarAction {
                                                                    label: "Dismiss".to_string(),
                                                                    on_click: Rc::new({
                                                                        let sb = sb.clone();
                                                                        move || sb.dismiss()
                                                                    }),
                                                                }),
                                                                Modifier::new().absolute().offset(
                                                                    Some(16.0),
                                                                    None,
                                                                    Some(16.0),
                                                                    None,
                                                                ),
                                                            )
                                                        }
                                                    }),
                                                });
                                                return;
                                            };

                                            bookmarks.update(|v| v.push(Bookmark { title, url }));
                                            storage::save_bookmarks(&bookmarks.get());

                                            new_title.set(String::new());
                                            new_url.set(String::new());
                                            form_epoch.update(|e| *e = e.wrapping_add(1));
                                            show_form.set(false);

                                            let sb = snackbar.clone();
                                            sb.show(SnackbarRequest {
                                                message: "Bookmark added".to_string(),
                                                action: None,
                                                duration_ms: 3000,
                                                builder: Rc::new({
                                                    let sb = sb.clone();
                                                    move || {
                                                        material3::Snackbar(
                                                            "Bookmark added",
                                                            Some(SnackbarAction {
                                                                label: "Dismiss".to_string(),
                                                                on_click: Rc::new({
                                                                    let sb = sb.clone();
                                                                    move || sb.dismiss()
                                                                }),
                                                            }),
                                                            Modifier::new().absolute().offset(
                                                                Some(16.0),
                                                                None,
                                                                Some(16.0),
                                                                None,
                                                            ),
                                                        )
                                                    }
                                                }),
                                            });
                                        }
                                    })
                                    .modifier(
                                        Modifier::new()
                                            .padding_values(PaddingValues {
                                                top: 12.0,
                                                ..Default::default()
                                            })
                                            .background(theme().primary)
                                            .clip_rounded(10.0),
                                    ),
                                )),
                            )
                        } else {
                            // Collapsed - just the + button
                            Button(
                                Text("+ Add bookmark")
                                    .size(14.0)
                                    .color(Color::from_hex("#6B7280")),
                                {
                                    let show = show_add_form.clone();
                                    move || show.set(true)
                                },
                            )
                            .modifier(
                                Modifier::new()
                                    .padding_values(PaddingValues {
                                        left: 16.0,
                                        right: 16.0,
                                        top: 10.0,
                                        bottom: 10.0,
                                    })
                                    .background(Color(0, 0, 0, 0))
                                    .clip_rounded(8.0)
                                    .border(1.0, theme().outline, 8.0),
                            )
                        }),
                    )),
                ),
            ),
        ),
    );

    overlay.host(Modifier::new().fill_max_size(), content)
}
