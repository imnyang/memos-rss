use crate::config::RssConfig;
use anyhow::Result;
use htmd::HtmlToMarkdown;
use rss::Channel;

pub async fn fetch_feed(url: &str) -> Result<Channel> {
    let content = reqwest::get(url).await?.bytes().await?;
    let channel = Channel::read_from(&content[..])?;
    Ok(channel)
}

pub fn get_field_value(
    item: &rss::Item,
    config: &RssConfig,
    field: &str,
) -> Option<String> {
    let setup = config.setup.as_ref();
    let path = match field {
        "title" => setup.and_then(|s| s.title.as_ref()),
        "link" => setup.and_then(|s| s.link.as_ref()),
        "content" => setup.and_then(|s| s.content.as_ref()),
        "author" => setup.and_then(|s| s.author.as_ref()),
        "authorLink" => setup.and_then(|s| s.author_link.as_ref()),
        "pubDate" => setup.and_then(|s| s.pub_date.as_ref()),
        _ => None,
    };

    if let Some(p) = path {
        // Simple JSON path-like resolution for extensions if needed
        // For now, check standard fields first, then extensions
        resolve_path(item, p)
    } else {
        match field {
            "title" => item.title().map(|s| s.to_string()),
            "link" => item.link().map(|s| s.to_string()),
            "content" => item.content().or(item.description()).map(|s| s.to_string()),
            "author" => item.author().map(|s| s.to_string()),
            "pubDate" => item.pub_date().map(|s| s.to_string()),
            _ => None,
        }
    }
}

fn resolve_path(item: &rss::Item, path: &str) -> Option<String> {
    // This is a simplified version. The TS version used a more complex object traversal.
    // rss-rs doesn't expose a raw JSON-like structure easily for all extensions.
    // We'll handle common cases or use extensions() if needed.
    
    match path {
        "title" => item.title().map(|s| s.to_string()),
        "link" => item.link().map(|s| s.to_string()),
        "description" => item.description().map(|s| s.to_string()),
        "content" => item.content().map(|s| s.to_string()),
        "pubDate" => item.pub_date().map(|s| s.to_string()),
        "dc:creator" => item.dublin_core_ext()
            .and_then(|dc| dc.creators().first())
            .map(|s| s.to_string()),
        _ => {
            // Check extensions
            for (ns, ext) in item.extensions() {
                if path.starts_with(ns) {
                    let local_name = &path[ns.len()+1..];
                    if let Some(v) = ext.get(local_name) {
                        if let Some(first) = v.first() {
                            return first.value().map(|s| s.to_string());
                        }
                    }
                }
            }
            None
        }
    }
}

pub fn build_content(config: &RssConfig, item: &rss::Item) -> String {
    let mut parts = Vec::new();
    let ht = HtmlToMarkdown::new();

    let title = get_field_value(item, config, "title").unwrap_or_else(|| "Untitled".to_string());
    let link = get_field_value(item, config, "link");
    let content = get_field_value(item, config, "content");
    let author = get_field_value(item, config, "author");
    let author_link = get_field_value(item, config, "authorLink");
    let pub_date = get_field_value(item, config, "pubDate");

    if let Some(l) = link.as_ref() {
        parts.push(format!("# {} | [{}](<{}>)", config.emoji, title, l));
    } else {
        parts.push(format!("# {} | {}", config.emoji, title));
    }

    if let Some(a) = author {
        if let Some(al) = author_link {
            parts.push(format!("-# 🖊️ [{}](<{}>)", a, al));
        } else {
            parts.push(format!("-# 🖊️ {}", a));
        }
    }

    if let Some(c) = content {
        let markdown = ht.convert(&c).unwrap_or_else(|_| c);
        parts.push(format!("\n{}\n", markdown.trim()));
    }

    if let Some(pd) = pub_date {
        if let Ok(dt) = chrono::DateTime::parse_from_rfc2822(&pd) {
            parts.push(format!("\n-# 🕐 <t:{}:f>", dt.timestamp()));
        } else if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&pd) {
             parts.push(format!("\n-# 🕐 <t:{}:f>", dt.timestamp()));
        }
    }

    let joined = parts.join("\n");
    if joined.is_empty() {
        link.unwrap_or_else(|| "No content available.".to_string())
    } else {
        joined
    }
}
