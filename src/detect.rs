use std::collections::HashMap;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::{self, Path, PathBuf};

use anyhow::Result;
use serde::Deserialize;
use uuid::Uuid;

#[allow(unused)]
#[derive(Debug, Clone, Deserialize)]
struct UserCache {
    name: String,
    uuid: Uuid,
    #[serde(rename = "expiresOn")]
    expires_on: String,
}

#[derive(Default, Debug, Clone)]
pub struct App {
    dir: PathBuf,

    pub uuids: HashMap<Uuid, ()>,
}

impl App {
    pub fn new(path: &str) -> Self {
        Self {
            dir: PathBuf::from(path),
            uuids: HashMap::new(),
        }
    }

    fn load_usercache(&mut self, server_path: PathBuf) -> Result<()> {
        let usercache_path = server_path.join("usercache.json");

        if usercache_path.exists() {
            let server_properties_file = File::open(usercache_path)?;
            let server_properties_reader = BufReader::new(server_properties_file);
            let usercache: Vec<UserCache> = serde_json::from_reader(server_properties_reader)?;

            for user in usercache {
                self.uuids.insert(user.uuid, ());
            }
        }

        Ok(())
    }

    fn read_playerdata(&mut self, world_path: PathBuf) -> Result<()> {
        let players_path = world_path.join("playerdata");

        for entry in fs::read_dir(players_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file()
                && let Some(prefix) = path.file_prefix()
                && let Some(uuid_str) = prefix.to_str()
                && let Ok(uuid) = Uuid::parse_str(uuid_str)
            {
                self.uuids.insert(uuid, ());
            }
        }

        Ok(())
    }

    pub fn get_world_path(&mut self) -> Option<PathBuf> {
        let server_properties_path = self.dir.join("server.properties");

        // 如果存在 server.properties 文件，则说明这是一个服务器文件夹
        let world_path = if server_properties_path.exists() {
            // 尝试从 usercache.json 中提取 UUID
            let _ = self.load_usercache(self.dir.clone());

            let server_properties_file = File::open(server_properties_path).ok()?;

            let server_properties_reader = BufReader::new(server_properties_file);
            let server_properties: HashMap<String, String> =
                java_properties::read(server_properties_reader).ok()?;

            server_properties
                .get("level-name")
                .map(|level_name| self.dir.join(level_name))?
        } else {
            self.dir.clone()
        };

        if world_path.join("level.dat").exists() {
            // 尝试从 playerdata 中提取 UUID
            let _ = self.read_playerdata(world_path.clone());

            Some(world_path)
        } else {
            None
        }
    }
}

#[test]
fn detect() {
    let folder = r"C:\Users\27978\Downloads\新建文件夹\server\world";

    let mut app = App::new(folder);
    println!("{:?}", app.get_world_path());
    println!("{:?}", app.uuids);

    let folder = r"C:\Users\27978\Downloads\新建文件夹\server";

    let mut app = App::new(folder);
    println!("{:?}", app.get_world_path());
    println!("{:?}", app.uuids);

    let folder = r"C:\Users\27978\Downloads\新建文件夹";

    let mut app = App::new(folder);
    println!("{:?}", app.get_world_path());
    println!("{:?}", app.uuids);
}
