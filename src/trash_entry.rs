use std::{
    fs::File,
    io::{self, BufRead},
    path::PathBuf,
};

use chrono::{DateTime, Local, NaiveDateTime, TimeZone};

use crate::{
    files::{get_trash_dirs, parse_line},
    ui::Message,
};

pub struct TrashEntry {
    pub name: String,
    pub info_path: PathBuf,
    pub content_path: PathBuf,
    pub restore_location: PathBuf,
    pub date: DateTime<Local>,
}

impl TrashEntry {
    pub fn get_content_path(&self) -> PathBuf {
        let (_, files_dir, _) = get_trash_dirs();
        files_dir.join(&self.info_path.file_name().unwrap())
    }

    pub fn from_trash_info(info_path: &PathBuf) -> Result<Self, Message> {
        let (_, files_dir, _) = get_trash_dirs();

        match File::open(info_path) {
            Err(e) => panic!("Error opening trash info file: {}", e),
            Ok(file) => {
                let mut lines = io::BufReader::new(file).lines();
                lines.next(); // Skip the first line (header)

                let restore_location = parse_line(&mut lines, info_path)?.replace("Path=", "");
                let date_str = parse_line(&mut lines, info_path)?.replace("DeletionDate=", "");

                let date =
                    match NaiveDateTime::parse_from_str(date_str.as_str(), "%Y-%m-%dT%H:%M:%S") {
                        Ok(dt) => match Local.from_local_datetime(&dt) {
                            chrono::offset::LocalResult::Single(date) => date,
                            _ => {
                                return Err(Message::error(
                                    "Invalid date format in trash info file".to_string(),
                                ));
                            }
                        },
                        Err(_) => {
                            return Err(Message::error(
                                "Invalid date format in trash info file".to_string(),
                            ));
                        }
                    };

                let name = match PathBuf::from(&restore_location).file_name() {
                    Some(name) => urlencoding::decode(name.to_str().unwrap())
                        .unwrap()
                        .to_string(),
                    None => {
                        return Err(Message::error(
                            "Invalid file name in trash info file".to_string(),
                        ));
                    }
                };

                let content_path = match info_path.file_stem() {
                    Some(name) => match name.to_str() {
                        Some(name) => name,
                        None => {
                            return Err(Message::error(
                                "Invalid file name in trash info file".to_string(),
                            ));
                        }
                    },
                    None => {
                        return Err(Message::error(
                            "Invalid file name in trash info file".to_string(),
                        ));
                    }
                };

                Ok(TrashEntry {
                    name,
                    restore_location: PathBuf::from(&restore_location),
                    info_path: info_path.to_owned(),
                    content_path: files_dir.join(content_path),
                    date,
                })
            }
        }
    }
}

impl Clone for TrashEntry {
    fn clone(&self) -> Self {
        TrashEntry {
            name: self.name.clone(),
            info_path: self.info_path.clone(),
            content_path: self.content_path.clone(),
            restore_location: self.restore_location.clone(),
            date: self.date.clone(),
        }
    }
}
