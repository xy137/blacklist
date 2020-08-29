#![feature(bool_to_option)]
use serenity::async_trait;
use serenity::client::{Client, Context, EventHandler};
use serenity::framework::standard::{
    macros::{command, group},
    Args, CommandResult, StandardFramework,
};
use serenity::model::{channel::Message, gateway::Ready, guild::Member, id::GuildId};
use sled::IVec;

#[group]
#[commands(add, remove)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn guild_member_addition(&self, ctx: Context, _guild_id: GuildId, new: Member) {
        if check(&new.display_name()) {
            new.ban(ctx.http, 0).await.expect("Could not ban user");
        }
    }

    async fn guild_member_update(&self, ctx: Context, old: Option<Member>, new: Member) {
        if let Some(old) = old {
            if old.display_name() != new.display_name() && check(&new.display_name()) {
                new.ban(ctx.http, 0).await.expect("Could not ban user");
            }
        } else if check(&new.display_name()) {
            new.ban(ctx.http, 0).await.expect("Could not ban user");
        };
    }

    async fn ready(&self, _ctx: Context, _data_about_bot: Ready) {
        println!("bot be ready !");
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().unwrap();

    let token = std::env::var("DISCORD_TOKEN").expect("No token provided in .env file");

    let framework = StandardFramework::new()
        .configure(|c| c.case_insensitivity(true).prefix("."))
        .group(&GENERAL_GROUP);

    let mut client = Client::new(token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Could not create client");

    client.start().await.unwrap();
}

#[command]
async fn add(_ctx: &Context, _msg: &Message, mut args: Args) -> CommandResult {
    let word = args.single::<String>()?;

    let tree =
        sled::open(dirs::cache_dir().unwrap().join("xyBlacklist")).expect("Could not open db");

    tree.insert(tree.generate_id()?.to_string(), IVec::from(word.as_str()))?;

    Ok(())
}

#[command]
async fn remove(_ctx: &Context, _msg: &Message, mut args: Args) -> CommandResult {
    let word = args.single::<String>()?;

    let tree =
        sled::open(dirs::cache_dir().unwrap().join("xyBlacklist")).expect("Could not open db");

    if let Some(key) = tree
        .iter()
        .find_map(|iv| (iv.as_ref().unwrap().1 == IVec::from(word.as_str())).then_some(iv.ok()?.0))
    {
        tree.remove(key)?;
    };

    Ok(())
}

fn check(word: &str) -> bool {
    let tree =
        sled::open(dirs::cache_dir().unwrap().join("xyBlacklist")).expect("Could not open db");

    tree.iter()
        .values()
        .any(|iv| IVec::from(word) == iv.as_ref().unwrap())
}
