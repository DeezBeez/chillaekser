use channel_interaction::ChannelInteractions;
use config::Config;
use serenity::all::ChannelType;
use serenity::all::ClientBuilder;
use serenity::all::CreateChannel;
use serenity::all::GuildChannel;
use serenity::all::Ready;
use serenity::all::VoiceState;
use serenity::async_trait;
use serenity::prelude::*;

mod channel_interaction;
mod config;

struct Handler;
#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _ctx: Context, _data_about_bot: Ready) {
        println!("Discord Bot is up and running!")
    }

    async fn voice_state_update(&self, ctx: Context, old: Option<VoiceState>, new: VoiceState) {
        // Always needed
        let create_channel_name = Config::get_create_channel_name().await.unwrap();
        let create_channel_category = Config::get_create_channel_category().await.unwrap();
        let interaction = self.get_channel_interactions(&old, &new);

        let category = {
            let guild = ctx.cache.guild(new.guild_id.unwrap()).unwrap();
            let mut ret = None;
            for (_c, gc) in guild.channels.iter() {
                if gc.name == create_channel_category {
                    ret = Some(gc.clone());
                    break;
                }
            }
            ret
        };
        let member = match &new.member {
            Some(m) => m,
            None => {
                println!("Could not find Member! Aborting voice_state_update!");
                return;
            }
        };
        let old_channel: Option<GuildChannel> = {
            let mut ret = None;
            if let Some(old) = &old {
                if let Some(id) = old.channel_id {
                    if let Ok(c) = ctx.http.get_channel(id).await {
                        if let Some(gc) = c.guild() {
                            ret = Some(gc);
                        }
                    }
                }
            }
            ret
        };
        let new_channel = {
            let mut ret = None;
            if let Some(id) = new.channel_id {
                if let Ok(c) = ctx.http.get_channel(id).await {
                    if let Some(gc) = c.guild() {
                        ret = Some(gc);
                    }
                }
            }
            ret
        };

        // Interactions
        println!("'{:?}' for '{}'", interaction, member.display_name());
        match interaction {
            ChannelInteractions::JoinedChannel => {
                if let None = new_channel {
                    return;
                }
                let new_channel = new_channel.unwrap();
                if create_channel_name == new_channel.name {
                    let channel = match self.create_new_channel(&ctx, &new).await {
                        Some(c) => c,
                        None => {
                            println!("Channel Creation failed!");
                            return;
                        }
                    };
                    match member.move_to_voice_channel(ctx.http, &channel).await {
                        Ok(_) => (),
                        Err(e) => {
                            println!(
                                "Error moving {} into {}: {}",
                                member.display_name(),
                                channel.name,
                                e
                            )
                        }
                    }
                }
            }

            ChannelInteractions::LeftChannel => {
                if let None = old_channel {
                    return;
                }
                if let None = category {
                    return;
                }
                let old_channel = old_channel.unwrap();
                if create_channel_name != old_channel.name {
                    if old_channel.members(ctx.cache).unwrap().len() <= 0
                        && old_channel.parent_id.unwrap() == category.unwrap().id
                        && old_channel.kind == ChannelType::Voice
                    {
                        ctx.http.delete_channel(old_channel.id, None).await.unwrap();
                    }
                }
            }

            ChannelInteractions::SwitchedChannel => {
                if let None = old_channel {
                    return;
                }
                let old_channel = old_channel.unwrap();
                if let None = new_channel {
                    return;
                }
                let new_channel = new_channel.unwrap();
                if let None = category {
                    return;
                }
                if create_channel_name == new_channel.name {
                    let channel = match self.create_new_channel(&ctx, &new).await {
                        Some(c) => c,
                        None => {
                            println!("Channel Creation failed!");
                            return;
                        }
                    };
                    match member.move_to_voice_channel(&ctx.http, &channel).await {
                        Ok(_) => (),
                        Err(e) => {
                            println!(
                                "Error moving {} into {}: {}",
                                member.display_name(),
                                channel.name,
                                e
                            )
                        }
                    }
                }
                if old_channel.name != create_channel_name {
                    if old_channel.members(ctx.cache).unwrap().len() <= 0
                        && old_channel.parent_id.unwrap() == category.unwrap().id
                        && old_channel.kind == ChannelType::Voice
                    {
                        ctx.http.delete_channel(old_channel.id, None).await.unwrap();
                    }
                }
            }
        }
    }
}

impl Handler {
    fn get_channel_interactions(
        &self,
        old: &Option<VoiceState>,
        new: &VoiceState,
    ) -> ChannelInteractions {
        if let None = old {
            return ChannelInteractions::JoinedChannel;
        }
        let new_channel_id = &new.channel_id;
        if let None = new_channel_id {
            return ChannelInteractions::LeftChannel;
        }
        ChannelInteractions::SwitchedChannel
    }
    async fn create_new_channel<'a>(&self, ctx: &Context, vc: &VoiceState) -> Option<GuildChannel> {
        let member = match &vc.member {
            Some(m) => m,
            None => {
                println!("create_new_channel: Could not find a member!");
                return None;
            }
        };
        let guild_id = match vc.guild_id {
            Some(g) => g,
            None => {
                println!("create_new_channel: Could not find guild id!");
                return None;
            }
        };
        let category: Option<GuildChannel>;
        let create_category_name = Config::get_create_channel_category().await.unwrap();
        category = {
            let guild = ctx.cache.guild(guild_id).unwrap();
            let mut ret = None;
            for (_c, gc) in guild.channels.iter() {
                if gc.name.to_lowercase() == create_category_name.to_lowercase() {
                    ret = Some(gc.clone());
                    break;
                }
            }
            ret
        };
        if let None = category {
            println!("Could not find create channel category!");
            return None;
        }
        let positions = ctx.cache.guild(guild_id).unwrap().channels.iter().count();
        let channel_builder = CreateChannel::new(member.display_name())
            .category(category.unwrap().id)
            .kind(ChannelType::Voice)
            .position((positions + 1) as u16);
        let created_channel = match ctx
            .http
            .create_channel(guild_id, &channel_builder, None)
            .await
        {
            Ok(c) => c,
            Err(e) => {
                println!("Error creating channel: {}", e);
                return None;
            }
        };
        Some(created_channel)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create Settings File if doesnt exist
    Config::create_settings_file().await?;

    let token = Config::get_token().await?;
    let intents = GatewayIntents::all();
    let mut client = ClientBuilder::new(token, intents)
        .event_handler(Handler)
        .await?;
    println!("Starting discord bot");
    client.start().await?;
    Ok(())
}
