mod config;
mod rss;
mod storage;
mod discord;

use anyhow::Result;
use dotenvy::dotenv;
use serenity::prelude::*;
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;

use crate::config::FullConfig;
use crate::storage::Storage;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    
    let token = env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN in environment");
    let interval_mins = env::var("INTERVAL_MINUTES")
        .unwrap_or_else(|_| "5".to_string())
        .parse::<u64>()
        .unwrap_or(5);
    
    let rss_toml_content = std::fs::read_to_string("rss.toml")?;
    let config: FullConfig = toml::from_str(&rss_toml_content)?;
    let config = Arc::new(config);
    
    let storage = Arc::new(Storage::new("processed_items.db")?);
    
    // Migration from old processed.json if it exists
    let old_processed_path = "../memos-rss/processed.json";
    if std::path::Path::new(old_processed_path).exists() {
        println!("Found old processed.json, migrating data...");
        let content = std::fs::read_to_string(old_processed_path)?;
        if let Ok(data) = serde_json::from_str::<std::collections::HashMap<String, Vec<String>>>(&content) {
            storage.mark_processed_bulk(data)?;
            println!("Migration complete. Deleting old processed.json...");
            // Optionally rename or delete it to avoid re-migration
            let _ = std::fs::rename(old_processed_path, format!("{}.bak", old_processed_path));
        }
    }
    
    let handler = discord::Handler {
        config: config.clone(),
    };
    
    let mut client = Client::builder(&token, GatewayIntents::GUILDS)
        .event_handler(handler)
        .await
        .expect("Err creating client");
    
    let http = client.http.clone();
    let config_clone = config.clone();
    let storage_clone = storage.clone();
    
    // Spawn RSS checking loop
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(interval_mins * 60));
        loop {
            interval.tick().await;
            println!("Checking RSS feeds...");
            for (name, rss_config) in config_clone.iter() {
                if let Err(e) = check_feed(name, rss_config, &http, &storage_clone).await {
                    eprintln!("[{}] Error checking feed: {}", name, e);
                }
            }
        }
    });

    if let Err(why) = client.start().await {
        eprintln!("Client error: {:?}", why);
    }
    
    Ok(())
}

async fn check_feed(
    name: &str,
    config: &crate::config::RssConfig,
    http: &Arc<serenity::http::Http>,
    storage: &Arc<Storage>,
) -> Result<()> {
    let feed = rss::fetch_feed(&config.rss).await?;
    
    // items usually come in descending order (newest first)
    // we should probably reverse them to process oldest first to maintain order in Discord
    let mut items = feed.items().to_vec();
    items.reverse();

    for item in items {
        let item_id = rss::get_field_value(&item, config, "link")
            .or_else(|| item.guid().map(|g| g.value().to_string()))
            .unwrap_or_default();
            
        if item_id.is_empty() { continue; }
        
        if !storage.is_processed(name, &item_id)? {
            // Category check
            if let Some(filters) = &config.category_filter {
                let item_categories: Vec<_> = item.categories().iter().map(|c| c.name()).collect();
                if item_categories.iter().any(|c| filters.contains(&c.to_string())) {
                    println!("[{}] Filtered by category: {:?}", name, item_categories);
                    storage.mark_processed(name, &item_id)?;
                    continue;
                }
            }

            println!("[{}] New item: {:?}", name, item.title());
            let content = rss::build_content(config, &item);
            
            let channel_id = config.channel.parse::<u64>()?;
            let channel = serenity::model::id::ChannelId::new(channel_id);
            
            let title = rss::get_field_value(&item, config, "title").unwrap_or_else(|| "Untitled".to_string());
            let tag_id = config.tag.parse::<u64>()?;
            
            // Post to forum
            let post = serenity::builder::CreateForumPost::new(
                title,
                serenity::builder::CreateMessage::new().content(content)
            ).add_applied_tag(serenity::model::id::ForumTagId::new(tag_id));
            
            if let Err(e) = channel.create_forum_post(&http, post).await {
                eprintln!("[{}] Failed to create forum post: {}", name, e);
                continue;
            }
            
            // After creating thread, post the message if needed, 
            // but CreateThread in Serenity for Forum usually takes message too?
            // Actually, for Forum channels, the first message is part of the thread creation.
            // Let's refine this if needed based on Serenity 0.12 API.
            
            storage.mark_processed(name, &item_id)?;
        }
    }
    
    Ok(())
}
