use repose_core::prelude::*;
use repose_ui::*;

use crate::storage::{self, Bookmark};

fn open_url(url: &str) {
    let _ = web_sys::window().and_then(|w| w.location().set_href(url).ok());
}

fn search(query: &str) {
    let q = urlencoding::encode(query);
    open_url(&format!("https://duckduckgo.com/?q={q}"));
}

pub fn app(_s: &mut Scheduler) -> View {
    let bookmarks = remember(|| signal(storage::load_bookmarks()));
    let query = remember(|| signal(String::new()));
    let new_title = remember(|| signal(String::new()));
    let new_url = remember(|| signal(String::new()));

    let mut t = Theme::default();
    t.background = Color::from_hex("#0b0f14");
    t.surface = Color::from_hex("#111827");
    t.on_surface = Color::from_hex("#e5e7eb");
    t.primary = Color::from_hex("#3b82f6");
    t.on_primary = Color::from_hex("#ffffff");

    with_theme(t, || {
        Surface(
            Modifier::new()
                .fill_max_size()
                .background(theme().background),
            Column(Modifier::new().fill_max_size().padding(16.0)).child((
                Text("Startpage")
                    .size(28.0)
                    .color(theme().on_surface)
                    .modifier(Modifier::new().padding(8.0)),
                // Search row - single child, no tuple needed
                Row(Modifier::new().fill_max_width().padding(8.0)).child(TextField(
                    "Search…",
                    Modifier::new()
                        .height(40.0)
                        .fill_max_width()
                        .background(theme().surface)
                        .border(1.0, theme().outline, 10.0)
                        .padding(8.0),
                    Some({
                        let query = query.clone();
                        move |s: String| query.set(s)
                    }),
                    Some({
                        let query = query.clone();
                        move |_: String| search(&query.get())
                    }),
                )),
                // Add bookmark form
                Column(Modifier::new().fill_max_width().padding(8.0)).child((
                    Text("Add bookmark")
                        .size(16.0)
                        .color(theme().on_surface)
                        .modifier(Modifier::new().padding(4.0)),
                    Row(Modifier::new().fill_max_width()).child((
                        TextField(
                            "Title",
                            Modifier::new()
                                .height(36.0)
                                .fill_max_width()
                                .background(theme().surface)
                                .border(1.0, theme().outline, 8.0)
                                .padding(8.0),
                            Some({
                                let new_title = new_title.clone();
                                move |s: String| new_title.set(s)
                            }),
                            None::<fn(String)>, // Explicit type for None
                        ),
                        Box(Modifier::new().width(8.0).height(1.0)),
                        TextField(
                            "URL (https://…)",
                            Modifier::new()
                                .height(36.0)
                                .fill_max_width()
                                .background(theme().surface)
                                .border(1.0, theme().outline, 8.0)
                                .padding(8.0),
                            Some({
                                let new_url = new_url.clone();
                                move |s: String| new_url.set(s)
                            }),
                            None::<fn(String)>, // Explicit type for None
                        ),
                        Box(Modifier::new().width(8.0).height(1.0)),
                        Button(Text("Add"), {
                            let bookmarks = bookmarks.clone();
                            let new_title = new_title.clone();
                            let new_url = new_url.clone();
                            move || {
                                let title = new_title.get().trim().to_string();
                                let url = new_url.get().trim().to_string();
                                if title.is_empty() || url.is_empty() {
                                    return;
                                }
                                bookmarks.update(|v| v.push(Bookmark { title, url }));
                                storage::save_bookmarks(&bookmarks.get());
                                new_title.set(String::new());
                                new_url.set(String::new());
                            }
                        })
                        .modifier(
                            Modifier::new()
                                .padding(2.0)
                                .background(theme().primary)
                                .clip_rounded(8.0),
                        ),
                    )),
                )),
                // Bookmarks grid
                {
                    let items = bookmarks.get();
                    let cols = 4usize;

                    Grid(
                        cols,
                        Modifier::new().fill_max_width().padding(8.0),
                        items
                            .iter()
                            .enumerate()
                            .map(|(i, bm)| {
                                let bm = bm.clone();
                                let bookmarks = bookmarks.clone();

                                Box(Modifier::new()
                                    .background(theme().surface)
                                    .border(1.0, theme().outline, 12.0)
                                    .padding(12.0))
                                .child(
                                    Column(Modifier::new()).child((
                                        Button(Text(bm.title.clone()).size(16.0), {
                                            let url = bm.url.clone();
                                            move || open_url(&url)
                                        })
                                        .modifier(Modifier::new().padding(2.0)),
                                        Text(bm.url.clone())
                                            .size(12.0)
                                            .color(Color::from_hex("#9ca3af"))
                                            .modifier(Modifier::new().padding(2.0)),
                                        Spacer(),
                                        Button(Text("Remove").size(12.0), move || {
                                            bookmarks.update(|v| {
                                                if i < v.len() {
                                                    v.remove(i);
                                                }
                                            });
                                            storage::save_bookmarks(&bookmarks.get());
                                        })
                                        .modifier(
                                            Modifier::new()
                                                .padding(2.0)
                                                .background(Color::from_hex("#374151"))
                                                .clip_rounded(8.0),
                                        ),
                                    )),
                                )
                            })
                            .collect::<Vec<_>>(),
                        12.0,
                        12.0,
                    )
                },
            )),
        )
    })
}
