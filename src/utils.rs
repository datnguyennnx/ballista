use std::process::Command;
use std::io::{self};
use std::fs::File;
use xml::reader::{EventReader, XmlEvent};

pub fn get_cpu_usage() -> Result<f64, io::Error> {
    let output = Command::new("sh")
        .arg("-c")
        .arg("top -bn1 | grep 'Cpu(s)' | sed 's/.*, *\\([0-9.]*\\)%* id.*/\\1/' | awk '{print 100 - $1}'")
        .output()?;
    
    String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
}

pub fn get_memory_usage() -> Result<f64, io::Error> {
    let output = Command::new("sh")
        .arg("-c")
        .arg("free | grep Mem | awk '{print $3/$2 * 100.0}'")
        .output()?;
    
    String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
}

pub fn parse_sitemap(path: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let parser = EventReader::new(file);
    let mut urls = Vec::new();
    let url_tags = ["loc", "url", "link"];
    let mut current_tag = String::new();

    for event in parser {
        match event {
            Ok(XmlEvent::StartElement { name, .. }) => {
                current_tag = name.local_name;
            }
            Ok(XmlEvent::Characters(content)) if url_tags.contains(&current_tag.as_str()) => {
                if content.starts_with("http") {
                    urls.push(content);
                }
            }
            Err(e) => return Err(Box::new(e)),
            _ => {}
        }
    }

    if urls.is_empty() {
        return Err("No valid URLs found in the XML file".into());
    }

    Ok(urls)
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