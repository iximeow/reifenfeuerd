extern crate serde_json;

#[derive(Clone)]
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
    fn get_source_id(structure: serde_json::Map<String, serde_json::Value>) -> Result<String, String> {
        match structure.get("source").and_then(|x| x.get("id_str").and_then(|x| x.as_str())) {
            Some(id) => Ok(id.to_string()),
            None => Err("No id_str string at .source.id_str".to_string())
        }
    }
    fn get_source_target_ids(structure: serde_json::Map<String, serde_json::Value>) -> Result<(String, String), String> {
        match (
            structure.get("source").and_then(|x| x.get("id_str").and_then(|x| x.as_str())),
            structure.get("target_obj").and_then(|x| x.get("id_str").and_then(|x| x.as_str()))
        ) {
            (Some(source_id), Some(target_id)) => Ok((source_id.to_string(), target_id.to_string())),
            (None, Some(target_id)) => Err("No id_str string at .source.id_str".to_string()),
            (Some(target_id), None) => Err("No id_str string at .target_object.id_str".to_string()),
            (None, None) => Err("No id_str at source or target_object".to_string())
        }
    }
                                                                            // maybe type error
                                                                            // better? string is ok
                                                                            // for now..
    pub fn from_json(structure: serde_json::Map<String, serde_json::Value>) -> Result<Event, String> {
        match structure.get("event").and_then(|x| x.as_str()).map(|x| x.to_owned()) {
            Some(event) => {
                let event_ref: &str = &event;
                match event_ref {
                    "follow" =>
                        Event::get_source_id(structure)
                            .map(|id_str|
                                Event::Followed {
                                    user_id: id_str
                                }
                            ),
                    "unfollow" =>
                        Event::get_source_id(structure)
                            .map(|id_str|
                                Event::Unfollowed {
                                    user_id: id_str
                                }
                            ),
                    "favorite" =>
                        Event::get_source_target_ids(structure)
                            .map(|(source_id, target_id)|
                                Event::Fav {
                                    user_id: source_id,
                                    twete_id: target_id
                                }
                            ),
                    "unfavorite" =>
                        Event::get_source_target_ids(structure)
                            .map(|(source_id, target_id)|
                                Event::Unfav {
                                    user_id: source_id,
                                    twete_id: target_id
                                }
                            ),
                    "favorited_retweet" =>
                        Event::get_source_target_ids(structure)
                            .map(|(source_id, target_id)|
                                Event::Fav_RT {
                                    user_id: source_id,
                                    twete_id: target_id
                                }
                            ),
                    "retweeted_retweet" =>
                        Event::get_source_target_ids(structure)
                            .map(|(source_id, target_id)|
                                Event::RT_RT {
                                    user_id: source_id,
                                    twete_id: target_id
                                }
                            ),
                    "quoted_tweet" =>
                        Event::get_source_target_ids(structure)
                            .map(|(source_id, target_id)|
                                Event::Quoted {
                                    user_id: source_id,
                                    twete_id: target_id
                                }
                            ),
        //                "list_member_added" =>
        //                what about removed?
        //                "blocked" => Blocked { },
        //                "unblocked" => Unblocked { },
                    e => { println!("unrecognized event: {}", e); Err(e.to_string()) }
                }
            },
            None => {
                Err("No event in event json...".to_string())
            }
        }
    }
}
