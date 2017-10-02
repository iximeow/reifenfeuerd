extern crate termion;

use self::termion::color;

use ::tw;

use std;

fn color_for(handle: &String) -> termion::color::Fg<&color::Color> {
    let color_map: Vec<&color::Color> = vec![
        &color::Blue,
        &color::Cyan,
        &color::Green,
        &color::LightBlue,
        &color::LightCyan,
        &color::LightGreen,
        &color::LightMagenta,
        &color::LightYellow,
        &color::Magenta,
        &color::Yellow
    ];

    let mut quot_hash_quot = std::num::Wrapping(0);
    for b in handle.as_bytes().iter() {
        quot_hash_quot = quot_hash_quot + std::num::Wrapping(*b);
    }
    color::Fg(color_map[quot_hash_quot.0 as usize % color_map.len()])
}

pub trait Render {
    fn render(self, tweeter: &::tw::TwitterCache);
}

impl Render for tw::events::Event {
    fn render(self, tweeter: &::tw::TwitterCache) {
        match self {
            tw::events::Event::Deleted { user_id, twete_id } => {
                if let Some(handle) = tweeter.retrieve_user(&user_id).map(|x| &x.handle) {
                    if let Some(_tweet) = tweeter.retrieve_tweet(&twete_id) {
                        println!("-------------DELETED------------------");
                        render_twete(&twete_id, tweeter);
                        println!("-------------DELETED------------------");
                    } else {
                        println!("dunno what, but do know who: {} - {}", user_id, handle);
                    }
                } else {
                    println!("delete...");
                    println!("dunno who...");
                }
            },
            tw::events::Event::RT_RT { user_id, twete_id } => {
                println!("---------------------------------");
                let user = tweeter.retrieve_user(&user_id).unwrap();
                println!("  +rt_rt    : {} (@{})", user.name, user.handle);
                render_twete(&twete_id, tweeter);
            },
            tw::events::Event::Fav_RT { user_id, twete_id } => {
                println!("---------------------------------");
                let user = tweeter.retrieve_user(&user_id).unwrap();
                println!("  +rt_fav   : {} (@{})", user.name, user.handle);
                render_twete(&twete_id, tweeter);
            },
            tw::events::Event::Fav { user_id, twete_id } => {
                println!("---------------------------------");
                let user = tweeter.retrieve_user(&user_id).unwrap();
                println!("{}  +fav      : {} (@{}){}", color::Fg(color::Yellow), user.name, user.handle, color::Fg(color::Reset));
                render_twete(&twete_id, tweeter);
            },
            tw::events::Event::Unfav { user_id, twete_id } => {
                println!("---------------------------------");
                let user = tweeter.retrieve_user(&user_id).unwrap();
                println!("{}  -fav      : {} (@{}){}", color::Fg(color::Yellow), user.name, user.handle, color::Fg(color::Reset));
                render_twete(&twete_id, tweeter);
            },
            tw::events::Event::Followed { user_id } => {
                println!("---------------------------------");
                let user = tweeter.retrieve_user(&user_id).unwrap();
                println!("  +fl       : {} (@{})", user.name, user.handle);
            },
            tw::events::Event::Unfollowed { user_id } => {
                println!("---------------------------------");
                let user = tweeter.retrieve_user(&user_id).unwrap();
                println!("  -fl       : {} (@{})", user.name, user.handle);
            }
            /*
            Blocked(user_id) => {
            },
            */
        }
        println!("");
    }
}

pub fn render_twete(twete_id: &String, tweeter: &tw::TwitterCache) {
    let id_color = color::Fg(color::Rgb(180, 80, 40));
    let twete = tweeter.retrieve_tweet(twete_id).unwrap();
    // if we got the tweet, the API gave us the user too
    let user = tweeter.retrieve_user(&twete.author_id).unwrap();
    match twete.rt_tweet {
        Some(ref rt_id) => {
            // same for a retweet
            let rt = tweeter.retrieve_tweet(rt_id).unwrap();
            // and its author
            let rt_author = tweeter.retrieve_user(&rt.author_id).unwrap();
            println!("{}  id:{} (rt_id:{}){}{}",
                id_color, rt.internal_id, twete.internal_id,
                rt.reply_to_tweet.clone()
                    .map(|id| tweeter.retrieve_tweet(&id)
                        .and_then(|tw| Some(format!(" reply_to:{}", tw.internal_id)))
                        .unwrap_or(format!(" reply_to:twitter::{}", id))
                    )
                    .unwrap_or("".to_string()),
                color::Fg(color::Reset)
            );
            println!("  {}{}{} ({}@{}{}) via {}{}{} ({}@{}{}) RT:",
                color_for(&rt_author.handle), rt_author.name, color::Fg(color::Reset),
                color_for(&rt_author.handle), rt_author.handle, color::Fg(color::Reset),
                color_for(&user.handle), user.name, color::Fg(color::Reset),
                color_for(&user.handle), user.handle, color::Fg(color::Reset)
            );
        }
        None => {
            println!("{}  id:{}{}{}",
                id_color, twete.internal_id,
                twete.reply_to_tweet.clone()
                    .map(|id| tweeter.retrieve_tweet(&id)
                        .and_then(|tw| Some(format!(" reply_to:{}", tw.internal_id)))
                        .unwrap_or(format!(" reply_to:twitter::{}", id))
                    )
                    .unwrap_or("".to_string()),
                color::Fg(color::Reset)
            );
            println!("  {}{}{} ({}@{}{})",
                color_for(&user.handle), user.name, color::Fg(color::Reset),
                color_for(&user.handle), user.handle, color::Fg(color::Reset)
            );
        }
    }

    println!("      {}", twete.text.replace("\r", "\\r").split("\n").collect::<Vec<&str>>().join("\n      "));

    if let Some(ref qt_id) = twete.quoted_tweet_id {
        if let Some(ref qt) = tweeter.retrieve_tweet(qt_id) {
            let qt_author = tweeter.retrieve_user(&qt.author_id).unwrap();
            println!("{}    id:{}{}{}",
                id_color, qt.internal_id,
                qt.reply_to_tweet.clone()
                    .map(|id| tweeter.retrieve_tweet(&id)
                        .and_then(|tw| Some(format!(" reply_to:{}", tw.internal_id)))
                        .unwrap_or(format!(" reply_to:twitter::{}", id))
                    )
                    .unwrap_or("".to_string()),
                color::Fg(color::Reset)
            );
            println!(
                "    {}{}{} ({}@{}{})",
                color_for(&qt_author.handle), qt_author.name, color::Fg(color::Reset),
                color_for(&qt_author.handle), qt_author.handle, color::Fg(color::Reset)
            );
            println!(
                "        {}",
                qt.text.replace("\r", "\\r").split("\n").collect::<Vec<&str>>().join("\n        ")
            );
        } else {
            println!("    << don't have quoted tweet! >>");
        }
    }
}
