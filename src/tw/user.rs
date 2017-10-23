extern crate serde_json;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: String,
    pub name: String,
    pub handle: String
}

impl Default for User {
    fn default() -> User {
        User {
            id: "".to_owned(),
            name: "_default_".to_owned(),
            handle: "_default_".to_owned()
        }
    }
}

impl User {
    pub fn from_json(json: serde_json::Value) -> Option<User> {
        if let serde_json::Value::Object(json_map) = json {
            if json_map.contains_key("id_str") &&
               json_map.contains_key("name") &&
               json_map.contains_key("screen_name") {
                if let (
                    Some(id_str),
                    Some(name),
                    Some(screen_name)
                ) = (
                    json_map["id_str"].as_str(),
                    json_map["name"].as_str(),
                    json_map["screen_name"].as_str()
                ) {
                    return Some(User {
                        id: id_str.to_owned(),
                        name: name.to_owned(),
                        handle: screen_name.to_owned()
                    })
                }
            }
        }
        None
    }
}

