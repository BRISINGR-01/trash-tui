use std::{
    env,
    fs::{self, File},
    io::{BufReader, Error, Lines},
    path::PathBuf,
};

use crate::{trash_entry::TrashEntry, ui::Message};

pub fn get_trash_dirs() -> (PathBuf, PathBuf, PathBuf) {
    match env::var("HOME") {
        Ok(val) => {
            let trash_path: PathBuf = [&val, ".local", "share", "Trash"].iter().collect();
            check_dir(&trash_path);

            let files_path = trash_path.join("files");
            check_dir(&files_path);

            let info_path = trash_path.join("info");
            check_dir(&info_path);

            (trash_path, files_path, info_path)
        }
        Err(_) => panic!("Error getting home directory"),
    }
}

fn check_dir(dir: &PathBuf) {
    if !dir.exists() {
        if let Err(err) = fs::create_dir(&dir) {
            panic!("Error creating {}: {}", &dir.display(), err);
        }
    }

    if !dir.is_dir() {
        panic!("{} not a directory", dir.display());
    }
}

pub fn list_files_from_dir(dir: &PathBuf) -> Vec<Option<PathBuf>> {
    match dir.read_dir() {
        Ok(entries) => entries
            .map(|entry| match entry {
                Ok(entry) => Some(entry.path()),
                Err(_) => None,
            })
            .collect(),
        Err(e) => panic!("Error: {}", e),
    }
}

pub fn empty_bin() -> Result<(), Error> {
    let (_, files_dir, info_dir) = get_trash_dirs();

    fs::remove_dir_all(&files_dir)?;
    fs::create_dir(&files_dir)?;

    fs::remove_dir_all(&info_dir)?;
    fs::create_dir(&info_dir)?;

    Ok(())
}

pub fn restore_item(item: &TrashEntry) -> Result<(), Error> {
    if !item.content_path.exists() {
        return Err(Error::new(
            std::io::ErrorKind::NotFound,
            format!(
                "File not found: {}, name: {}, restore location: {}",
                item.content_path.display(),
                item.name,
                item.restore_location.display(),
            ),
        ));
    }

    fs::rename(&item.content_path, &item.restore_location)?;

    if item.info_path.exists() {
        fs::remove_file(&item.info_path)?;
    }

    Ok(())
}

pub fn delete_item(item: &TrashEntry) -> Result<(), Error> {
    if item.info_path.exists() {
        fs::remove_file(&item.info_path)?;
    }

    let p = item.get_content_path();
    if p.exists() {
        fs::remove_file(&p)?;
    }

    Ok(())
}

pub fn parse_line(lines: &mut Lines<BufReader<File>>, path: &PathBuf) -> Result<String, Message> {
    match lines.next() {
        Some(Ok(line)) => Ok(line),
        Some(Err(e)) => Err(Message::error(format!(
            "Error reading trash info file: {} - {}",
            path.to_string_lossy(),
            e
        ))),
        None => Err(Message::error(format!(
            "No lines found in trash info file: {}",
            path.to_string_lossy()
        ))),
    }
}

#[cfg(test)]
mod tests {
    use chrono::Local;

    use super::*;

    #[test]
    fn can_find_trash_dirs() {
        let result = get_trash_dirs();
        assert_eq!(
            result,
            (
                PathBuf::from(env::var("HOME").unwrap())
                    .join(".local")
                    .join("share")
                    .join("Trash"),
                PathBuf::from(env::var("HOME").unwrap())
                    .join(".local")
                    .join("share")
                    .join("Trash")
                    .join("files"),
                PathBuf::from(env::var("HOME").unwrap())
                    .join(".local")
                    .join("share")
                    .join("Trash")
                    .join("info")
            )
        );
    }

    #[test]
    fn can_handle_missing_home_dir() {
        let home_backup = env::var("HOME").ok();
        unsafe {
            env::remove_var("HOME");
        }

        let result = std::panic::catch_unwind(|| get_trash_dirs());
        assert!(result.is_err());

        unsafe {
            if let Some(home) = home_backup {
                env::set_var("HOME", home);
            } else {
                panic!("Cannot restore HOME environment variable");
            }
        }
    }

    #[test]
    fn can_restore_item() {
        let (_, files_dir, info_dir) = get_trash_dirs();
        let test_file = files_dir.join("test_restore.txt");
        let restore_location = PathBuf::from("/tmp").join("test_restore.txt");
        let info_location = info_dir.join("test_restore.txt.trashinfo");

        fs::write(&test_file, "Test content").unwrap();
        fs::write(
            &info_location,
            format!(
                "[Trash Info]\nPath={}\nDeletionDate=2023-10-01T12:00:00",
                restore_location.display(),
            ),
        )
        .unwrap();

        let entry = TrashEntry {
            name: "test_restore.txt".to_string(),
            restore_location: restore_location.clone(),
            info_path: info_location.clone(),
            content_path: test_file.clone(),
            date: Local::now(),
        };

        restore_item(&entry).unwrap();
        assert!(restore_location.exists());

        fs::remove_file(restore_location.clone()).unwrap(); // Clean up after test
    }
}
