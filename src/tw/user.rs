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
    pub fn from_json(json: serde_json::Value) -> Result<User, String> {
        if let serde_json::Value::Object(json_map) = json {
            if let (
                Some(id_str),
                Some(name),
                Some(screen_name)
            ) = (
                json_map.get("id_str").and_then(|x| x.as_str()),
                json_map.get("name").and_then(|x| x.as_str()),
                json_map.get("screen_name").and_then(|x| x.as_str())
            ) {
                Ok(User {
                    id: id_str.to_owned(),
                    name: name.to_owned(),
                    handle: screen_name.to_owned()
                })
            } else {
                Err("user json missing one of id_str, name, screen_name".to_owned())
            }
        } else {
            Err(format!("Invalid json: {:?}", json))
        }
    }
}

