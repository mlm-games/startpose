use serde::{Deserialize, Serialize};

const KEY: &str = "startpage.bookmarks.v1";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bookmark {
    pub title: String,
    pub url: String,
}

fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

pub fn load_bookmarks() -> Vec<Bookmark> {
    let Some(st) = storage() else {
        return vec![];
    };
    let Ok(Some(raw)) = st.get_item(KEY) else {
        return vec![];
    };
    serde_json::from_str(&raw).unwrap_or_default()
}

pub fn save_bookmarks(items: &[Bookmark]) {
    let Some(st) = storage() else {
        return;
    };
    if let Ok(raw) = serde_json::to_string(items) {
        let _ = st.set_item(KEY, &raw);
    }
}
