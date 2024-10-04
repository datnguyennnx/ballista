use std::fs::File;
use std::io;
use xml::reader::{EventReader, XmlEvent};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UtilError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("XML parsing error: {0}")]
    Xml(#[from] xml::reader::Error),
    #[error("No valid URLs found in the XML file")]
    NoValidUrls,
}

/// Parse a sitemap XML file and extract URLs
pub fn parse_sitemap(path: &str) -> Result<Vec<String>, UtilError> {
    let file = File::open(path)?;
    let parser = EventReader::new(file);
    let url_tags = ["loc", "url", "link"];

    let urls: Vec<String> = parser
        .into_iter()
        .filter_map(Result::ok)
        .filter_map(|e| match e {
            XmlEvent::Characters(content) if content.starts_with("http") => Some(content),
            _ => None,
        })
        .filter(|url| url_tags.iter().any(|&tag| url.contains(tag)))
        .filter(|url| !url.is_empty())
        .collect();

    if urls.is_empty() {
        Err(UtilError::NoValidUrls)
    } else {
        Ok(urls)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sitemap() {
        // This test would require a sample XML file
        // For now, we'll just test the error case
        let result = parse_sitemap("nonexistent_file.xml");
        assert!(result.is_err());
    }
}