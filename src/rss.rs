use crate::config::RssConfig;
use anyhow::Result;
use htmd::HtmlToMarkdown;
use feed_rs::model::{Feed, Entry};

pub async fn fetch_feed(url: &str) -> Result<Feed> {
    let content = reqwest::get(url).await?.bytes().await?;
    let feed = feed_rs::parser::parse(&content[..])?;
    Ok(feed)
}

pub fn get_field_value(
    item: &Entry,
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

    if let Some(_p) = path {
        // Path-based resolution is harder with feed-rs because it's already abstracted.
        // We'll map some known paths to feed-rs fields.
        match _p.as_str() {
            "title" => item.title.as_ref().map(|t| t.content.clone()),
            "link" => item.links.first().map(|l| l.href.clone()),
            "description" | "summary" => item.summary.as_ref().map(|s| s.content.clone()),
            "content" => item.content.as_ref().and_then(|c| c.body.clone()),
            "published" | "pubDate" => item.published.map(|d| d.to_rfc3339()),
            "dc:creator" | "author.name" => item.authors.first().map(|a| a.name.clone()),
            "author.uri" => item.authors.first().and_then(|a| a.uri.clone()),
            _ => None,
        }
    } else {
        match field {
            "title" => item.title.as_ref().map(|t| t.content.clone()),
            "link" => item.links.first().map(|l| l.href.clone()),
            "content" => item.content.as_ref().and_then(|c| c.body.clone())
                .or_else(|| item.summary.as_ref().map(|s| s.content.clone())),
            "author" => item.authors.first().map(|a| a.name.clone()),
            "pubDate" => item.published.map(|d| d.to_rfc3339()),
            _ => None,
        }
    }
}

pub fn build_content(config: &RssConfig, item: &Entry) -> String {
    let mut parts = Vec::new();
    let ht = HtmlToMarkdown::new();

    let title = get_field_value(item, config, "title").unwrap_or_else(|| "Untitled".to_string());
    let link = get_field_value(item, config, "link");
    let content = get_field_value(item, config, "content");
    let author = get_field_value(item, config, "author");
    let author_link = get_field_value(item, config, "authorLink");
    let pub_date = item.published;

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
        parts.push(format!("\n-# 🕐 <t:{}:f>", pd.timestamp()));
    }

    let joined = parts.join("\n");
    if joined.is_empty() {
        link.unwrap_or_else(|| "No content available.".to_string())
    } else {
        joined
    }
}
