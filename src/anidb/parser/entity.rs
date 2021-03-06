/// Anime entity
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Anime {
    /// ID of the anime in AniDB database
    pub id: i32,
    /// Canonical title for the anime
    pub title: String,
    /// Non-canonical title for the anime
    pub variations: Vec<TitleVariation>,
}

impl Anime {
    pub fn new(id: i32, title: String, variations: Vec<TitleVariation>) -> Self {
        Anime {
            id,
            title,
            variations,
        }
    }
}

/// Non-canonical title for an anime entity
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TitleVariation {
    pub title: String,
    pub lang: String,
    pub kind: TitleKind,
}

impl TitleVariation {
    pub fn new(title: String, lang: String, kind: TitleKind) -> Self {
        TitleVariation { title, lang, kind }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TitleKind {
    /// Canonical title
    Main,
    /// As seen on official resources like Crunchyroll on theaters
    Official,
    /// "Also known as" title
    Synonym,
    /// Shorter title
    Short,
}
