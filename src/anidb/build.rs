use super::entity::*;

use std::convert::TryFrom;
use std::string::ToString;

/// Produces instances of `Anime` from parts
pub struct AnimeBuilder {
    id: Option<u32>,
    title: Option<String>,
    variations: Vec<TitleVariation>,

    /// `NameVariation` producer for current `Anime` instance
    variation_builder: Option<TitleVariationBuilder>,
}

impl AnimeBuilder {
    pub fn new() -> Self {
        AnimeBuilder {
            id: None,
            title: None,
            variations: vec![],
            variation_builder: None,
        }
    }

    /// Returns `Anime` instance from provided instance's data or `AnimeBuildError` if failed
    pub fn build(self) -> Result<Anime, AnimeBuildError> {
        let id = self.id.ok_or(AnimeBuildError::MissingId)?;
        let title = self.title.ok_or(AnimeBuildError::MissingTitle)?;

        let anime = Anime::new(id, title, self.variations);
        Ok(anime)
    }

    /// Returns `true` if contains some data but it's not enough to produce `Anime` instance
    pub fn is_started(&self) -> bool {
        self.id.is_some() || self.title.is_some()
    }

    /// Returns `true` if `Anime` instance can be produced
    pub fn is_complete(&self) -> bool {
        self.id.is_some() && self.title.is_some()
    }

    /// Returns `true` if is in process of aggregating data for anime title
    pub fn is_building_title(&self) -> bool {
        self.variation_builder.is_some()
    }

    /// Sets `id` for current `Anime` instance
    pub fn set_id(&mut self, id: u32) {
        self.id = Some(id);
    }

    /// Starts process of aggregating data for anime title. Returns `AnimeBuildError` in case if
    /// the process is already started
    pub fn start_title_building(&mut self) -> Result<(), AnimeBuildError> {
        if self.is_building_title() {
            return Err(AnimeBuildError::AlreadyStarted);
        }

        self.variation_builder = Some(TitleVariationBuilder::new());
        Ok(())
    }

    /// Sets anime title. Should be in a process of building the title,
    /// otherwise  `AnimeBuildError` will be returned
    pub fn set_title(&mut self, title: &str) -> Result<(), AnimeBuildError> {
        let builder = self
            .variation_builder
            .as_mut()
            .ok_or(AnimeBuildError::NotStarted)?;

        builder.set_title(title);
        Ok(())
    }

    /// Sets lang for the anime title. Should be in a process of building the title,
    /// otherwise  `AnimeBuildError` will be returned
    pub fn set_title_lang(&mut self, lang: &str) -> Result<(), AnimeBuildError> {
        let builder = self
            .variation_builder
            .as_mut()
            .ok_or(AnimeBuildError::NotStarted)?;

        builder.set_lang(lang);
        Ok(())
    }

    /// Sets kind for the anime title. Should be in a process of building the title,
    /// otherwise  `AnimeBuildError` will be returned
    pub fn set_title_kind(&mut self, kind: &str) -> Result<(), AnimeBuildError> {
        let builder = self
            .variation_builder
            .as_mut()
            .ok_or(AnimeBuildError::NotStarted)?;

        builder.set_kind(kind);
        Ok(())
    }

    /// Finishes aggregating data for the anime title. Title with kind `Main` will be uses as
    /// a canonical anime title. Returns `AnimeBuildError` if aggregation process was not started
    /// or if there is not enough data to build anime title
    pub fn finish_title_building(&mut self) -> Result<(), AnimeBuildError> {
        let builder = self
            .variation_builder
            .take()
            .ok_or(AnimeBuildError::NotStarted)?;

        let variation= builder.build()?;
        if variation.kind == TitleKind::Main {
            debug_assert_eq!(self.title, None);
            self.title = Some(variation.title.clone());
        }

        self.variations.push(variation);
        Ok(())
    }
}

pub enum AnimeBuildError {
    MissingId,
    MissingTitle,
    NotStarted,
    AlreadyStarted,
    MalformedTitle,
}

impl From<TitleVariationError> for AnimeBuildError {
    fn from(_: TitleVariationError) -> AnimeBuildError {
        AnimeBuildError::MalformedTitle
    }
}

/// Produces instances of `TitleVariation` from parts
struct TitleVariationBuilder {
    title: Option<String>,
    lang: Option<String>,
    kind: Option<TitleKind>,
}

impl TitleVariationBuilder {
    fn new() -> Self {
        TitleVariationBuilder {
            title: None,
            lang: None,
            kind: None,
        }
    }

    /// Returns `TitleVariation` from collected data or `TitleVariationError` if ther is
    /// not enough data
    fn build(self) -> Result<TitleVariation, TitleVariationError> {
        let title = self.title.ok_or(TitleVariationError::NameMissed)?;
        let lang = self.lang.ok_or(TitleVariationError::LangMissed)?;
        let kind = self.kind.ok_or(TitleVariationError::KindMissed)?;

        let variation = TitleVariation::new(title, lang, kind);
        Ok(variation)
    }

    fn set_title(&mut self, title: &str) {
        self.title = Some(title.to_string());
    }

    fn set_lang(&mut self, lang: &str) {
        self.lang = Some(lang.to_string());
    }

    fn set_kind(&mut self, kind: &str) -> Option<TitleVariationError> {
        match TitleKind::try_from(kind) {
            Ok(kind) => {
                self.kind = Some(kind);
                None
            },
            Err(err) => Some(err),
        }
    }
}

enum TitleVariationError {
    NameMissed,
    LangMissed,
    KindMissed,

    /// Unknown title kind. See `TitleKind` for supported values
    KindUnknown,
}

impl TitleKind {
    fn try_from(value: &str) -> Result<Self, TitleVariationError> {
        match value {
            "main" => Ok(TitleKind::Main),
            "official" => Ok(TitleKind::Official),
            "syn" => Ok(TitleKind::Synonym),
            "short" => Ok(TitleKind::Short),
            _ => Err(TitleVariationError::KindUnknown),
        }
    }
}
