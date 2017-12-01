extern crate termion;

use std::io::Write;
use std::io::stdout;

use std::iter::Iterator;
use std::fmt;

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
    ansi_aware_into_display_lines(x, width)
    /*
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
     */
}

#[derive(Clone)]
enum AnsiInfo {
    Esc,
    EscBracket,
    CSI(String), // CSI <n_string> with no tailing character ... yet?
    FullSequence(String, char) // CSI n_string param
}

impl fmt::Display for AnsiInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> fmt::Result {
        match self {
            &AnsiInfo::Esc => {
                write!(f, "\x1b")
            },
            &AnsiInfo::EscBracket => {
                write!(f, "\x1b[")
            },
            &AnsiInfo::CSI(ref n) => {
                write!(f, "\x1b[{}", n)
            },
            &AnsiInfo::FullSequence(ref n, ref c) => {
                write!(f, "\x1b[{}{}", n, c)
            }
        }
    }
}

#[derive(Clone)]
struct TextState {
    color: Option<String>, // Box<termion::color::Color>>,
    underline: bool,
    italic: bool
}

/*
 * wraps x so each line is width for fewer displayed characters
 * (this probably doesn't work for zero width unicode symbols)
 *
 * preserves coloration of the string across splits:
 *                       | <-- wrap here
 * "hello talking to \x1b[5m@som\x1b[0m"
 * "\x1b[5mename\x1b[0m"
 */
#[cfg(test)]
mod tests {
    #[test]
    fn ansi_display_lines_test() {
        let initial = "hello talking to \x1b[5m@somename\x1b[0m".to_owned();
        let split = ::display::ansi_aware_into_display_lines(vec![initial], 22);
        assert_eq!(split.len(), 2);
        assert_eq!(split[0], "hello talking to \x1b[5m@some\x1b[0m");
        assert_eq!(split[1], "\x1b[5mname\x1b[0m");
    }
}
fn ansi_aware_into_display_lines(x: Vec<String>, width: u16) -> Vec<String> {
    let mut current_color: Option<u8> = None;
    let mut ansi_code: Option<AnsiInfo> = None;
    let mut text_state: Option<TextState> = None;
    let mut display_len: u16 = 0;
    let mut split_lines = Vec::new();
    if x.len() == 0 {
        return split_lines;
    } else {
        split_lines.push(String::new());
    }
    for (i, line) in x.iter().enumerate() {
        for chr in line.chars() {
            let addend = match chr {
                '\x1b' => {
                    match ansi_code.clone() {
                        None => {
                            ansi_code = Some(AnsiInfo::Esc);
                            "".to_owned()
                        }
                        Some(ansi) => {
                            ansi_code = Some(AnsiInfo::Esc);
                            format!("{}", ansi)
                        }
                    }
                },
                '[' => {
                    match ansi_code.clone() {
                        Some(AnsiInfo::Esc) => {
                            ansi_code = Some(AnsiInfo::EscBracket);
                            "".to_owned()
                        },
                        Some(info @ AnsiInfo::EscBracket) => {
                            format!("{}[", info)
                        },
                        Some(info @ AnsiInfo::CSI(_)) => {
                            format!("{}[", info)
                        },
                        Some(info @ AnsiInfo::FullSequence(_, _)) => {
                            format!("{}[", info)
                        },
                        None => {
                            "[".to_owned()
                        }
                    }
                },
                c @ '0'...'9' => {
                    match ansi_code.clone() {
                        Some(AnsiInfo::EscBracket) => {
                            ansi_code = Some(AnsiInfo::CSI(c.to_string()));
                            "".to_owned()
                        },
                        Some(info @ AnsiInfo::FullSequence(_, _)) => {
                            ansi_code = None;
                            format!("{}{}", info, c)
                        }
                        Some(AnsiInfo::CSI(mut n)) => {
                            n.push(c);
                            ansi_code = Some(AnsiInfo::CSI(n));
                            "".to_owned()
                        },
                        Some(AnsiInfo::Esc) => {
                            //  TODO: flush
                            ansi_code = None;
                            format!("{}{}", AnsiInfo::Esc, c)
                        },
                        None => {
                            c.to_string()
                        }
                    }
                },
                ';' => {
                    match ansi_code.clone() {
                        Some(info @ AnsiInfo::FullSequence(_, _)) => {
                            ansi_code = None;
                            format!("{};", info)
                        }
                        Some(AnsiInfo::EscBracket) => {
                            ansi_code = None;
                            format!("{};", AnsiInfo::EscBracket)
                        },
                        Some(AnsiInfo::CSI(n)) => {
                            ansi_code = Some(AnsiInfo::CSI(format!("{};", n)));
                            "".to_string()
                        },
                        Some(AnsiInfo::Esc) => {
                            ansi_code = None;
                            format!("{};", AnsiInfo::Esc)
                        },
                        None => {
                            ';'.to_string()
                        }
                    }
                },
                c => {
                    match ansi_code.clone() {
                        Some(info @ AnsiInfo::FullSequence(_, _)) => {
                            panic!("This should not be reachable - a FullSequence should be flushed immediately after construction.");
                        }
                        Some(AnsiInfo::EscBracket) => {
                            ansi_code = Some(AnsiInfo::FullSequence("".to_owned(), c));
                            "".to_string()
                        },
                        Some(AnsiInfo::CSI(n)) => {
                            ansi_code = Some(AnsiInfo::FullSequence(n, c));
                            "".to_string()
                        },
                        Some(AnsiInfo::Esc) => {
                            ansi_code = None;
                            format!("{}{}", AnsiInfo::Esc, c)
                        },
                        None => {
                            c.to_string()
                        }
                    }
                }
            };

            // if we've produced a full sequence, dump that to the string and set that as the
            // curret info
            // 
            // TODO: support ansi sequences other than m aka colors.

            if let Some(AnsiInfo::FullSequence(n, c)) = ansi_code.clone() {
                // this is not printable so we don't advance the printable text counter
                split_lines.last_mut().unwrap().push_str(&format!("\x1b[{}{}", n, c));
                text_state = match text_state {
                    None => {
                        if n != "0" && n != "" {
                            Some(TextState {
                                color: Some(n),
                                underline: false,
                                italic: false
                            })
                        } else {
                            None
                        }
                    },
                    Some(mut state) => {
                        if n == "0" || n == "" {
                            state.color = None;
                        } else {
                            state.color = Some(n);
                        };
                        Some(state)
                    }
                };
                ansi_code = None;
            }

            for chr in addend.chars() {
                // If we're adding a new character, see if we have to add a new line
                if display_len == width || chr == '\n' {
                    match &text_state {
                        &Some(ref state) => {
                            split_lines.last_mut().unwrap().push_str("\x1b[0m");
                            split_lines.push(String::new());
                            split_lines.last_mut().unwrap().push_str(&format!("\x1b[{}m", state.color.clone().unwrap_or("".to_owned())));
                            display_len = 0;
                        }
                        &None => {
                            split_lines.push(String::new());
                            display_len = 0;
                        }
                    }
                }
                // whatever happened, we're now ready to add a character
                split_lines.last_mut().unwrap().push(chr);
                display_len += 1;
            }
        }

        if i < x.len() - 1 {
            match &text_state {
                &Some(ref state) => {
                    split_lines.last_mut().unwrap().push_str("\x1b[0m");
                    split_lines.push(String::new());
                    split_lines.last_mut().unwrap().push_str(&format!("\x1b[{}m", state.color.clone().unwrap_or("".to_owned())));
                    display_len = 0;
                }
                &None => {
                    split_lines.push(String::new());
                    display_len = 0;
                }
            }
        }
    }
    split_lines
}

pub fn paint(tweeter: &::tw::TwitterCache, display_info: &mut DisplayInfo) -> Result<(), std::io::Error> {
    match termion::terminal_size() {
        Ok((width, height)) => {
            // draw input prompt
            let mut i = 0;
            let log_size = 4;
            let last_tail_log = display_info.log.len().saturating_sub(display_info.infos_seek as usize);
            let first_tail_log = last_tail_log.saturating_sub(log_size);
            {
                let to_show = display_info.log[first_tail_log..last_tail_log].iter().rev();
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
                    lines.push(std::iter::repeat("-").take((width as usize).saturating_sub(2)).collect());
                    lines.extend(msg_lines);
                    if lines.len() == 0 {
                        lines.push("".to_owned());
                    }
                    // TODO: properly probe tweet length lim
                    let counter = format!("{}/{}", x.len(), 280);
                    lines.push(format!("{}{}", counter, std::iter::repeat("-").take((width as usize).saturating_sub(counter.len() + 2)).collect::<String>()));
                    lines.insert(0, "".to_owned());
                    let mut lines_drawn: u16 = 0;
                    for line in lines.into_iter().rev() {
                        print!("{}{}  {}",
                            cursor::Goto(1, height - 4 - lines_drawn), clear::CurrentLine,
                            line //, std::iter::repeat(" ").take((width as usize).saturating_sub(line.len() + 2)).collect::<String>()
                        );
                        lines_drawn += 1;
                    }
                    h += lines_drawn - 3;
                    (cursor_idx as u16 + 3, height as u16 - 5) // TODO: panic on underflow
                }
                Some(DisplayMode::Reply(twid, msg)) => {
                    let mut lines: Vec<String> = vec![];
                    lines.push(std::iter::repeat("-").take((width as usize).saturating_sub(2)).collect());
                    lines.extend(render_twete(&twid, tweeter, display_info, Some(width)));
                    let reply_delineator = "--------reply";
                    lines.push(format!("{}{}", reply_delineator, std::iter::repeat("-").take((width as usize).saturating_sub(reply_delineator.len() + 2)).collect::<String>()));
                    let msg_lines = into_display_lines(msg.split("\n").map(|x| x.to_owned()).collect(), width - 2);
                    let cursor_idx = msg_lines.last().map(|x| x.len()).unwrap_or(0);
                    if msg_lines.len() == 0 {
                        lines.push("".to_owned());
                    }
                    lines.extend(msg_lines);
                    // TODO: properly probe tweet length lim
                    let counter = format!("{}/{}", msg.len(), 280);
                    lines.push(format!("{}{}", counter, std::iter::repeat("-").take((width as usize).saturating_sub(counter.len() + 2)).collect::<String>()));
                    lines.insert(0, "".to_owned());
                    let mut lines_drawn: u16 = 0;
                    for line in lines.into_iter().rev() {
                        print!("{}{}  {}",
                            cursor::Goto(1, height - 4 - lines_drawn), clear::CurrentLine,
                            line //, std::iter::repeat(" ").take((width as usize).saturating_sub(line.len() + 2)).collect::<String>()
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
                            vec![format!("This tweet discarded for your convenience: (id: {})", id)]
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
                    if let Some(_tweet) = tweeter.retrieve_tweet(&TweetId::Twitter(twete_id.to_owned())).map(|x| x.clone()) {
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
    match tweeter.retrieve_tweet(twete_id).map(|x| x.clone()) {
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
    match tweeter.retrieve_tweet(twete_id).map(|x| x.clone()) {
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
            let tweet = tweeter.retrieve_tweet(&tweet_id).unwrap().clone();
            let tweet_id = tweeter.display_id_for_tweet(&tweet);
            let tweet_author = tweeter.retrieve_user(&tweet.author_id).unwrap().clone();

            // now we've unfurled it so id is the original tweet either way, maybe_rt_id is the id
            // of the retweeter tweet if it's there

            let mut id_string = format!("{}id {}", id_color, tweet_id);
            let mut author_string = format!("{}{}{} ({}@{}{})", color_for(&tweet_author.handle), tweet_author.name, color::Fg(color::Reset), color_for(&tweet_author.handle), tweet_author.handle, color::Fg(color::Reset));

            if let Some(reply_id) = tweet.reply_to_tweet.clone() {
                let reply_tweet_id = tweeter.display_id_for_tweet_id(&TweetId::Twitter(reply_id.to_owned()));
                id_string.push_str(&format!(" reply to {}", reply_tweet_id))
            }

            if let Some(rt_id) = maybe_rt_id {
                let rt = tweeter.retrieve_tweet(&rt_id).unwrap().clone();
                let rt_id = tweeter.display_id_for_tweet(&rt);
                let rt_author = tweeter.retrieve_user(&rt.author_id).unwrap().clone();
                id_string.push_str(&format!(" (rt id {})", rt_id));
                author_string.push_str(&format!(" via {}{}{} ({}@{}{}) RT", color_for(&rt_author.handle), rt_author.name, color::Fg(color::Reset), color_for(&rt_author.handle), rt_author.handle, color::Fg(color::Reset)));
            }

            id_string.push_str(&format!("{}", color::Fg(color::Reset)));

            result.push(id_string);
            result.push(author_string);

            let raw_lines: Vec<String> = twete.text
                .split("\n").map(|line| line.replace("\r", "\\r")).collect();

            // now colorize @'s:
            let mut colorized_lines: Vec<String> = vec![];

            for line in raw_lines {
                let mut name: Option<String> = None;
                let mut new_line = String::new();
                for c in line.chars() {
                    name = match name {
                        Some(mut handle) => {
                            match c {
                                'a'...'z' | 'A'...'Z' | '0'...'9' | '_' => {
                                    // so if we have a handle WIP, append to that string.
                                    handle.push(c);
                                    Some(handle)
                                },
                                c => {
                                    // we HAD a handle, this just terminated it.
                                    // if it was empty string, it's not really a mention, we can
                                    // discard it.
                                    if handle.len() > 0 {
                                        new_line.push_str(&format!("{}@{}{}{}", color_for(&handle), &handle, termion::style::Reset, c));
                                    } else {
                                        new_line.push('@');
                                        new_line.push(c);
                                    }
                                    None
                                }
                            }
                        },
                        None => {
                            if c == '@' {
                                Some(String::new())
                            } else {
                                new_line.push(c);
                                None
                            }
                        }
                    }
                }
                if let Some(mut handle) = name {
                    new_line.push_str(&format!("{}@{}{}", color_for(&handle), &handle, termion::style::Reset));
                }
                colorized_lines.push(new_line);
            }

            let mut urls_to_include: Vec<&str> = vec![];
            let urls_replaced = colorized_lines
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
