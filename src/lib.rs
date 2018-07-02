#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

extern crate reqwest;

use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct AceResponse<T> {
    pub response: T,
    error: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AceResult<T> {
    pub result: T,
    error: Option<String>,
}

#[derive(Debug)]
pub struct Engine {
    pub engine_url: reqwest::Url,
    pub streams: HashMap<String, Stream>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Player {
    pub protocol: Option<String>,
    pub icon: Option<String>,
    #[serde(rename = "type")]
    pub type_name: Option<String>,
    pub id: Option<String>,
    pub name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Stream {
    command_url: Option<String>,
    is_live: Option<usize>,
    playback_session_id: Option<String>,
    playback_url: Option<String>,
    stat_url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Stat {
    downloaded: Option<usize>,
    pub peers: Option<usize>,
    playback_session_id: Option<String>,
    progress: Option<usize>,
    pub speed_down: Option<usize>,
    pub speed_up: Option<usize>,
    pub status: Option<String>,
    time: Option<usize>,
    total_progress: Option<usize>,

    uploaded: Option<usize>,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new("http://127.0.0.1:6878")
    }
}

impl Engine {
    pub fn new(http_link: &str) -> Self {
        Engine {
            engine_url: reqwest::Url::parse(http_link).unwrap(),
            streams: HashMap::new(),
        }
    }

    pub fn is_up(&self) -> bool {
        reqwest::get(self.engine_url.clone()).is_ok()
    }

    pub fn version(&self) -> Result<serde_json::Value, reqwest::Error> {
        let mut map = HashMap::new();
        map.insert("method".to_owned(), "get_version".to_owned());
        map.insert("format".to_owned(), "json".to_owned());
        let url = self.build_url("webui/api/service", &map);
        let mut resp = reqwest::get(url).unwrap();
        resp.json()
    }

    pub fn build_url(&self, path: &str, queries: &HashMap<String, String>) -> reqwest::Url {
        let mut url = self.engine_url.join(path).unwrap();
        {
            let mut query_pairs = url.query_pairs_mut();
            for (ref k, ref v) in queries.iter() {
                query_pairs.append_pair(k.as_str(), v.as_str());
            }
        }
        url
    }

    pub fn get_stream(&self, id: &str) -> Stream {
        let mut map = HashMap::new();
        map.insert("id".to_owned(), id.to_owned());
        map.insert("format".to_owned(), "json".to_owned());
        let url = self.build_url("ace/getstream", &map);
        let mut resp = reqwest::get(url).unwrap();
        resp.json::<AceResponse<Stream>>().unwrap().response
    }

    pub fn add_stream(&mut self, id: &str) {
        let stream = self.get_stream(id);
        self.streams.insert(id.to_owned(), stream);
    }

    pub fn get_stream_link(&self, id: &str) -> String {
        let stream = &self.streams[id];
        stream.playback_url.as_ref().unwrap().to_owned()
    }

    pub fn is_stream_live(&self, id: &str) -> bool {
        let stream = &self.streams[id];
        stream.is_live.unwrap() > 0
    }

    pub fn get_stream_stat(&self, id: &str) -> Stat {
        let stream = &self.streams[id];
        let url = stream.stat_url.as_ref().unwrap();
        let mut resp = reqwest::get(url).unwrap();
        resp.json::<AceResponse<Stat>>().unwrap().response
    }

    pub fn stop_stream(&self, id: &str) -> String {
        let stream = &self.streams[id];
        let mut url = reqwest::Url::parse(stream.command_url.as_ref().unwrap()).unwrap();
        url.query_pairs_mut().append_pair("method", "stop");
        let mut resp = reqwest::get(url).unwrap();
        resp.json::<AceResponse<String>>().unwrap().response
    }

    pub fn get_players(&self) -> Vec<Player> {
        let mut map = HashMap::new();
        map.insert("method".to_owned(), "get_available_players".to_owned());
        map.insert(
            "content_id".to_owned(),
            "94c2fd8fb9bc8f2fc71a2cbe9d4b866f227a0209".to_owned(),
        );
        map.insert("format".to_owned(), "json".to_owned());
        let url = self.build_url("server/api", &map);
        let mut resp = reqwest::get(url).unwrap();
        resp.json::<AceResult<HashMap<String, Vec<Player>>>>()
            .unwrap()
            .result
            .remove("players")
            .unwrap()
    }

    pub fn play_on_player(&self, id: &str, player_id: &str) -> AceResult<String> {
        let mut map = HashMap::new();
        map.insert("method".to_owned(), "open_in_player".to_owned());
        map.insert("content_id".to_owned(), id.to_owned());
        map.insert("player_id".to_owned(), player_id.to_owned());
        map.insert("format".to_owned(), "json".to_owned());
        let url = self.build_url("server/api", &map);
        let mut resp = reqwest::get(url).unwrap();
        resp.json::<AceResult<String>>().unwrap()
    }
}
