use std::process::Command;
use std::io::{self, Error, ErrorKind};
use std::fs::File;
use xml::reader::{EventReader, XmlEvent};

/// Get the current CPU usage as a percentage
pub fn get_cpu_usage() -> io::Result<f64> {
    Command::new("sh")
        .arg("-c")
        .arg("top -bn1 | grep 'Cpu(s)' | sed 's/.*, *\\([0-9.]*\\)%* id.*/\\1/' | awk '{print 100 - $1}'")
        .output()
        .and_then(|output| {
            String::from_utf8(output.stdout)
                .map_err(|e| Error::new(ErrorKind::InvalidData, e))
                .and_then(|s| s.trim().parse::<f64>().map_err(|e| Error::new(ErrorKind::InvalidData, e)))
        })
}

/// Get the current memory usage as a percentage
pub fn get_memory_usage() -> io::Result<f64> {
    Command::new("sh")
        .arg("-c")
        .arg("free | grep Mem | awk '{print $3/$2 * 100.0}'")
        .output()
        .and_then(|output| {
            String::from_utf8(output.stdout)
                .map_err(|e| Error::new(ErrorKind::InvalidData, e))
                .and_then(|s| s.trim().parse::<f64>().map_err(|e| Error::new(ErrorKind::InvalidData, e)))
        })
}

/// Parse a sitemap XML file and extract URLs
pub fn parse_sitemap(path: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let parser = EventReader::new(file);
    let url_tags = ["loc", "url", "link"];

    parser.into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|e| match e {
            XmlEvent::Characters(content) if content.starts_with("http") => Some(content),
            _ => None,
        })
        .collect::<Vec<String>>()
        .into_iter()
        .filter(|url| url_tags.iter().any(|&tag| url.contains(tag)))
        .collect::<Vec<String>>()
        .into_iter()
        .filter(|url| !url.is_empty())
        .map(|url| Ok(url))
        .collect::<Result<Vec<String>, Box<dyn std::error::Error>>>()
        .and_then(|urls| {
            if urls.is_empty() {
                Err("No valid URLs found in the XML file".into())
            } else {
                Ok(urls)
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_parse_sitemap() {
        let sitemap_content = r#"
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

        let generic_xml_content = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <root>
            <item>
                <link>https://example.com/item1</link>
            </item>
            <item>
                <link>https://example.com/item2</link>
            </item>
        </root>
        "#;

        let test_files = vec![
            ("test_sitemap.xml", sitemap_content),
            ("test_generic.xml", generic_xml_content),
        ];

        for (filename, content) in test_files {
            let mut file = File::create(filename).unwrap();
            file.write_all(content.as_bytes()).unwrap();

            let urls = parse_sitemap(filename).unwrap();
            assert_eq!(urls.len(), 2);
            assert!(urls.iter().all(|url| url.starts_with("https://example.com/")));

            std::fs::remove_file(filename).unwrap();
        }
    }
}