use std::io;
use xml::reader::{EventReader, XmlEvent};
use thiserror::Error;
use serde_json;

#[derive(Error, Debug)]
pub enum UtilError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("XML parsing error: {0}")]
    Xml(#[from] xml::reader::Error),
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("No valid URLs found in the XML file")]
    NoValidUrls,
}

/// Parse a sitemap XML file and extract URLs
pub fn parse_urls(content: &str) -> Result<Vec<String>, UtilError> {
    let parser = EventReader::from_str(content);
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

/// Parse JSON content
pub fn parse_json<T: serde::de::DeserializeOwned>(content: &str) -> Result<T, UtilError> {
    serde_json::from_str(content).map_err(UtilError::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_urls() {
        let xml_content = r#"
            <?xml version="1.0" encoding="UTF-8"?>
            <urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
                <url>
                    <loc>https://example.com/page1</loc>
                </url>
                <url>
                    <loc>https://example.com/page2</loc>
                </url>
            </urlset>
        "#;
        let result = parse_urls(xml_content);
        assert!(result.is_ok());
        let urls = result.unwrap();
        assert_eq!(urls.len(), 2);
        assert!(urls.contains(&"https://example.com/page1".to_string()));
        assert!(urls.contains(&"https://example.com/page2".to_string()));
    }

    #[test]
    fn test_parse_json() {
        #[derive(serde::Deserialize, Debug, PartialEq)]
        struct TestStruct {
            name: String,
            age: u32,
        }

        let json_content = r#"
            {
                "name": "John Doe",
                "age": 30
            }
        "#;
        let result: Result<TestStruct, UtilError> = parse_json(json_content);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed, TestStruct { name: "John Doe".to_string(), age: 30 });
    }
}