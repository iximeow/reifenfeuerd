extern crate serde_json;

pub enum Event {
    Deleted { user_id: String, twete_id: String },
    RT_RT { user_id: String, twete_id: String },
    Fav_RT { user_id: String, twete_id: String },
    Fav { user_id: String, twete_id: String },
    Unfav { user_id: String, twete_id: String },
    Quoted { user_id: String, twete_id: String },
    Followed { user_id: String },
    Unfollowed { user_id: String }
}

impl Event {
    pub fn from_json(structure: serde_json::Map<String, serde_json::Value>) -> Option<Event> {
        match &structure["event"].as_str().unwrap() {
            &"follow" => Some(Event::Followed {
                user_id: structure["source"]["id_str"].as_str().unwrap().to_owned()
            }),
            &"unfollow" => Some(Event::Unfollowed {
                user_id: structure["source"]["id_str"].as_str().unwrap().to_owned()
            }),
            &"favorite" => Some(Event::Fav {
                user_id: structure["source"]["id_str"].as_str().unwrap().to_owned(),
                twete_id: structure["target_object"]["id_str"].as_str().unwrap().to_owned()
            }),
            &"unfavorite" => Some(Event::Unfav {
                user_id: structure["source"]["id_str"].as_str().unwrap().to_owned(),
                twete_id: structure["target_object"]["id_str"].as_str().unwrap().to_owned()
            }),
            &"favorited_retweet" => Some(Event::Fav_RT {
                user_id: structure["source"]["id_str"].as_str().unwrap().to_owned(),
                twete_id: structure["target_object"]["id_str"].as_str().unwrap().to_owned()
            }),
            &"retweeted_retweet" => Some(Event::RT_RT {
                user_id: structure["source"]["id_str"].as_str().unwrap().to_owned(),
                twete_id: structure["target_object"]["id_str"].as_str().unwrap().to_owned()
            }),
            &"quoted_tweet" => Some(Event::Quoted {
                user_id: structure["source"]["id_str"].as_str().unwrap().to_owned(),
                twete_id: structure["target_object"]["id_str"].as_str().unwrap().to_owned()
            }),
//                &"list_member_added" =>
//                what about removed?
//                &"blocked" => Blocked { },
//                &"unblocked" => Unblocked { },
            e => { println!("unrecognized event: {}", e); None }
        }
    }
}
