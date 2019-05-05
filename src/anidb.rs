mod build;
mod entity;

use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::str::FromStr;

use std::num::ParseIntError;
use std::str::Utf8Error;

use quick_xml::events::BytesStart;
use quick_xml::events::BytesText;
use quick_xml::events::Event;
use quick_xml::Reader;

use quick_xml::Error as QXError;

use build::AnimeBuildError;
use build::AnimeBuilder;

pub use entity::Anime;
pub use entity::TitleVariation;

/// AniDB dumb parser
pub struct AniDb {
    reader: Reader<BufReader<File>>,
    buffer: Vec<u8>,
}

impl AniDb {
    /// Returns parser for file at `path` or `XmlError` if file doesn't contain valid xml
    pub fn new(path: &Path) -> Result<Self, XmlError> {
        let reader = Reader::from_file(path)?;
        let buffer = Vec::with_capacity(1024);

        Ok(AniDb { reader, buffer })
    }
}

// TODO: add logging
impl Iterator for AniDb {
    type Item = Anime;

    fn next(&mut self) -> Option<Anime> {
        let mut builder = AnimeBuilder::new();

        loop {
            match self.reader.read_event(&mut self.buffer) {
                Ok(Event::Start(ref tag)) if tag.name() == b"anime" => {
                    if let Err(e) = builder.handle_id(&tag) {
                        eprintln!("failed to parse title entry: {:?}", e);
                        continue;
                    }
                }
                Ok(Event::Start(ref tag)) if tag.name() == b"title" => {
                    if let Err(e) = builder.handle_title_start(tag) {
                        eprintln!("failed to parse title tag: {:?}", e);
                        continue;
                    }
                }
                Ok(Event::Text(ref text)) if builder.is_building_title() => {
                    if let Err(e) = builder.handle_title(text) {
                        eprintln!("failed to parse title: {:?}", e);
                        continue;
                    }
                }
                Ok(Event::End(ref tag)) if tag.name() == b"title" => {
                    if let Err(e) = builder.handle_title_end() {
                        eprintln!("failed to parse title tag: {:?}", e);
                        continue;
                    }
                }
                Ok(Event::End(ref tag)) if tag.name() == b"anime" => {
                    // if we started parsing anime title but can't build it
                    // we should move to next one
                    if builder.is_started() && !builder.is_complete() {
                        continue;
                    } else {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                _ => continue,
            }
        }

        self.buffer.clear();
        builder.build().ok()
    }
}

/// Represents error that may happen on xml parsing
#[derive(Debug)]
pub enum XmlError {
    Io(std::io::Error),
    InvalidXml(String),
}

impl std::fmt::Display for XmlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use XmlError::*;

        match self {
            Io(e) => e.fmt(f),
            InvalidXml(msg) => msg.fmt(f),
        }
    }
}

impl std::error::Error for XmlError {}

impl From<QXError> for XmlError {
    fn from(error: QXError) -> Self {
        match error {
            QXError::Io(io_err) => XmlError::Io(io_err),
            _ => XmlError::InvalidXml(format!("{}", error)),
        }
    }
}

/// Represents error that may happen on xml processing
#[derive(Debug)]
pub enum ParseError {
    MalformedAttribute,
    UnexpectedState,
    BadUtf8,
}

impl From<QXError> for ParseError {
    fn from(_: QXError) -> Self {
        ParseError::MalformedAttribute
    }
}

impl From<Utf8Error> for ParseError {
    fn from(_: Utf8Error) -> Self {
        ParseError::BadUtf8
    }
}

impl From<ParseIntError> for ParseError {
    fn from(_: ParseIntError) -> Self {
        ParseError::MalformedAttribute
    }
}

impl From<AnimeBuildError> for ParseError {
    fn from(err: AnimeBuildError) -> Self {
        use AnimeBuildError::*;

        match err {
            NotStarted | AlreadyStarted | MalformedTitle => ParseError::UnexpectedState,
            _ => ParseError::MalformedAttribute,
        }
    }
}

// private helpers
impl AnimeBuilder {
    fn handle_id(&mut self, tag: &BytesStart<'_>) -> Result<(), ParseError> {
        let mut attributes = tag.attributes();
        let attr = attributes.next().ok_or(ParseError::MalformedAttribute)??;

        if attr.key != b"aid" {
            return Err(ParseError::MalformedAttribute);
        }

        let raw_id = attr.unescaped_value()?;
        let raw_id = std::str::from_utf8(&raw_id)?;
        let id = i32::from_str(&raw_id)?;
        self.set_id(id);

        Ok(())
    }

    fn handle_title_start(&mut self, tag: &BytesStart<'_>) -> Result<(), ParseError> {
        let attributes = tag.attributes();
        self.start_title_building()?;

        for attr in attributes {
            let attr = attr?;
            let value = attr.unescaped_value()?;
            let value = std::str::from_utf8(&value)?;

            match attr.key {
                b"xml:lang" => {
                    self.set_title_lang(value)?;
                }
                b"type" => {
                    self.set_title_kind(value)?;
                }
                _ => continue,
            }
        }

        Ok(())
    }

    fn handle_title(&mut self, text: &BytesText<'_>) -> Result<(), ParseError> {
        let raw_name = text.unescaped()?;
        let name = std::str::from_utf8(&raw_name)?;
        self.set_title(name)?;

        Ok(())
    }

    fn handle_title_end(&mut self) -> Result<(), ParseError> {
        self.finish_title_building()?;
        Ok(())
    }
}
