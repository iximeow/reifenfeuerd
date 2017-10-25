extern crate termion;

use std::io::Write;
use std::io::stdout;

use self::termion::color;
use self::termion::{clear, cursor};

use ::tw;

use ::tw::TweetId;

use std;

#[derive(Clone)]
pub enum Infos {
    Tweet(TweetId),
    Thread(Vec<TweetId>),
    Event(tw::events::Event),
    DM(String)
}

pub fn paint(tweeter: &mut ::tw::TwitterCache) {
    match termion::terminal_size() {
        Ok((width, height)) => {
            // draw input prompt
            println!("{}{}", cursor::Goto(1, height - 6), clear::CurrentLine);
            println!("{}{}>", cursor::Goto(1, height - 5), clear::CurrentLine);
            println!("{}{}", cursor::Goto(1, height - 4), clear::CurrentLine);
            let mut i = 0;
            let log_size = 4;
            let last_elem = tweeter.display_info.log.len().saturating_sub(log_size);
            {
                let to_show = tweeter.display_info.log.drain(last_elem..);
                for line in to_show {
                    println!("{}{}{}", cursor::Goto(1, height - i), clear::CurrentLine, line);
                    i = i + 1;
                }
            }
            while i < log_size as u16 {
                println!("{}{}", cursor::Goto(1, height - i), clear::CurrentLine);
                i = i + 1;
            }
            // draw status lines
            // draw tweets
            let last_twevent = tweeter.display_info.infos.len().saturating_sub(height as usize - 4);
            let last_few_twevent: Vec<Infos> = tweeter.display_info.infos[last_twevent..].iter().map(|x| x.clone()).rev().collect::<Vec<Infos>>();

            let mut h = 7;
            for info in last_few_twevent {
                let mut to_draw = match info {
                    Infos::Tweet(id) => {
                        render_twete(&id, tweeter).iter().map(|x| x.to_owned()).rev().collect()
                    }
                    Infos::Thread(ids) => {
                        vec![format!("{}{}I'd show a thread if I knew how", cursor::Goto(1, height - h), clear::CurrentLine)]
                    },
                    Infos::Event(e) => {
                        e.clone().render(tweeter).into_iter().rev().collect()
                    },
                    Infos::DM(msg) => {
                        vec![format!("{}{}DM: {}", cursor::Goto(1, height - h), clear::CurrentLine, msg)]
                    }
                };
                for line in to_draw {
                    println!("{}{}{}", cursor::Goto(1, height - h), clear::CurrentLine, line);
                    h = h + 1;
                    if h >= height {
                        print!("{}", cursor::Goto(2, height - 6));
            stdout().flush();
                        return;
                    }
                }
                println!("{}{}", cursor::Goto(1, height - h), clear::CurrentLine);
                h = h + 1;
                if h >= height {
                    print!("{}", cursor::Goto(2, height - 6));
            stdout().flush();
                    return;
                }
            }
            while h < height {
                println!("{}{}", cursor::Goto(1, height - h), clear::CurrentLine);
                h = h + 1;
            }
            print!("{}", cursor::Goto(2, height - 6));
            stdout().flush();
        },
        Err(e) => {
            println!("Can't get term dimensions: {}", e);
        }
    }
}

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
    fn render(self, tweeter: &mut ::tw::TwitterCache) -> Vec<String>;
}

impl Render for tw::events::Event {
    fn render(self, tweeter: &mut ::tw::TwitterCache) -> Vec<String> {
        let mut result = Vec::new();
        match self {
            tw::events::Event::Quoted { user_id, twete_id } => {
                result.push("---------------------------------".to_string());
                {
                    let user = tweeter.retrieve_user(&user_id).unwrap();
                    println!("  quoted_tweet    : {} (@{})", user.name, user.handle);
                }
                render_twete(&TweetId::Twitter(twete_id), tweeter);
            }
            tw::events::Event::Deleted { user_id, twete_id } => {
                if let Some(handle) = tweeter.retrieve_user(&user_id).map(|x| &x.handle).map(|x| x.clone()) {
                    if let Some(_tweet) = tweeter.retrieve_tweet(&TweetId::Twitter(twete_id.to_owned())).map(|x| x.clone()) {
                        result.push(format!("-------------DELETED------------------"));
                        result.extend(render_twete(&TweetId::Twitter(twete_id), tweeter));
                        result.push(format!("-------------DELETED------------------"));
                    } else {
                        result.push(format!("dunno what, but do know who: {} - {}", user_id, handle));
                    }
                } else {
                    result.push("delete... dunno who".to_string());
                }
            },
            tw::events::Event::RT_RT { user_id, twete_id } => {
                result.push("---------------------------------".to_string());
                {
                let user = tweeter.retrieve_user(&user_id).unwrap();
                result.push(format!("  +rt_rt    : {} (@{})", user.name, user.handle));
                }
                result.extend(render_twete(&TweetId::Twitter(twete_id), tweeter));
            },
            tw::events::Event::Fav_RT { user_id, twete_id } => {
                result.push("---------------------------------".to_string());
                if let Some(user) = tweeter.retrieve_user(&user_id) {
                    result.push(format!("  +rt_fav   : {} (@{})", user.name, user.handle));
                } else {
                    result.push(format!("  +rt_fav but don't know who {} is", user_id));
                }
                result.extend(render_twete(&TweetId::Twitter(twete_id), tweeter));
            },
            tw::events::Event::Fav { user_id, twete_id } => {
                result.push("---------------------------------".to_string());
                {
                let user = tweeter.retrieve_user(&user_id).unwrap();
                result.push(format!("{}  +fav      : {} (@{}){}", color::Fg(color::Yellow), user.name, user.handle, color::Fg(color::Reset)));
                }
                result.extend(render_twete(&TweetId::Twitter(twete_id), tweeter));
            },
            tw::events::Event::Unfav { user_id, twete_id } => {
                result.push("---------------------------------".to_string());
                {
                let user = tweeter.retrieve_user(&user_id).unwrap();
                result.push(format!("{}  -fav      : {} (@{}){}", color::Fg(color::Yellow), user.name, user.handle, color::Fg(color::Reset)));
                }
                result.extend(render_twete(&TweetId::Twitter(twete_id), tweeter));
            },
            tw::events::Event::Followed { user_id } => {
                result.push("---------------------------------".to_string());
                let user = tweeter.retrieve_user(&user_id).unwrap();
                result.push(format!("  +fl       : {} (@{})", user.name, user.handle));
            },
            tw::events::Event::Unfollowed { user_id } => {
                result.push("---------------------------------".to_string());
                let user = tweeter.retrieve_user(&user_id).unwrap();
                result.push(format!("  -fl       : {} (@{})", user.name, user.handle));
            }
            /*
            Blocked(user_id) => {
            },
            */
        }
        result
    }
}

pub fn render_twete(twete_id: &TweetId, tweeter: &mut tw::TwitterCache) -> Vec<String> {
    let mut result = Vec::new();
    let id_color = color::Fg(color::Rgb(180, 80, 40));
    let maybe_twete = tweeter.retrieve_tweet(twete_id).map(|x| x.clone());
    if maybe_twete.is_none() {
        result.push(format!("No such tweet: {:?}", twete_id));
        return result;
    }
    let twete = maybe_twete.unwrap();
    // if we got the tweet, the API gave us the user too
    let user = tweeter.retrieve_user(&twete.author_id).map(|x| x.clone()).unwrap();
    match twete.rt_tweet {
        Some(ref rt_id) => {
            // same for a retweet
            let rt = tweeter.retrieve_tweet(&TweetId::Twitter(rt_id.to_owned())).unwrap().clone();
            // and its author
            let rt_author = tweeter.retrieve_user(&rt.author_id).unwrap().clone();
            result.push(format!("{}  id:{} (rt_id:{}){}{}",
                id_color, rt.internal_id, twete.internal_id,
                rt.reply_to_tweet.clone()
                    .map(|id| tweeter.retrieve_tweet(&TweetId::Twitter(id.to_owned()))
                        .and_then(|tw| Some(format!(" reply_to:{}", tw.internal_id)))
                        .unwrap_or(format!(" reply_to:twitter::{}", id))
                    )
                    .unwrap_or("".to_string()),
                color::Fg(color::Reset)
            ));
            result.push(format!("  {}{}{} ({}@{}{}) via {}{}{} ({}@{}{}) RT:",
                color_for(&rt_author.handle), rt_author.name, color::Fg(color::Reset),
                color_for(&rt_author.handle), rt_author.handle, color::Fg(color::Reset),
                color_for(&user.handle), user.name, color::Fg(color::Reset),
                color_for(&user.handle), user.handle, color::Fg(color::Reset)
            ));
        }
        None => {
            result.push(format!("{}  id:{}{}{}",
                id_color, twete.internal_id,
                twete.reply_to_tweet.clone()
                    .map(|id| tweeter.retrieve_tweet(&TweetId::Twitter(id.to_owned()))
                        .and_then(|tw| Some(format!(" reply_to:{}", tw.internal_id)))
                        .unwrap_or(format!(" reply_to:twitter::{}", id))
                    )
                    .unwrap_or("".to_string()),
                color::Fg(color::Reset)
            ));
            result.push(format!("  {}{}{} ({}@{}{})",
                color_for(&user.handle), user.name, color::Fg(color::Reset),
                color_for(&user.handle), user.handle, color::Fg(color::Reset)
            ));
        }
    }

    result.extend(
        format!("      {}", twete.text.replace("\r", "\\r").split("\n").collect::<Vec<&str>>().join("\n      ")).split("\n").map(|x| x.to_owned())
    );

    if let Some(ref qt_id) = twete.quoted_tweet_id {
        let maybe_qt = tweeter.retrieve_tweet(&TweetId::Twitter(qt_id.to_owned())).map(|x| x.to_owned());
        if let Some(qt) = maybe_qt {
            let qt_author = tweeter.retrieve_user(&qt.author_id).unwrap().clone();
            result.push(format!("{}    id:{}{}{}",
                id_color, qt.internal_id,
                qt.reply_to_tweet.clone()
                    .map(|id| tweeter.retrieve_tweet(&TweetId::Twitter(id.to_owned()))
                        .and_then(|tw| Some(format!(" reply_to:{}", tw.internal_id)))
                        .unwrap_or(format!(" reply_to:twitter::{}", id))
                    )
                    .unwrap_or("".to_string()),
                color::Fg(color::Reset)
            ));
            result.push(format!(
                "    {}{}{} ({}@{}{})",
                color_for(&qt_author.handle), qt_author.name, color::Fg(color::Reset),
                color_for(&qt_author.handle), qt_author.handle, color::Fg(color::Reset)
            ));
            result.push(format!(
                "        {}",
                qt.text.replace("\r", "\\r").split("\n").collect::<Vec<&str>>().join("\n        ")
            ));
        } else {
            result.push(format!("    << don't have quoted tweet! >>"));
        }
    }

    result
}
