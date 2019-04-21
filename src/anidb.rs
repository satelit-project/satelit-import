mod build;
mod entity;

use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::str::FromStr;

use quick_xml::events::BytesStart;
use quick_xml::events::BytesText;
use quick_xml::events::Event;
use quick_xml::Reader;

use build::AnimeTitleBuilder;
use build::BuildError;
use entity::AnimeTitle;
use super::DecodingError;
use super::XmlError;

pub struct AniDb {
    reader: Reader<BufReader<File>>,
    buffer: Vec<u8>,
}

impl AniDb {
    pub fn new(path: &Path) -> Result<Self, XmlError> {
        let reader = Reader::from_file(path)?;
        let buffer = Vec::with_capacity(1024);

        Ok(AniDb { reader, buffer })
    }
}

// TODO: don't copy strings
impl Iterator for AniDb {
    type Item = AnimeTitle;

    fn next(&mut self) -> Option<AnimeTitle> {
        let mut builder = AnimeTitleBuilder::new();

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
                Ok(Event::Text(ref text)) if builder.is_building_name() => {
                    if let Err(e) = builder.handle_title_name(text) {
                        eprintln!("failed to parse title name: {:?}", e);
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

impl From<BuildError> for DecodingError {
    fn from(err: BuildError) -> Self {
        use self::BuildError::*;

        match err {
            NameBuildingNotStarted | NameBuildingNotFinished | IncompleteNameVariation => {
                DecodingError::IncorrectAttributeParsing
            }
            _ => DecodingError::MalformedAttribute,
        }
    }
}

impl AnimeTitleBuilder {
    fn handle_id(&mut self, tag: &BytesStart<'_>) -> Result<(), DecodingError> {
        let mut attributes = tag.attributes();
        let attr = attributes
            .next()
            .ok_or(DecodingError::MalformedAttribute)??;

        if attr.key != b"aid" {
            return Err(DecodingError::MalformedAttribute);
        }

        let raw_id = attr.unescaped_value()?;
        let raw_id = std::str::from_utf8(&raw_id)?;
        let id = u32::from_str(&raw_id)?;
        self.set_id(id);

        Ok(())
    }

    fn handle_title_start(&mut self, tag: &BytesStart<'_>) -> Result<(), DecodingError> {
        let attributes = tag.attributes();
        self.start_name_building()?;

        for attr in attributes {
            let attr = attr?;
            let value = attr.unescaped_value()?;
            let value = std::str::from_utf8(&value)?;

            match attr.key {
                b"xml:lang" => {
                    self.set_name_lang(value)?;
                }
                b"type" => {
                    self.set_name_type(value)?;
                }
                _ => continue,
            }
        }

        Ok(())
    }

    fn handle_title_name(&mut self, text: &BytesText<'_>) -> Result<(), DecodingError> {
        let raw_name = text.unescaped()?;
        let name = std::str::from_utf8(&raw_name)?;
        self.set_name(name)?;

        Ok(())
    }

    fn handle_title_end(&mut self) -> Result<(), DecodingError> {
        self.finish_name_building()?;
        Ok(())
    }
}
