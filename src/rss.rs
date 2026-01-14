use crate::config::RssConfig;
use anyhow::Result;
use htmd::HtmlToMarkdown;
use feed_rs::model::{Feed, Entry};
use regex::Regex;

pub async fn fetch_feed(url: &str) -> Result<Feed> {
    let client = reqwest::Client::builder()
        .user_agent(std::env::var("RSS_USER_AGENT").unwrap_or_else(|_| "NekoRSS/1.0 (+https://github.com/imnyang/memos-rss)".to_string()))
        .build()?;
    
    let resp = client.get(url).send().await?;
    let status = resp.status();
    let content = resp.bytes().await?;
    
    match feed_rs::parser::parse(&content[..]) {
        Ok(feed) => Ok(feed),
        Err(e) => {
            let body_preview = String::from_utf8_lossy(&content).chars().take(200).collect::<String>();
            anyhow::bail!("Failed to parse feed (Status: {}): {} | Body preview: {}", status, e, body_preview);
        }
    }
}

pub fn should_include_by_link(link: Option<&String>, link_filters: Option<&Vec<String>>) -> bool {
    match (link, link_filters) {
        (Some(link_url), Some(filters)) => {
            // 모든 필터 중 하나라도 매치되면 포함
            filters.iter().any(|filter| {
                if let Ok(regex) = Regex::new(filter) {
                    regex.is_match(link_url)
                } else {
                    false
                }
            })
        }
        (_, None) => true, // 필터가 없으면 모두 포함
        (None, Some(_)) => false, // 필터가 있는데 link가 없으면 제외
    }
}

pub fn extract_image_url(item: &Entry) -> Option<String> {
    // 첫 번째로 media:content에서 이미지 찾기
    item.media
        .iter()
        .find(|m| m.content.iter().any(|c| c.medium.as_deref() == Some("image")))
        .and_then(|m| m.content.first())
        .and_then(|c| c.url.clone())
        .map(|url| url.to_string())
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

    let timestamp = pub_date
        .map(|pd| pd.timestamp())
        .unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64
        });
    parts.push(format!("\n-# 🕐 <t:{}:f>", timestamp));

    let joined = parts.join("\n");
    if joined.is_empty() {
        link.unwrap_or_else(|| "No content available.".to_string())
    } else {
        joined
    }
}
