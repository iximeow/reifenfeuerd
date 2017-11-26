extern crate termion;

use std::io::Write;
use std::io::stdout;

use self::termion::color;
use self::termion::{clear, cursor};

use ::tw;

use ::tw::TweetId;

use std;

#[derive(Clone)]
pub enum DisplayMode {
    Compose(String),
    Reply(TweetId, String)
}

#[derive(Clone)]
pub enum Infos {
    Tweet(TweetId),
    TweetWithContext(TweetId, String),
    Thread(Vec<TweetId>),
    Event(tw::events::Event),
    DM(String),
    User(tw::user::User),
    Text(Vec<String>)
}

const COMPOSE_HEIGHT: u16 = 5;

pub struct DisplayInfo {
    pub log_height: u16,
    pub prompt_height: u16,
    pub mode: Option<DisplayMode>,
    pub log_seek: u32,
    pub infos_seek: u32,
    pub log: Vec<String>,
    pub infos: Vec<Infos>,
    pub input_buf: Vec<char>
}

impl Default for DisplayInfo {
    fn default() -> Self {
        DisplayInfo {
            log_height: 4,
            prompt_height: 3,
            mode: None,
            log_seek: 0,
            infos_seek: 0,
            log: Vec::new(),
            infos: Vec::new(),
            input_buf: Vec::new()
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

    pub fn ui_height(&self) -> u16 {
        self.log_height + self.prompt_height
    }
}

/*
 * wraps x so each line is width or fewer characters, after splitting by \n.
 */
fn into_display_lines(x: Vec<String>, width: u16) -> Vec<String> {
    let split_on_newline: Vec<String> = x.into_iter()
        .flat_map(|x| x.split("\n")
            .map(|x| x.to_owned())
            .collect::<Vec<String>>()
        ).collect();
    let wrapped: Vec<String> = split_on_newline.iter()
        .map(|x| x.chars().collect::<Vec<char>>())
        .flat_map(|x| x.chunks(width as usize)
            .map(|x| x.into_iter().collect::<String>())
            .collect::<Vec<String>>())
        .collect();
    wrapped
}

pub fn paint(tweeter: &::tw::TwitterCache, display_info: &mut DisplayInfo) -> Result<(), std::io::Error> {
    match termion::terminal_size() {
        Ok((width, height)) => {
            // draw input prompt
            let mut i = 0;
            let log_size = 4;
            let last_elem = display_info.log.len().saturating_sub(log_size);
            {
                let to_show = display_info.log[last_elem..].iter().rev();
                for line in to_show {
                    print!("{}{}{}/{}: {}", cursor::Goto(1, height - i), clear::CurrentLine, display_info.log.len() - 1 - i as usize, display_info.log.len() - 1, line);
                    i = i + 1;
                }
            }
            while i < log_size as u16 {
                print!("{}{}", cursor::Goto(1, height - i), clear::CurrentLine);
                i = i + 1;
            }
            // draw status lines
            // draw tweets
            let last_tail_twevent = display_info.infos.len().saturating_sub(display_info.infos_seek as usize);
            let first_tail_twevent = last_tail_twevent.saturating_sub(height as usize - 4);
            let last_few_twevent: Vec<Infos> = display_info.infos[first_tail_twevent..last_tail_twevent].iter().map(|x| x.clone()).rev().collect::<Vec<Infos>>();

            let mut h = display_info.ui_height();

            /*
             * draw in whatever based on mode...
             */
            let (cursor_x, cursor_y) = match display_info.mode.clone() {
                None => {
                    let handle = tweeter.current_profile().map(|profile| profile.user.handle.to_owned()).unwrap_or("_default_".to_owned());
                    print!("{}{}", cursor::Goto(1, height - 6), clear::CurrentLine);
                    print!("{}{}@{}>{}", cursor::Goto(1, height - 5), clear::CurrentLine, handle, display_info.input_buf.clone().into_iter().collect::<String>());
                    print!("{}{}", cursor::Goto(1, height - 4), clear::CurrentLine);
                    ((1 + handle.len() + 2 + display_info.input_buf.len()) as u16, height as u16 - 5)
                }
                Some(DisplayMode::Compose(x)) => {
                    let mut lines: Vec<String> = vec![];
                    let msg_lines = into_display_lines(x.split("\n").map(|x| x.to_owned()).collect(), width - 2);
                    let cursor_idx = msg_lines.last().map(|x| x.len()).unwrap_or(0);
                    lines.extend(msg_lines);
                    if lines.len() == 0 {
                        lines.push("".to_owned());
                    }
                    // TODO: properly probe tweet length lim
                    lines.push(format!("{}/{}", x.len(), 140));
                    lines.insert(0, "".to_owned());
                    let mut lines_drawn: u16 = 0;
                    for line in lines.into_iter().rev() {
                        print!("{}{}  {}{}{}{}",
                            cursor::Goto(1, height - 4 - lines_drawn), clear::CurrentLine,
                            color::Bg(color::Blue), line, std::iter::repeat(" ").take((width as usize).saturating_sub(line.len() + 2)).collect::<String>(), termion::style::Reset
                        );
                        lines_drawn += 1;
                    }
                    h += lines_drawn - 3;
                    (cursor_idx as u16 + 3, height as u16 - 5) // TODO: panic on underflow
                }
                Some(DisplayMode::Reply(twid, msg)) => {
                    let mut lines = render_twete(&twid, tweeter, display_info, Some(width));
                    lines.push("  --------  ".to_owned());
                    let msg_lines = into_display_lines(msg.split("\n").map(|x| x.to_owned()).collect(), width - 2);
                    let cursor_idx = msg_lines.last().map(|x| x.len()).unwrap_or(0);
                    if msg_lines.len() == 0 {
                        lines.push("".to_owned());
                    }
                    lines.extend(msg_lines);
                    // TODO: properly probe tweet length lim
                    lines.push(format!("{}/{}", msg.len(), 140));
                    lines.insert(0, "".to_owned());
                    let mut lines_drawn: u16 = 0;
                    for line in lines.into_iter().rev() {
                        print!("{}{}  {}{}{}{}",
                            cursor::Goto(1, height - 4 - lines_drawn), clear::CurrentLine,
                            color::Bg(color::Blue), line, std::iter::repeat(" ").take((width as usize).saturating_sub(line.len() + 2)).collect::<String>(), termion::style::Reset
                        );
                        lines_drawn += 1;
                    }
                    h += lines_drawn - 3;
                    (cursor_idx as u16 + 3, height as u16 - 5) // TODO: panic on underflow
                }
            };

            for info in last_few_twevent {
                let to_draw: Vec<String> = match info {
                    Infos::Text(lines) => {
                        let wrapped = into_display_lines(lines, width);
                        wrapped.into_iter().rev().collect()
                    }
                    Infos::Tweet(id) => {
                        let pre_split: Vec<String> = render_twete(&id, tweeter, display_info, Some(width));
                        let total_length: usize = pre_split.iter().map(|x| x.len()).sum();
                        let wrapped = if total_length <= 1024 {
                            into_display_lines(pre_split, width)
                        } else {
                            vec!["This tweet discarded for your convenience".to_owned()]
                        };
                        wrapped.into_iter().rev().collect()
                    }
                    Infos::TweetWithContext(id, context) => {
                        let mut lines = render_twete(&id, tweeter, display_info, Some(width)).iter().map(|x| x.to_owned()).rev().collect::<Vec<String>>();
                        lines.push(context);
                        lines
                    }
                    Infos::Thread(ids) => {
                        // TODO: group together thread elements by the same person a little
                        // better..
                        let mut tweets: Vec<Vec<String>> = ids.iter().rev().map(|x| render_twete(x, tweeter, display_info, Some(width))).collect();
                        let last = tweets.pop();
                        let mut lines = tweets.into_iter().fold(Vec::new(), |mut sum, lines| {
                            sum.extend(lines);
                            sum.extend(vec![
                                "      ^".to_owned(),
                                "      |".to_owned()
                            ]);
                            sum
                        });
                        if let Some(last_lines) = last {
                            lines.extend(last_lines);
                        }
                        //let mut lines = vec![format!("{}{}I'd show a thread if I knew how", cursor::Goto(1, height - h), clear::CurrentLine)];
                        lines.push("".to_owned());
                        lines.into_iter().rev().collect()
                    },
                    Infos::Event(e) => {
                        let pre_split = e.clone().render(tweeter, display_info, width);
                        let total_length: usize = pre_split.iter().map(|x| x.len()).sum();
                        let wrapped = if total_length <= 1024 {
                            into_display_lines(pre_split, width)
                        } else {
                            vec!["This tweet discarded for your convenience".to_owned()]
                        };
                        wrapped.into_iter().rev().collect()
                    },
                    Infos::DM(msg) => {
                        let mut lines = vec![format!("{}{}{} DM:", cursor::Goto(1, height - h), clear::CurrentLine, "from")];
                        lines.push(msg);
                        lines
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
                        break;
                    }
                }
                print!("{}{}", cursor::Goto(1, height - h), clear::CurrentLine);
                h = h + 1;
                if h >= height {
                    break;
                }
            }
            while h < height {
                print!("{}{}", cursor::Goto(1, height - h), clear::CurrentLine);
                h = h + 1;
            }
            print!("{}", cursor::Goto(cursor_x, cursor_y));
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
    fn render(self, tweeter: &::tw::TwitterCache, display_info: &mut DisplayInfo, width: u16) -> Vec<String>;
}

impl Render for tw::events::Event {
    fn render(self, tweeter: &::tw::TwitterCache, display_info: &mut DisplayInfo, width: u16) -> Vec<String> {
        let mut result = Vec::new();
        match self {
            tw::events::Event::Quoted { user_id, twete_id } => {
                result.push("---------------------------------".to_string());
                {
                    let user = tweeter.retrieve_user(&user_id).unwrap();
                    result.push(format!("  quoted_tweet    : {} (@{})", user.name, user.handle));
                }
                render_twete(&TweetId::Twitter(twete_id), tweeter, display_info, Some(width));
            }
            tw::events::Event::Deleted { user_id, twete_id } => {
                if let Some(handle) = tweeter.retrieve_user(&user_id).map(|x| &x.handle).map(|x| x.clone()) {
                    if let Some(_tweet) = tweeter.retrieve_tweet(&TweetId::Twitter(twete_id.to_owned()), display_info).map(|x| x.clone()) {
                        result.push(format!("-------------DELETED------------------"));
                        result.extend(render_twete(&TweetId::Twitter(twete_id), tweeter, display_info, Some(width)));
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
                result.extend(render_twete(&TweetId::Twitter(twete_id), tweeter, display_info, Some(width)));
            },
            tw::events::Event::Fav_RT { user_id, twete_id } => {
                result.push("---------------------------------".to_string());
                if let Some(user) = tweeter.retrieve_user(&user_id) {
                    result.push(format!("  +rt_fav   : {} (@{})", user.name, user.handle));
                } else {
                    result.push(format!("  +rt_fav but don't know who {} is", user_id));
                }
                result.extend(render_twete(&TweetId::Twitter(twete_id), tweeter, display_info, Some(width)));
            },
            tw::events::Event::Fav { user_id, twete_id } => {
                result.push("---------------------------------".to_string());
                {
                let user = tweeter.retrieve_user(&user_id).unwrap();
                result.push(format!("{}  +fav      : {} (@{}){}", color::Fg(color::Yellow), user.name, user.handle, color::Fg(color::Reset)));
                }
                result.extend(render_twete(&TweetId::Twitter(twete_id), tweeter, display_info, Some(width)));
            },
            tw::events::Event::Unfav { user_id, twete_id } => {
                result.push("---------------------------------".to_string());
                {
                let user = tweeter.retrieve_user(&user_id).unwrap();
                result.push(format!("{}  -fav      : {} (@{}){}", color::Fg(color::Yellow), user.name, user.handle, color::Fg(color::Reset)));
                }
                result.extend(render_twete(&TweetId::Twitter(twete_id), tweeter, display_info, Some(width)));
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

// really dumb...
fn pad_lines(lines: Vec<String>, padding: &str) -> Vec<String> {
    lines.into_iter().map(|x| format!("{}{}", padding, x)).collect()
}

pub fn render_twete(twete_id: &TweetId, tweeter: &tw::TwitterCache,  display_info: &mut DisplayInfo, width: Option<u16>) -> Vec<String> {
    let mut lines = render_twete_no_recurse(twete_id, tweeter, display_info, width);
    match tweeter.retrieve_tweet(twete_id, display_info).map(|x| x.clone()) {
        Some(twete) => {
            if let Some(ref qt_id) = twete.quoted_tweet_id {
                lines.extend(pad_lines(render_twete_no_recurse(&TweetId::Twitter(qt_id.to_owned()), tweeter, display_info, width.map(|x| x - 4)), "    "));
            }
        },
        None => { /* awkward */ }
    }
    lines
}

pub fn render_twete_no_recurse(twete_id: &TweetId, tweeter: &tw::TwitterCache, display_info: &mut DisplayInfo, width: Option<u16>) -> Vec<String> {
    // ~~ reactive ~~ layout if the terminal isn't wide enough? for now just wrap to passed width
    let mut result = Vec::new();
    let id_color = color::Fg(color::Rgb(180, 80, 40));
    match tweeter.retrieve_tweet(twete_id, display_info).map(|x| x.clone()) {
        Some(twete) => {
            // if we got the tweet, the API gave us the user too
            let user = tweeter.retrieve_user(&twete.author_id).map(|x| x.clone()).unwrap();
            /*
             * two cases here:
             * A: just a tweet, no retweet. So, show:
             *      id <tweet_id>
             *  . . . ok.
             *
             * B: X retweeted Y. We got the retweet Z, so "id" is the retweet id.
             * Want to show:
             *      id Y (rt id Z)
             *      <Y author> via <Z RTer>
             *
             * which means for B, "id" is ".rt_tweet"!
             */
            let (tweet_id, maybe_rt_id) = twete.rt_tweet
                .clone()
                .map(|rt_id| (TweetId::Twitter(rt_id), Some(TweetId::Twitter(twete.id.to_owned())))).unwrap_or((TweetId::Twitter(twete.id.to_owned()), None));
            // retrieve_* taking mut tweeter REALLY messes stuff up.
            let tweet = tweeter.retrieve_tweet(&tweet_id, display_info).unwrap().clone();
            let tweet_author = tweeter.retrieve_user(&tweet.author_id).unwrap().clone();

            // now we've unfurled it so id is the original tweet either way, maybe_rt_id is the id
            // of the retweeter tweet if it's there

            let mut id_string = format!("{}id {}", id_color, tweet.internal_id);
            let mut author_string = format!("{}{}{} ({}@{}{})", color_for(&tweet_author.handle), tweet_author.name, color::Fg(color::Reset), color_for(&tweet_author.handle), tweet_author.handle, color::Fg(color::Reset));

            if let Some(reply_id) = tweet.reply_to_tweet.clone() {
                let reply_tweet_id = match tweeter.retrieve_tweet(&TweetId::Twitter(reply_id.to_owned()), display_info) {
                    Some(reply_tweet) => TweetId::Bare(reply_tweet.internal_id),
                    None => TweetId::Twitter(reply_id)
                };
                id_string.push_str(&format!(" reply to {}", reply_tweet_id))
            }

            if let Some(rt_id) = maybe_rt_id {
                let rt = tweeter.retrieve_tweet(&rt_id, display_info).unwrap().clone();
                let rt_author = tweeter.retrieve_user(&rt.author_id).unwrap().clone();
                id_string.push_str(&format!(" (rt id {})", rt.internal_id));
                author_string.push_str(&format!(" via {}{}{} ({}@{}{}) RT", color_for(&rt_author.handle), rt_author.name, color::Fg(color::Reset), color_for(&rt_author.handle), rt_author.handle, color::Fg(color::Reset)));
            }

            id_string.push_str(&format!("{}", color::Fg(color::Reset)));

            result.push(id_string);
            result.push(author_string);

            let mut original_lines: Vec<String> = twete.text
//                .replace("\r", "\\r")
                .split("\n").map(|line| line.replace("\r", "\\r")).collect();

            let mut urls_to_include: Vec<&str> = vec![];
            let urls_replaced = original_lines
                .into_iter()
                .map(|line| {
                    let mut result: String = line.to_owned();
                    for elem in tweet.urls.iter() {
                        if (result.ends_with(elem.0) || result.ends_with(elem.1)) && twete.quoted_tweet_id.clone().map(|id| elem.1.ends_with(&id)).unwrap_or(false) {
                            // replace with nothing! this dumb url is the quoted tweet url.
                            if result.ends_with(elem.0) {
                                result = result.replace(&format!(" {}", elem.0), "");
                            } else {
                                result = result.replace(&format!(" {}", elem.1), "");
                            }
                        } else {
                            if line.contains(elem.0) {
                                result = result.replace(elem.0, &format!("[{}]", urls_to_include.len()));
                                urls_to_include.push(elem.0);
                            }
                        }
                    }
                    result
                })
                .collect();

            let renderable_lines = match width {
                Some(width) => {
                    let mut text = pad_lines(into_display_lines(urls_replaced, width - 2), "  ");
                    for (i, short_url) in urls_to_include.into_iter().enumerate() {
                        let expanded = &tweet.urls[short_url];
                        // elem.0 is short url, elem.1 is expanded url
                        // TODO: can panic if width is really small
                        if expanded.len() < (width - 9) as usize { // "[XX]: " is 6 + some padding space?
                            text.push(format!("[{}]: {}", i, expanded));
                        } else {
                            // TODO: try to just show domain, THEN fall back to just a link if the
                            // domain is too long
                            text.push(format!("[{}]: {}", i, short_url));
                        }
                    }
                    text
                },
                None => urls_replaced
            };

            result.extend(renderable_lines);
        },
        None => {
            result.push(format!("No such tweet: {:?}", twete_id));
        }
    }

    result
}
