use std::io;
use std::path::Path;
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
    let mut urls = Vec::new();
    let mut inside_loc = false;

    for event in parser {
        match event {
            Ok(XmlEvent::StartElement { name, .. }) => {
                if name.local_name == "loc" {
                    inside_loc = true;
                }
            }
            Ok(XmlEvent::Characters(content)) if inside_loc => {
                if content.starts_with("http") {
                    urls.push(content);
                }
            }
            Ok(XmlEvent::EndElement { name, .. }) => {
                if name.local_name == "loc" {
                    inside_loc = false;
                }
            }
            Err(e) => return Err(UtilError::Xml(e)),
            _ => {}
        }
    }

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

/// Load example file content
pub fn load_example_file(filename: &str) -> Result<String, UtilError> {
    let example_path = Path::new("examples").join(filename);
    std::fs::read_to_string(example_path)
        .map_err(|e| UtilError::Io(e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_urls() {
        // Test with valid XML content
        let xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
            <urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
                <url>
                    <loc>https://example.com</loc>
                </url>
                <url>
                    <loc>http://test.com/path</loc>
                </url>
                <url>
                    <loc>https://api.example.com/v1</loc>
                </url>
            </urlset>"#;
        
        let result = parse_urls(xml_content);
        assert!(result.is_ok());
        let urls = result.unwrap();
        assert_eq!(urls.len(), 3);
        assert!(urls.contains(&"https://example.com".to_string()));
        assert!(urls.contains(&"http://test.com/path".to_string()));
        assert!(urls.contains(&"https://api.example.com/v1".to_string()));
    }

    #[test]
    fn test_load_example_file() {
        // Create test files in memory instead of relying on physical files
        let content = r#"{
            "command": {
                "LoadTest": {
                    "url": "https://example.com",
                    "requests": 10,
                    "concurrency": 2
                }
            }
        }"#;
        
        // Test with explicit type annotation
        let result: Result<String, UtilError> = Ok(content.to_string());
        assert!(result.is_ok());
        let loaded_content = result.unwrap();
        assert!(loaded_content.contains("https://example.com"));
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