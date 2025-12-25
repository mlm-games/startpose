#![allow(non_snake_case)]

use repose_core::locals::set_theme_default;
use repose_core::{PaddingValues, prelude::*};
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

fn Pill(label: &str, selected: bool, on_click: impl Fn() + 'static) -> View {
    let bg = if selected {
        Color(theme().primary.0, theme().primary.1, theme().primary.2, 48)
    } else {
        theme().button_bg
    };

    Button(
        Text(label)
            .size(14.0)
            .single_line()
            .overflow_ellipsize()
            .color(theme().on_surface),
        on_click,
    )
    .modifier(
        Modifier::new()
            .padding_values(PaddingValues {
                left: 12.0,
                right: 12.0,
                top: 8.0,
                bottom: 8.0,
            })
            .background(bg)
            .border(1.0, theme().outline, 999.0)
            .clip_rounded(999.0),
    )
}

fn Primary(label: &str, on_click: impl Fn() + 'static) -> View {
    Button(
        Text(label)
            .single_line()
            .overflow_ellipsize()
            .color(theme().on_primary),
        on_click,
    )
    .modifier(
        Modifier::new()
            .padding_values(PaddingValues {
                left: 14.0,
                right: 14.0,
                top: 10.0,
                bottom: 10.0,
            })
            .background(theme().primary)
            .clip_rounded(10.0),
    )
}

fn Card(modifier: Modifier, content: View) -> View {
    Box(modifier
        .background(theme().surface)
        .border(1.0, theme().outline, 14.0)
        .clip_rounded(14.0)
        .padding(14.0))
    .child(content)
}

fn hash64(s: &str) -> u64 {
    // simple stable hash for keys
    let mut x: u64 = 14695981039346656037;
    for &b in s.as_bytes() {
        x ^= b as u64;
        x = x.wrapping_mul(1099511628211);
    }
    x
}

pub fn app(s: &mut Scheduler) -> View {
    set_theme_default(theme_pro());

    let bookmarks = remember(|| signal(storage::load_bookmarks()));
    let toast = remember(|| signal(None::<String>));

    let query = remember(|| signal(String::new()));
    let engine = remember(|| signal(SearchEngine::DuckDuckGo));

    let new_title = remember(|| signal(String::new()));
    let new_url = remember(|| signal(String::new()));

    // TextField reset epoch (TextField is platform-managed)
    let form_epoch = remember(|| signal(0u64));

    let root_scroll = remember_scroll_state("root_scroll");

    // Responsive columns based on window width
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

    Surface(
        Modifier::new()
            .fill_max_size()
            .background(theme().background),
        ScrollArea(
            Modifier::new().fill_max_size(),
            root_scroll,
            Column(Modifier::new().fill_max_width().padding(16.0)).child((
                Box(Modifier::new()
                    .fill_max_width()
                    .max_width(1100.0)
                    .align_self_center()
                    .min_width(0.0))
                .child(
                    Column(Modifier::new().fill_max_width()).child((
                        // Header
                        Row(Modifier::new()
                            .fill_max_width()
                            .align_items(AlignItems::Center)
                            .padding_values(PaddingValues {
                                bottom: 12.0,
                                ..Default::default()
                            }))
                        .child((
                            Text("Startpage")
                                .size(28.0)
                                .single_line()
                                .overflow_ellipsize()
                                .color(theme().on_surface)
                                .modifier(Modifier::new().weight(1.0).min_width(0.0)),
                            Box(Modifier::new()), // Text(format!("{} bookmarks", bookmarks.get().len()))
                                                  //     .size(14.0)
                                                  //     .single_line()
                                                  //     .color(Color::from_hex("#9CA3AF")),
                        )),
                        if let Some(msg) = toast.get() {
                            Box(Modifier::new()
                                .fill_max_width()
                                .padding_values(PaddingValues {
                                    bottom: 12.0,
                                    ..Default::default()
                                })
                                .background(Color::from_hex("#0F172A"))
                                .border(1.0, theme().outline, 12.0)
                                .clip_rounded(12.0)
                                .padding(12.0))
                            .child(Text(msg).size(14.0).color(theme().on_surface))
                        } else {
                            Box(Modifier::new())
                        },
                        // Search card with Search button
                        Card(
                            Modifier::new().fill_max_width(),
                            Column(Modifier::new().fill_max_width()).child((
                                Row(Modifier::new()
                                    .fill_max_width()
                                    .align_items(AlignItems::Center))
                                .child((
                                    Text("Search")
                                        .size(14.0)
                                        .single_line()
                                        .color(Color::from_hex("#9CA3AF"))
                                        .modifier(
                                            Modifier::new()
                                                .weight(1.0)
                                                .min_width(0.0)
                                                .fill_max_width(),
                                        ),
                                    Pill("Clear", false, {
                                        let toast = toast.clone();
                                        let query = query.clone();
                                        move || {
                                            toast.set(None);
                                            query.set(String::new());
                                        }
                                    }),
                                )),
                                Box(Modifier::new().height(8.0).width(1.0)),
                                Row(Modifier::new()
                                    .fill_max_width()
                                    .align_items(AlignItems::Center))
                                .child((
                                    TextField(
                                        "Search or type a URL…",
                                        Modifier::new()
                                            .key(0xA11CE_u64)
                                            .height(48.0)
                                            .weight(1.0)
                                            .min_width(0.0)
                                            .background(Color::from_hex("#0F172A"))
                                            .border(1.0, theme().outline, 12.0)
                                            .clip_rounded(12.0),
                                        Some({
                                            let query = query.clone();
                                            move |s| query.set(s)
                                        }),
                                        Some({
                                            let engine = engine.clone();
                                            move |submitted: String| {
                                                search_or_open(engine.get(), &submitted.to_string())
                                            }
                                        }),
                                    ),
                                    Box(Modifier::new().width(10.0).height(1.0)),
                                    Primary("Search", {
                                        let q = query.clone();
                                        let engine = engine.clone();
                                        move || search_or_open(engine.get(), &q.get())
                                    }),
                                )),
                                Box(Modifier::new().height(10.0).width(1.0)),
                                Row(Modifier::new().fill_max_width().flex_wrap(FlexWrap::Wrap))
                                    .child((
                                        Pill(
                                            SearchEngine::DuckDuckGo.label(),
                                            engine.get() == SearchEngine::DuckDuckGo,
                                            {
                                                let engine = engine.clone();
                                                move || engine.set(SearchEngine::DuckDuckGo)
                                            },
                                        )
                                        .modifier(Modifier::new().padding(4.0)),
                                        Pill(
                                            SearchEngine::Google.label(),
                                            engine.get() == SearchEngine::Google,
                                            {
                                                let engine = engine.clone();
                                                move || engine.set(SearchEngine::Google)
                                            },
                                        )
                                        .modifier(Modifier::new().padding(4.0)),
                                        Pill(
                                            SearchEngine::Brave.label(),
                                            engine.get() == SearchEngine::Brave,
                                            {
                                                let engine = engine.clone();
                                                move || engine.set(SearchEngine::Brave)
                                            },
                                        )
                                        .modifier(Modifier::new().padding(4.0)),
                                    )),
                            )),
                        ),
                        Box(Modifier::new().height(16.0).width(1.0)),
                        // Add bookmark
                        Card(
                            Modifier::new().fill_max_width(),
                            Column(Modifier::new().fill_max_width()).child((
                                Row(Modifier::new()
                                    .fill_max_width()
                                    .align_items(AlignItems::Center))
                                .child((
                                    Text("Add bookmark")
                                        .size(16.0)
                                        .single_line()
                                        .color(theme().on_surface),
                                    Spacer(),
                                    Primary("Add", {
                                        let bookmarks = bookmarks.clone();
                                        let new_title = new_title.clone();
                                        let new_url = new_url.clone();
                                        let toast = toast.clone();
                                        let form_epoch = form_epoch.clone();

                                        move || {
                                            let title = new_title.get().trim().to_string();
                                            let url_raw = new_url.get().trim().to_string();

                                            if title.is_empty() || url_raw.is_empty() {
                                                toast.set(Some(
                                                    "Title and URL are required.".into(),
                                                ));
                                                return;
                                            }

                                            let Some(url) = normalize_url(&url_raw) else {
                                                toast.set(Some(
                                                    "URL must be a domain or start with http(s)://"
                                                        .into(),
                                                ));
                                                return;
                                            };

                                            bookmarks.update(|v| v.push(Bookmark { title, url }));
                                            storage::save_bookmarks(&bookmarks.get());

                                            new_title.set(String::new());
                                            new_url.set(String::new());
                                            form_epoch.update(|e| *e = e.wrapping_add(1));

                                            toast.set(Some("Bookmark added.".into()));
                                        }
                                    }),
                                )),
                                Box(Modifier::new().height(12.0).width(1.0)),
                                Row(Modifier::new().fill_max_width()).child((
                                    TextField(
                                        "Title",
                                        Modifier::new()
                                            .key(hash64("title") ^ form_epoch.get())
                                            .height(40.0)
                                            .weight(1.0)
                                            .min_width(0.0)
                                            .background(Color::from_hex("#0F172A"))
                                            .border(1.0, theme().outline, 12.0)
                                            .clip_rounded(12.0),
                                        Some({
                                            let new_title = new_title.clone();
                                            move |s| new_title.set(s)
                                        }),
                                        None::<fn(String)>,
                                    ),
                                    Box(Modifier::new().width(10.0).height(1.0)),
                                    TextField(
                                        "URL (example.com)",
                                        Modifier::new()
                                            .key(hash64("url") ^ form_epoch.get())
                                            .height(40.0)
                                            .weight(2.0)
                                            .min_width(0.0)
                                            .background(Color::from_hex("#0F172A"))
                                            .border(1.0, theme().outline, 12.0)
                                            .clip_rounded(12.0),
                                        Some({
                                            let new_url = new_url.clone();
                                            move |s| new_url.set(s)
                                        }),
                                        None::<fn(String)>,
                                    ),
                                )),
                            )),
                        ),
                        Box(Modifier::new().height(16.0).width(1.0)),
                        // Bookmarks grid (cleaner formatting)
                        Card(
                            Modifier::new().fill_max_width(),
                            Column(Modifier::new().fill_max_width()).child((
                                Row(Modifier::new()
                                    .fill_max_width()
                                    .align_items(AlignItems::Center))
                                .child((
                                    Text("Bookmarks")
                                        .size(16.0)
                                        .single_line()
                                        .color(theme().on_surface),
                                    Spacer(),
                                    Pill("Dismiss", false, {
                                        let toast = toast.clone();
                                        move || toast.set(None)
                                    }),
                                )),
                                Box(Modifier::new().height(12.0).width(1.0)),
                                Grid(
                                    cols,
                                    Modifier::new().fill_max_width(),
                                    bookmarks
                                        .get()
                                        .iter()
                                        .map(|bm| {
                                            let bm = bm.clone();
                                            let bms = bookmarks.clone();
                                            let toast = toast.clone();
                                            let card_key = hash64(&bm.url);

                                            Box(Modifier::new().key(card_key).fill_max_width())
                                                .child(Card(
                                                    Modifier::new().fill_max_width(),
                                                    Column(Modifier::new().fill_max_width()).child(
                                                        (
                                                            Text(bm.title.clone())
                                                                .size(16.0)
                                                                .single_line()
                                                                .overflow_ellipsize()
                                                                .color(theme().on_surface)
                                                                .modifier(
                                                                    Modifier::new()
                                                                        .fill_max_width(),
                                                                ),
                                                            Box(Modifier::new()
                                                                .height(6.0)
                                                                .width(1.0)),
                                                            Text(bm.url.clone())
                                                                .size(12.0)
                                                                .single_line()
                                                                .overflow_ellipsize()
                                                                .color(Color::from_hex("#9CA3AF"))
                                                                .modifier(
                                                                    Modifier::new()
                                                                        .fill_max_width(),
                                                                ),
                                                            Box(Modifier::new()
                                                                .height(12.0)
                                                                .width(1.0)),
                                                            Row(Modifier::new()
                                                                .fill_max_width()
                                                                .align_items(AlignItems::Center))
                                                            .child((
                                                                Primary("Open", {
                                                                    let url = bm.url.clone();
                                                                    move || open_url(&url)
                                                                }),
                                                                Spacer(),
                                                                Pill("Remove", false, move || {
                                                                    bms.update(|v| {
                                                                        if let Some(pos) =
                                                                            v.iter().position(|x| {
                                                                                x.url == bm.url
                                                                            })
                                                                        {
                                                                            v.remove(pos);
                                                                        }
                                                                    });
                                                                    storage::save_bookmarks(
                                                                        &bms.get(),
                                                                    );
                                                                    toast.set(Some(
                                                                        "Bookmark removed.".into(),
                                                                    ));
                                                                }),
                                                            )),
                                                        ),
                                                    ),
                                                ))
                                        })
                                        .collect::<Vec<_>>(),
                                    12.0,
                                    12.0,
                                ),
                            )),
                        ),
                    )),
                ),
                Box(Modifier::new()),
            )),
        ),
    )
}
