use serenity::async_trait;
use serenity::model::gateway::Ready;
use serenity::model::application::Interaction;
use serenity::prelude::*;
use crate::config::FullConfig;
use std::sync::Arc;

pub struct Handler {
    pub config: Arc<FullConfig>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        
        // Register slash commands
        let commands = vec![
            serenity::builder::CreateCommand::new("clear-forum")
                .description("Clear posts in the forum channel")
        ];

        for command in commands {
            if let Err(e) = serenity::model::application::Command::create_global_command(&ctx.http, command).await {
                eprintln!("Cannot create command: {}", e);
            }
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            if command.data.name == "clear-forum" {
                let _ = command.defer(&ctx.http).await;
                
                let mut count = 0;
                for rss_config in self.config.values() {
                    if let Ok(channel_id_val) = rss_config.channel.parse::<u64>() {
                        let channel_id = serenity::model::id::ChannelId::new(channel_id_val);
                        if let Ok(serenity::all::Channel::Guild(guild_channel)) = channel_id.to_channel(&ctx.http).await {
                            if let Ok(threads) = guild_channel.guild_id.get_active_threads(&ctx.http).await {
                                for thread in threads.threads {
                                    if thread.parent_id == Some(channel_id) {
                                        if let Err(e) = thread.delete(&ctx.http).await {
                                            eprintln!("Failed to delete thread {}: {}", thread.id, e);
                                        } else {
                                            count += 1;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                let _ = command.edit_response(&ctx.http, serenity::builder::EditInteractionResponse::new()
                    .content(format!("✅ Successfully cleared {} threads from forum channels.", count))).await;
            }
        }
    }
}
