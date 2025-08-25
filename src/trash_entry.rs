use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

use chrono::{DateTime, Local, NaiveDateTime, TimeZone};

use crate::{
    io::{get_trash_dirs, parse_line},
    ui::Message,
};

pub struct TrashEntry {
    pub display_name: String,
    pub info_path: PathBuf,
    pub content_path: PathBuf,
    pub restore_location: PathBuf,
    pub date: DateTime<Local>,
}

// Example:
// trash file info path: <trash files dir>/files/video_2.avi
// trash file contents path: <trash files dir>/files/video_2.avi
//
// Info file:
// [Trash Info]
// Path=/tmp/%D1%81%D0%B5%D0%B72/video.avi
// DeletionDate=2025-07-02T13:40:56

impl TrashEntry {
    pub fn from_trash_info(path_to_info_file: &PathBuf) -> Result<Self, Message> {
        let (_, files_dir, _) = get_trash_dirs();

        let file = File::open(path_to_info_file)
            .map_err(|e| Message::error(format!("Error opening trash info file: {}", e)))?;
        let mut lines = BufReader::new(file).lines();
        lines.next(); // Skip header

        let restore_location = PathBuf::from(
            parse_line(&mut lines, path_to_info_file)?
                .strip_prefix("Path=")
                .ok_or_else(|| {
                    Message::error("Missing Path= prefix in restore location".to_string())
                })?
                .to_string(),
        );

        let date = extract_date(parse_line(&mut lines, path_to_info_file)?.as_str())?;

        let file_name = restore_location
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| Message::error("Invalid or missing file name".to_string()))?;

        let display_name = urlencoding::decode(file_name)
            .map_err(|e| Message::error(format!("Failed to decode filename: {}", e)))?
            .to_string();

        Ok(TrashEntry {
            display_name,
            info_path: path_to_info_file.clone(),
            content_path: files_dir.join(
                path_to_info_file
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .ok_or_else(|| Message::error("Invalid info file name".to_string()))?,
            ),
            restore_location,
            date,
        })
    }
}

fn extract_date(date_str: &str) -> Result<DateTime<Local>, Message> {
    let date_str = date_str
        .strip_prefix("DeletionDate=")
        .ok_or_else(|| Message::error("Missing DeletionDate= prefix".to_string()))?
        .to_string();

    let naive_date = NaiveDateTime::parse_from_str(&date_str, "%Y-%m-%dT%H:%M:%S")
        .map_err(|_| Message::error("Invalid date format in trash info file".to_string()))?;

    Local
        .from_local_datetime(&naive_date)
        .single()
        .ok_or_else(|| Message::error("Ambiguous or invalid local datetime".to_string()))
}

impl Clone for TrashEntry {
    fn clone(&self) -> Self {
        Self {
            display_name: self.display_name.clone(),
            info_path: self.info_path.clone(),
            content_path: self.content_path.clone(),
            restore_location: self.restore_location.clone(),
            date: self.date.clone(),
        }
    }
}
