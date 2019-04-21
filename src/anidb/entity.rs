/// Anime entity
pub struct Anime {
    /// ID of the anime in AniDB database
    pub id: u32,
    /// Canonical name for the anime
    pub name: String,
    /// Non-canonical names for the anime
    pub variations: Vec<NameVariation>,
}

impl Anime {
    pub fn new(id: u32, name: String, variations: Vec<NameVariation>) -> Self {
        AnimeTitle { id, name, variations }
    }
}

/// Non-canonical name for an anime title
pub struct NameVariation {
    pub name: String,
    pub lang: String,
    pub kind: NameKind,
}

impl NameVariation {
    pub fn new(name: String, lang: String, kind: NameKind) -> Self {
        NameVariation { name, lang, kind }
    }
}

#[derive(PartialEq)]
pub enum NameKind {
    /// Canonical name
    Main,
    /// As seen on official resources like Crunchyroll on theaters
    Official,
    /// "Also known as" name
    Synonym,
    /// Shorter name
    Short,
}
