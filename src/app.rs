#![allow(non_snake_case)]

use std::hash::{DefaultHasher, Hash, Hasher};
use std::rc::Rc;

use repose_core::{
    Color, ColorScheme, CursorIcon, Dp, FontWeight, PaddingValues, Shapes, Sp, Theme, Typography,
    prelude::*, set_theme_default,
};
use repose_material::material3::{
    self, ButtonConfig, CardConfig, ChipConfig, OutlinedTextFieldConfig, ScaffoldConfig,
    SnackbarConfig, SurfaceConfig,
};
use repose_material::{Icon, material_symbols};
use repose_ui::overlay::{OverlayHandle, SnackbarAction, SnackbarController, SnackbarRequest};
use repose_ui::scroll::{ScrollArea, remember_scroll_state};
use repose_ui::*;

use crate::storage::{self, Bookmark};

material_symbols! {
    SEARCH : '\u{E8B6}',
    ADD : '\u{E145}',
    CLOSE : '\u{E5CD}',
    DELETE : '\u{E872}',
    OPEN_IN_NEW : '\u{E89E}',
    BOOKMARK : '\u{E866}',
    DRAG_INDICATOR : '\u{E945}',
}

const PAGE_MAX: Dp = Dp(1180.0);
const GRID_GAP: Dp = Dp(12.0);
const SEARCH_PANEL_RADIUS: Dp = Dp(22.0);
const CARD_RADIUS: Dp = Dp(16.0);

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

fn stable_key(value: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

fn make_theme() -> Theme {
    Theme {
        colors: ColorScheme {
            background: Color::from_hex("#090D13"),
            surface: Color::from_hex("#0E151E"),
            surface_variant: Color::from_hex("#202D3B"),
            surface_container_lowest: Color::from_hex("#06090E"),
            surface_container_low: Color::from_hex("#0C131C"),
            surface_container: Color::from_hex("#111B26"),
            surface_container_high: Color::from_hex("#182534"),
            surface_container_highest: Color::from_hex("#223344"),
            surface_bright: Color::from_hex("#2A3E52"),
            surface_dim: Color::from_hex("#080C12"),
            surface_tint: Color::from_hex("#74E0BE"),
            on_surface: Color::from_hex("#E7EEF5"),
            on_surface_variant: Color::from_hex("#9BAAB9"),
            on_background: Color::from_hex("#E7EEF5"),
            primary: Color::from_hex("#74E0BE"),
            on_primary: Color::from_hex("#06231A"),
            primary_container: Color::from_hex("#163C35"),
            on_primary_container: Color::from_hex("#B5F4DC"),
            secondary: Color::from_hex("#7DB7FF"),
            on_secondary: Color::from_hex("#071B33"),
            secondary_container: Color::from_hex("#193650"),
            on_secondary_container: Color::from_hex("#C3DEFF"),
            tertiary: Color::from_hex("#C4A0FF"),
            on_tertiary: Color::from_hex("#24103F"),
            tertiary_container: Color::from_hex("#3A275B"),
            on_tertiary_container: Color::from_hex("#E6D7FF"),
            error: Color::from_hex("#FF9BA7"),
            on_error: Color::from_hex("#3A0710"),
            error_container: Color::from_hex("#661D2A"),
            on_error_container: Color::from_hex("#FFD9DE"),
            inverse_surface: Color::from_hex("#E7EEF5"),
            inverse_on_surface: Color::from_hex("#111923"),
            inverse_primary: Color::from_hex("#163C35"),
            outline: Color::from_hex("#506274"),
            outline_variant: Color::from_hex("#293A4A"),
            scrim: Color(0, 0, 0, 190),
            shadow: Color::BLACK,
            focus: Color::from_hex("#7DB7FF"),
        },
        typography: Typography {
            display_small: Sp(36.0),
            headline_large: Sp(30.0),
            headline_medium: Sp(26.0),
            headline_small: Sp(22.0),
            title_large: Sp(20.0),
            title_medium: Sp(16.0),
            title_small: Sp(13.0),
            body_large: Sp(15.0),
            body_medium: Sp(13.0),
            body_small: Sp(12.0),
            label_large: Sp(13.0),
            label_medium: Sp(12.0),
            label_small: Sp(10.0),
            ..Default::default()
        },
        shapes: Shapes {
            extra_small: Dp(8.0),
            small: Dp(12.0),
            medium: Dp(18.0),
            large: Dp(24.0),
            extra_large: Dp(32.0),
        },
        focus: Color::from_hex("#7DB7FF"),
        scrollbar_track: Color(255, 255, 255, 10),
        scrollbar_thumb: Color(157, 181, 204, 100),
        button_bg: Color::from_hex("#74E0BE"),
        button_bg_hover: Color::from_hex("#163C35"),
        button_bg_pressed: Color::from_hex("#224E43"),
        ..Default::default()
    }
}

fn primary_button_config() -> ButtonConfig {
    let th = theme();
    ButtonConfig {
        content_color: Some(th.on_primary),
        container_color: Some(th.primary),
        shape_radius: Dp(12.0),
        height: Dp(44.0),
        content_padding: Some(PaddingValues {
            left: Dp(16.0),
            right: Dp(16.0),
            top: Dp(8.0),
            bottom: Dp(8.0),
        }),
        ..Default::default()
    }
}

fn quiet_button_config() -> ButtonConfig {
    let th = theme();
    ButtonConfig {
        content_color: Some(th.on_surface_variant),
        container_color: Some(Color::TRANSPARENT),
        shape_radius: Dp(10.0),
        height: Dp(40.0),
        content_padding: Some(PaddingValues {
            left: Dp(12.0),
            right: Dp(12.0),
            top: Dp(6.0),
            bottom: Dp(6.0),
        }),
        ..Default::default()
    }
}

fn engine_chip_config() -> ChipConfig {
    let th = theme();
    let mut config = ChipConfig::default();
    config.colors.container_color = th.surface_container_high;
    config.colors.label_color = th.on_surface_variant;
    config.colors.leading_icon_color = th.on_surface_variant;
    config.colors.trailing_icon_color = th.on_surface_variant;
    config.colors.selected_container_color = th.primary.with_alpha_f32(0.22);
    config.colors.selected_label_color = th.primary;
    config.colors.selected_leading_icon_color = th.primary;
    config.colors.selected_trailing_icon_color = th.primary;
    config.border_color = th.outline_variant;
    config.selected_border_color = th.primary.with_alpha_f32(0.75);
    config.shape_radius = Dp(10.0);
    config.horizontal_padding = Dp(12.0);
    config
}

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
        builder: Rc::new(move |dismissing: bool| {
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
                    .offset(Some(Dp(16.0)), None, Some(Dp(16.0)), None),
                SnackbarConfig::default(),
                dismissing,
            )
        }),
    });
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

#[derive(Clone)]
struct BookmarkDrag {
    url: String,
}

fn move_bookmark(
    bookmarks: &Rc<Signal<Vec<Bookmark>>>,
    from_url: &str,
    to_url: &str,
    snackbar: &SnackbarController,
) -> bool {
    let mut items = bookmarks.get();
    let Some(from) = items.iter().position(|item| item.url == from_url) else {
        return false;
    };
    let Some(to) = items.iter().position(|item| item.url == to_url) else {
        return false;
    };
    if from == to {
        return false;
    }

    let item = items.remove(from);
    items.insert(to, item);
    bookmarks.set(items);
    storage::save_bookmarks(&bookmarks.get());
    show_snackbar(snackbar, "Bookmark moved", 2000);
    true
}

fn BookmarkCard(
    bm: Bookmark,
    bookmarks: Rc<Signal<Vec<Bookmark>>>,
    snackbar: Rc<SnackbarController>,
    dragging: Rc<Signal<Option<String>>>,
    drop_target: Rc<Signal<Option<String>>>,
) -> View {
    let th = theme();
    let url_open = bm.url.clone();
    let url_remove = bm.url.clone();
    let target_url = bm.url.clone();
    let title = bm.title.clone();
    let display_url = truncate_url(&bm.url);
    let is_dragging = dragging.get().as_deref() == Some(target_url.as_str());
    let is_drop_target = drop_target.get().as_deref() == Some(target_url.as_str());

    let drag_handle = Box(Modifier::new()
        .size(Dp(24.0), Dp(36.0))
        .flex_shrink(0.0)
        .cursor(CursorIcon::Grab)
        .drag_preview_label(title.clone(), th.primary)
        .on_drag_start({
            let payload = BookmarkDrag {
                url: target_url.clone(),
            };
            let dragging = dragging.clone();
            move |_| {
                dragging.set(Some(payload.url.clone()));
                Some(drag_payload(payload.clone()))
            }
        })
        .on_drag_end({
            let dragging = dragging.clone();
            let drop_target = drop_target.clone();
            move |_| {
                dragging.set(None);
                drop_target.set(None);
            }
        })
        .semantics(Semantics::new(Role::Container).with_label("Reorder bookmark")))
    .child(
        Icon(Symbols::DRAG_INDICATOR)
            .size(Sp(18.0))
            .color(th.on_surface_variant),
    );

    let content = Row(Modifier::new()
        .fill_max_width()
        .padding_values(PaddingValues {
            left: Dp(10.0),
            right: Dp(12.0),
            top: Dp(14.0),
            bottom: Dp(14.0),
        })
        .align_items(AlignItems::CENTER)
        .gap(Dp(8.0)))
    .child((
        drag_handle,
        Column(Modifier::new().weight(1.0).min_width(Dp(0.0)).gap(Dp(4.0))).child((
            Row(Modifier::new().align_items(AlignItems::CENTER).gap(Dp(6.0))).child((
                Icon(Symbols::OPEN_IN_NEW)
                    .size(Sp(15.0))
                    .color(th.on_surface_variant),
                Text(title)
                    .size(Sp(15.0))
                    .font_weight(FontWeight::SEMI_BOLD)
                    .single_line()
                    .overflow_ellipsize()
                    .color(th.on_surface),
            )),
            Text(display_url)
                .font_family("monospace")
                .size(Sp(11.0))
                .single_line()
                .overflow_ellipsize()
                .color(th.on_surface_variant),
        )),
        material3::TextButton(
            Modifier::new().flex_shrink(0.0),
            {
                let bms = bookmarks.clone();
                let snackbar = snackbar.clone();
                move || {
                    bms.update(|v| v.retain(|b| b.url != url_remove));
                    storage::save_bookmarks(&bms.get());
                    show_snackbar(&snackbar, "Bookmark removed", 3000);
                }
            },
            quiet_button_config(),
            || {
                Row(Modifier::new().align_items(AlignItems::CENTER).gap(Dp(6.0)))
                    .child((Icon(Symbols::DELETE).size(Sp(16.0)), Text("Remove")))
            },
        ),
    ));

    let card_modifier = Modifier::new()
        .fill_max_width()
        .key(stable_key(&bm.url))
        .cursor(CursorIcon::Pointer)
        .on_drag_enter({
            let target_url = target_url.clone();
            let drop_target = drop_target.clone();
            move |event| {
                if event.payload.as_ref().is::<BookmarkDrag>() {
                    drop_target.set(Some(target_url.clone()));
                }
            }
        })
        .on_drag_over({
            let target_url = target_url.clone();
            let drop_target = drop_target.clone();
            move |event| {
                if event.payload.as_ref().is::<BookmarkDrag>() {
                    drop_target.set(Some(target_url.clone()));
                }
            }
        })
        .on_drag_leave({
            let target_url = target_url.clone();
            let drop_target = drop_target.clone();
            move |event| {
                if event.payload.as_ref().is::<BookmarkDrag>()
                    && drop_target.get().as_deref() == Some(target_url.as_str())
                {
                    drop_target.set(None);
                }
            }
        })
        .on_drop({
            let target_url = target_url.clone();
            let bookmarks = bookmarks.clone();
            let snackbar = snackbar.clone();
            let dragging = dragging.clone();
            let drop_target = drop_target.clone();
            move |event| {
                let Some(payload) = event.payload.as_ref().downcast_ref::<BookmarkDrag>() else {
                    return false;
                };
                let from_url = payload.url.clone();
                let accepted = if from_url == target_url {
                    true
                } else {
                    move_bookmark(&bookmarks, &from_url, &target_url, &snackbar)
                };
                drop_target.set(None);
                dragging.set(None);
                accepted
            }
        });

    let border_color = if is_drop_target {
        th.primary
    } else if is_dragging {
        th.primary.with_alpha_f32(0.65)
    } else {
        th.outline_variant
    };

    material3::ClickableCard(
        move || open_url(&url_open),
        card_modifier,
        CardConfig {
            modifier: Modifier::new(),
            enabled: true,
            container_color: th.surface_container_low,
            content_color: th.on_surface,
            disabled_container_color: th.surface_container_low,
            disabled_content_color: th.on_surface.with_alpha_f32(0.38),
            shape_radius: CARD_RADIUS,
            tonal_elevation: Dp::ZERO,
            state_elevation: Some(StateElevation {
                default: Dp::ZERO,
                hovered: Dp(1.0),
                focused: Dp(1.0),
                pressed: Dp::ZERO,
                dragged: Dp(2.0),
                disabled: Dp::ZERO,
            }),
            border: Some((Dp(1.0), border_color)),
            interaction_source: None,
        },
        || content,
    )
}

fn AddBookmarkForm(
    bookmarks: Rc<Signal<Vec<Bookmark>>>,
    new_title: Rc<Signal<String>>,
    new_url: Rc<Signal<String>>,
    snackbar: Rc<SnackbarController>,
    on_dismiss: impl Fn() + 'static + Clone,
) -> View {
    let th = theme();
    let dismiss_submit = on_dismiss.clone();
    let submit_bookmarks = bookmarks.clone();
    let submit_title = new_title.clone();
    let submit_url = new_url.clone();
    let submit_snackbar = snackbar.clone();

    let submit = move || {
        let title = submit_title.get().trim().to_string();
        let url_raw = submit_url.get().trim().to_string();

        if title.is_empty() || url_raw.is_empty() {
            show_snackbar(&submit_snackbar, "Title and URL are required", 4000);
            return;
        }
        let Some(url) = normalize_url(&url_raw) else {
            show_snackbar(&submit_snackbar, "Enter a valid URL", 4000);
            return;
        };
        if submit_bookmarks
            .get()
            .iter()
            .any(|bookmark| bookmark.url == url)
        {
            show_snackbar(&submit_snackbar, "That bookmark is already saved", 3500);
            return;
        }

        submit_bookmarks.update(|items| items.push(Bookmark { title, url }));
        storage::save_bookmarks(&submit_bookmarks.get());
        submit_title.set(String::new());
        submit_url.set(String::new());
        dismiss_submit();
        show_snackbar(&submit_snackbar, "Bookmark added", 3000);
    };

    material3::Surface(
        SurfaceConfig {
            modifier: Modifier::new().fill_max_width(),
            enabled: true,
            color: th.surface_container,
            content_color: th.on_surface,
            shape_radius: Dp(18.0),
            tonal_elevation: Dp::ZERO,
            shadow_elevation: Dp::ZERO,
            border: Some((Dp(1.0), th.outline_variant)),
            interaction_source: None,
        },
        || {
            Column(
                Modifier::new()
                    .fill_max_width()
                    .padding_values(PaddingValues {
                        left: Dp(4.0),
                        right: Dp(4.0),
                        top: Dp(20.0),
                        bottom: Dp(20.0),
                    })
                    .gap(Dp(14.0)),
            )
            .child((
                Text("Add bookmark")
                    .size(Sp(20.0))
                    .font_weight(FontWeight::SEMI_BOLD)
                    .color(th.on_surface),
                material3::OutlinedTextField(
                    Modifier::new().fill_max_width(),
                    new_title.get(),
                    {
                        let new_title = new_title.clone();
                        move |value| new_title.set(value)
                    },
                    OutlinedTextFieldConfig {
                        label: Some("Title".to_string()),
                        placeholder: Some("GitHub".to_string()),
                        single_line: true,
                        ..Default::default()
                    },
                ),
                material3::OutlinedTextField(
                    Modifier::new().fill_max_width(),
                    new_url.get(),
                    {
                        let new_url = new_url.clone();
                        move |value| new_url.set(value)
                    },
                    OutlinedTextFieldConfig {
                        label: Some("URL".to_string()),
                        placeholder: Some("example.com".to_string()),
                        single_line: true,
                        ..Default::default()
                    },
                ),
                Row(Modifier::new()
                    .fill_max_width()
                    .justify_content(JustifyContent::FLEX_END))
                .child(material3::Button(
                    Modifier::new(),
                    submit,
                    primary_button_config(),
                    || Text("Add bookmark"),
                )),
            ))
        },
    )
}

fn EmptyState() -> View {
    let th = theme();
    material3::Surface(
        SurfaceConfig {
            modifier: Modifier::new().fill_max_width(),
            enabled: true,
            color: th.surface_container_low,
            content_color: th.on_surface,
            shape_radius: Dp(18.0),
            tonal_elevation: Dp::ZERO,
            shadow_elevation: Dp::ZERO,
            border: Some((Dp(1.0), th.outline_variant)),
            interaction_source: None,
        },
        || {
            Column(
                Modifier::new()
                    .fill_max_width()
                    .padding_values(PaddingValues {
                        left: Dp(20.0),
                        right: Dp(20.0),
                        top: Dp(28.0),
                        bottom: Dp(28.0),
                    })
                    .align_items(AlignItems::CENTER),
            )
            .child(
                Text("No bookmarks")
                    .size(Sp(20.0))
                    .font_weight(FontWeight::SEMI_BOLD)
                    .color(th.on_surface),
            )
        },
    )
}

fn AppHeader(show_add: Rc<Signal<bool>>, compact: bool) -> View {
    let form_open = show_add.get();
    let action_label = if form_open {
        "Cancel"
    } else if compact {
        "Add"
    } else {
        "Add bookmark"
    };

    Row(Modifier::new()
        .fill_max_width()
        .align_items(AlignItems::CENTER)
        .gap(Dp(12.0)))
    .child((
        Text("Startpage")
            .size(Sp(26.0))
            .font_weight(FontWeight::SEMI_BOLD)
            .color(theme().on_surface)
            .modifier(Modifier::new().weight(1.0).min_width(Dp(0.0))),
        material3::Button(
            Modifier::new().flex_shrink(0.0),
            {
                let show_add = show_add.clone();
                move || show_add.update(|open| *open = !*open)
            },
            primary_button_config(),
            || {
                Row(Modifier::new().align_items(AlignItems::CENTER).gap(Dp(7.0))).child((
                    Icon(if form_open {
                        Symbols::CLOSE
                    } else {
                        Symbols::ADD
                    })
                    .size(Sp(17.0)),
                    Text(action_label),
                ))
            },
        ),
    ))
}

fn SearchPanel(query: Rc<Signal<String>>, engine: Rc<Signal<SearchEngine>>, compact: bool) -> View {
    let th = theme();
    let horizontal_padding = if compact { Dp(4.0) } else { Dp(20.0) };
    let engine_chips = SearchEngine::ALL
        .iter()
        .map(|selected| {
            let engine = engine.clone();
            let selected = *selected;
            material3::FilterChip(
                engine.get() == selected,
                move || engine.set(selected),
                Text(selected.label()),
                None,
                None,
                engine_chip_config(),
            )
        })
        .collect::<Vec<_>>();

    let search_field = material3::OutlinedTextField(
        Modifier::new().fill_max_width().height(Dp(60.0)),
        query.get(),
        {
            let query = query.clone();
            move |value| query.set(value)
        },
        OutlinedTextFieldConfig {
            placeholder: Some("Search or paste a URL".to_string()),
            leading_icon: Some(Icon(Symbols::SEARCH).size(Sp(20.0))),
            single_line: true,
            on_submit: Some(Rc::new({
                let engine = engine.clone();
                move |submitted| search_or_open(engine.get(), &submitted)
            })),
            ..Default::default()
        },
    );

    Box(Modifier::new()
        .fill_max_width()
        .clip_rounded(SEARCH_PANEL_RADIUS)
        .background(th.surface_container)
        .border(Dp(1.0), th.outline_variant, SEARCH_PANEL_RADIUS)
        .padding_values(PaddingValues {
            left: horizontal_padding,
            right: horizontal_padding,
            top: Dp(16.0),
            bottom: Dp(16.0),
        }))
    .child(
        Column(Modifier::new().fill_max_width().gap(Dp(12.0))).child((
            search_field,
            material3::FilterChipGroup(Modifier::new().fill_max_width(), engine_chips),
        )),
    )
}

fn BookmarkSection(
    bookmarks: Rc<Signal<Vec<Bookmark>>>,
    snackbar: Rc<SnackbarController>,
    dragging: Rc<Signal<Option<String>>>,
    drop_target: Rc<Signal<Option<String>>>,
    columns: usize,
) -> View {
    let th = theme();
    let items = bookmarks.get();
    let count = items.len();
    let cards = items
        .into_iter()
        .map(|bookmark| {
            BookmarkCard(
                bookmark,
                bookmarks.clone(),
                snackbar.clone(),
                dragging.clone(),
                drop_target.clone(),
            )
        })
        .collect::<Vec<_>>();

    let count_label = format!("{} bookmark{}", count, if count == 1 { "" } else { "s" });

    Column(Modifier::new().fill_max_width().gap(Dp(14.0))).child((
        Row(Modifier::new()
            .fill_max_width()
            .align_items(AlignItems::CENTER))
        .child((
            Row(Modifier::new().align_items(AlignItems::CENTER).gap(Dp(8.0))).child((
                Icon(Symbols::BOOKMARK).size(Sp(22.0)).color(th.primary),
                Text("Bookmarks")
                    .size(Sp(24.0))
                    .font_weight(FontWeight::SEMI_BOLD)
                    .color(th.on_surface),
            )),
            Box(Modifier::new().weight(1.0).height(Dp(1.0))),
            Text(count_label)
                .size(Sp(12.0))
                .color(th.on_surface_variant),
        )),
        if count == 0 {
            EmptyState()
        } else {
            Grid(
                columns,
                Modifier::new().fill_max_width(),
                cards,
                GRID_GAP,
                GRID_GAP,
            )
        },
    ))
}

fn vertical_space(height: Dp) -> View {
    Box(Modifier::new().width(Dp(1.0)).height(height))
}

pub fn app(s: &mut Scheduler) -> View {
    set_theme_default(make_theme());

    let bookmarks = remember(|| signal(storage::load_bookmarks()));
    let query = remember(|| signal(String::new()));
    let engine = remember(|| signal(SearchEngine::DuckDuckGo));
    let new_title = remember(|| signal(String::new()));
    let new_url = remember(|| signal(String::new()));
    let show_add = remember(|| signal(false));
    let dragging_bookmark = remember(|| signal(None::<String>));
    let drop_target = remember(|| signal(None::<String>));
    let root_scroll = remember_scroll_state("root_scroll");

    let overlay = remember(OverlayHandle::new);
    let snackbar = remember(|| SnackbarController::new((*overlay).clone()));

    let px_w = s.size.0 as f32;
    let scale = repose_core::locals::density().scale * repose_core::locals::ui_scale().0;
    let dp_w = if scale > 0.0 { px_w / scale } else { px_w };
    let compact = dp_w < 600.0;
    let columns = if dp_w < 560.0 {
        1
    } else if dp_w < 900.0 {
        2
    } else {
        3
    };
    let page_padding = if compact {
        PaddingValues {
            left: Dp(12.0),
            right: Dp(12.0),
            top: Dp(20.0),
            bottom: Dp(40.0),
        }
    } else {
        PaddingValues {
            left: Dp(32.0),
            right: Dp(32.0),
            top: Dp(32.0),
            bottom: Dp(56.0),
        }
    };

    let bms = bookmarks.clone();
    let q = query.clone();
    let eng = engine.clone();
    let nt = new_title.clone();
    let nu = new_url.clone();
    let show = show_add.clone();
    let sb = snackbar.clone();
    let drag_state = dragging_bookmark.clone();
    let drop_state = drop_target.clone();
    let rs = root_scroll.clone();
    let th = theme();

    let content = material3::Scaffold(
        move |padding| {
            let bms = bms.clone();
            let q = q.clone();
            let eng = eng.clone();
            let nt = nt.clone();
            let nu = nu.clone();
            let show = show.clone();
            let sb = sb.clone();
            let drag_state = drag_state.clone();
            let drop_state = drop_state.clone();
            let rs = rs.clone();
            let th = theme();

            let page = Column(
                Modifier::new()
                    .fill_max_width()
                    .padding_values(page_padding)
                    .align_items(AlignItems::CENTER),
            )
            .child(
                Box(Modifier::new()
                    .fill_max_width()
                    .max_width(PAGE_MAX)
                    .min_width(Dp(0.0)))
                .child(Column(Modifier::new().fill_max_width()).child(vec![
                    AppHeader(show.clone(), compact),
                    vertical_space(if compact { Dp(24.0) } else { Dp(32.0) }),
                    SearchPanel(q.clone(), eng.clone(), compact),
                    if show.get() {
                        vertical_space(Dp(18.0))
                    } else {
                        vertical_space(Dp(2.0))
                    },
                    if show.get() {
                        AddBookmarkForm(bms.clone(), nt.clone(), nu.clone(), sb.clone(), {
                            let show = show.clone();
                            move || show.set(false)
                        })
                    } else {
                        Box(Modifier::new())
                    },
                    vertical_space(if compact { Dp(30.0) } else { Dp(40.0) }),
                    BookmarkSection(
                        bms.clone(),
                        sb.clone(),
                        drag_state.clone(),
                        drop_state.clone(),
                        columns,
                    ),
                ])),
            );

            material3::Surface(
                SurfaceConfig {
                    modifier: Modifier::new().fill_max_size().padding_values(padding),
                    color: th.background,
                    content_color: th.on_background,
                    ..Default::default()
                },
                || ScrollArea(Modifier::new().fill_max_size(), rs, page),
            )
        },
        ScaffoldConfig {
            container_color: th.background,
            content_color: th.on_background,
            ..Default::default()
        },
    );

    overlay.host(Modifier::new().fill_max_size(), content)
}
