extern crate serde_json;

use tw::user::User;

#[derive(Debug, Serialize, Deserialize)]
pub struct Tweet {
    pub id: String,
    pub author_id: String,
    pub text: String,
    pub created_at: String,     // lol
    #[serde(skip_serializing_if="Option::is_none")]
    #[serde(default = "Option::default")]
    pub quoted_tweet_id: Option<String>,
    #[serde(skip_serializing_if="Option::is_none")]
    #[serde(default = "Option::default")]
    pub rt_tweet: Option<String>,
    #[serde(skip_serializing_if="Option::is_none")]
    #[serde(default = "Option::default")]
    pub reply_to_tweet: Option<String>,
    #[serde(skip)]
    pub internal_id: u64
}

impl Tweet {
    pub fn get_mentions(&self) -> Vec<&str> {
        self.text.split(&[
            ',', '.', '/', ';', '\'',
            '[', ']', '\\', '~', '!',
            '@', '#', '$', '%', '^',
            '&', '*', '(', ')', '-',
            '=', '{', '}', '|', ':',
            '"', '<', '>', '?', '`',
            ' ' // forgot this initially. awkward.
        ][..])
            .filter(|x| x.starts_with("@") && x.len() > 1)
            .collect()
    }

    pub fn from_api_json(json: serde_json::Value) -> Option<(Tweet, User)> {
        Tweet::from_json(json.clone()).and_then(|tw| {
            json.get("user").and_then(|user_json|
                User::from_json(user_json.to_owned()).map(|u| (tw, u))
            )
        })
    }
    pub fn from_json(json: serde_json::Value) -> Option<Tweet> {
        if let serde_json::Value::Object(json_map) = json {
            let text = ::tw::full_twete_text(&json_map);
            let rt_twete = json_map.get("retweeted_status")
                .and_then(|x| x.get("id_str"))
                .and_then(|x| x.as_str())
                .map(|x| x.to_owned());
            let reply_to_tweet = json_map.get("in_reply_to_status_id_str")
                .and_then(|x| x.as_str())
                .map(|x| x.to_owned());
            if json_map.contains_key("id_str") &&
               json_map.contains_key("user") &&
               json_map.contains_key("created_at") {
                if let (
                    Some(id_str),
                    Some(author_id),
                    Some(created_at)
                ) = (
                    json_map["id_str"].as_str(),
                    json_map["user"]["id_str"].as_str(),
                    json_map["created_at"].as_str()
                ) {
                    return Some(Tweet {
                        id: id_str.to_owned(),
                        author_id: author_id.to_owned(),
                        text: text,
                        created_at: created_at.to_owned(),
                        quoted_tweet_id: json_map.get("quoted_status_id_str")
                            .and_then(|x| x.as_str())
                            .map(|x| x.to_owned()),
                        rt_tweet: rt_twete,
                        reply_to_tweet: reply_to_tweet,
                        internal_id: 0
                    })
                }
            }
        }
        None
    }
}
