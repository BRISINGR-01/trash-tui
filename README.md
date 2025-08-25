# trash-tui

## How Trash Works on Linux

The Trash system on Linux follows the FreeDesktop.org Trash Specification, used by desktop environments like GNOME, KDE, and others.

Files moved to the trash are stored in two parts:

- The actual content (data)
- The metadata (info)

### Trash Directory Structure

Each partition (mount point) has its own trash directory, typically located at:
`/<mount-point>/.Trash-<uid>/`

Or if not accessible:
`~/.local/share/Trash/`

Inside the trash directory, there are three subdirectories:

```
Trash/
├── files/ <- Contains the actual deleted files
├── info/ <- Contains .trashinfo metadata files
└── expunged/ <- (Optional, used by some DEs to store permanently deleted items)
```

### .trashinfo Metadata Format

Each deleted file gets a corresponding .trashinfo file in the `info/` directory. This file holds two pieces of information:

Example
For a file originally at `/tmp/video.avi` moved to trash on July 2, 2025, 13:40:56:

Content file path:  
`~/.local/share/Trash/files/video_2.avi`

Info file path:  
`~/.local/share/Trash/info/video_2.avi.trashinfo`

Info file contents:

```trashinfo
[Trash Info]
Path=/tmp/video.avi
DeletionDate=2025-07-02T13:40:56
```

- `Path` - The original location of the file (URL-encoded).
- `DeletionDate` - The date and time the file was deleted (in ISO 8601 format).

## How restoring files works?

To restore a file from trash:

1. Read the .trashinfo file from `info/`.
2. Parse the Path value and decode it.
3. Move the corresponding file from `files/` back to the original path.

## Build

`cargo build --target x86_64-unknown-linux-gnu`
