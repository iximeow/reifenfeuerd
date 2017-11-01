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
    TweetWithContext(TweetId, String),
    Thread(Vec<TweetId>),
    Event(tw::events::Event),
    DM(String),
    User(tw::user::User)
}

pub struct DisplayInfo {
    pub log_seek: u32,
    pub infos_seek: u32,
    pub log: Vec<String>,
    pub infos: Vec<Infos>
}

impl Default for DisplayInfo {
    fn default() -> Self {
        DisplayInfo {
            log_seek: 0,
            infos_seek: 0,
            log: Vec::new(),
            infos: Vec::new()
        }
    }
}

impl DisplayInfo {
    pub fn status(&mut self, stat: String) {
        self.log.push(stat);
    }

    pub fn recv(&mut self, info: Infos) {
        self.infos.push(info);
    }
}

pub fn paint(tweeter: &mut ::tw::TwitterCache) -> Result<(), std::io::Error> {
    match termion::terminal_size() {
        Ok((_width, height)) => {
            // draw input prompt
            print!("{}{}", cursor::Goto(1, height - 6), clear::CurrentLine);
            print!("{}{}>", cursor::Goto(1, height - 5), clear::CurrentLine);
            print!("{}{}", cursor::Goto(1, height - 4), clear::CurrentLine);
            let mut i = 0;
            let log_size = 4;
            let last_elem = tweeter.display_info.log.len().saturating_sub(log_size);
            {
                let to_show = tweeter.display_info.log[last_elem..].iter().rev();
                for line in to_show {
                    print!("{}{}{}/{}: {}", cursor::Goto(1, height - i), clear::CurrentLine, tweeter.display_info.log.len() - 1 - i as usize, tweeter.display_info.log.len() - 1, line);
                    i = i + 1;
                }
            }
            while i < log_size as u16 {
                print!("{}{}", cursor::Goto(1, height - i), clear::CurrentLine);
                i = i + 1;
            }
            // draw status lines
            // draw tweets
            let last_twevent = tweeter.display_info.infos.len().saturating_sub(height as usize - 4).saturating_sub(tweeter.display_info.infos_seek as usize);
            let last_few_twevent: Vec<Infos> = tweeter.display_info.infos[last_twevent..].iter().map(|x| x.clone()).rev().collect::<Vec<Infos>>();

            let mut h = 7;
            for info in last_few_twevent {
                let to_draw: Vec<String> = match info {
                    Infos::Tweet(id) => {
                        let pre_split: Vec<String> = render_twete(&id, tweeter);
                        let split_on_newline: Vec<String> = pre_split.into_iter().flat_map(|x| x.split("\n").map(|x| x.to_owned()).collect::<Vec<String>>()).collect();
                        let wrapped: Vec<String> = split_on_newline.iter()
                            .map(|x| x.chars().collect::<Vec<char>>())
                            .flat_map(|x| x.chunks(_width as usize)
                                .map(|x| x.into_iter().collect::<String>())
                                .collect::<Vec<String>>())
                            .collect();
                        wrapped.into_iter().rev().collect()
                    }
                    Infos::TweetWithContext(id, context) => {
                        let mut lines = render_twete(&id, tweeter).iter().map(|x| x.to_owned()).rev().collect::<Vec<String>>();
                        lines.push(context);
                        lines
                    }
                    Infos::Thread(_ids) => {
                        let mut lines = vec![format!("{}{}I'd show a thread if I knew how", cursor::Goto(1, height - h), clear::CurrentLine)];
                        lines.push("".to_owned());
//                        lines.push(format!("link: https://twitter.com/i/web/status/{}", id));
                        lines
                    },
                    Infos::Event(e) => {
                        e.clone().render(tweeter).into_iter().rev().collect()
                    },
                    Infos::DM(msg) => {
                        vec![format!("{}{}DM: {}", cursor::Goto(1, height - h), clear::CurrentLine, msg)]
                    }
                    Infos::User(user) => {
                        vec![
                            format!("{} (@{})", user.name, user.handle)
                        ]
                    }
                };
                for line in to_draw {
                    print!("{}{}{}", cursor::Goto(1, height - h), clear::CurrentLine, line);
                    h = h + 1;
                    if h >= height {
                        print!("{}", cursor::Goto(2, height - 6));
                        return stdout().flush();
                    }
                }
                print!("{}{}", cursor::Goto(1, height - h), clear::CurrentLine);
                h = h + 1;
                if h >= height {
                    print!("{}", cursor::Goto(2, height - 6));
                    return stdout().flush();
                }
            }
            while h < height {
                print!("{}{}", cursor::Goto(1, height - h), clear::CurrentLine);
                h = h + 1;
            }
            print!("{}", cursor::Goto(2, height - 5));
            stdout().flush()?;
        },
        Err(e) => {
            println!("Can't get term dimensions: {}", e);
        }
    }
    Ok(())
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
                    result.push(format!("  quoted_tweet    : {} (@{})", user.name, user.handle));
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
    match tweeter.retrieve_tweet(twete_id).map(|x| x.clone()) {
        Some(twete) => {
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
        },
        None => {
            result.push(format!("No such tweet: {:?}", twete_id));
        }
    }

    result
}
