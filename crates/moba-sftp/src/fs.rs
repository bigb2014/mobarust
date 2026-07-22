//! SFTP remote filesystem model.
//!
//! Represents remote files and directories for the SFTP browser.

use serde::{Deserialize, Serialize};

/// A remote file entry: file or directory.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteEntry {
    /// Entry name (filename or directory name).
    pub name: String,
    /// Full path on the remote server.
    pub path: String,
    /// Whether this entry is a directory.
    pub is_dir: bool,
    /// File size in bytes (0 for directories).
    pub size: u64,
    /// Modification time as Unix timestamp (0 if unknown).
    pub modified: u64,
    /// File permissions as octal string (e.g. "755").
    pub permissions: String,
}

impl RemoteEntry {
    /// Creates a new remote entry.
    #[must_use]
    pub fn new(
        name: &str,
        path: &str,
        is_dir: bool,
        size: u64,
        modified: u64,
        permissions: &str,
    ) -> Self {
        Self {
            name: name.to_string(),
            path: path.to_string(),
            is_dir,
            size,
            modified,
            permissions: permissions.to_string(),
        }
    }

    /// Returns the file extension, if any.
    #[must_use]
    pub fn extension(&self) -> Option<&str> {
        if self.is_dir {
            return None;
        }
        self.name.rsplit_once('.').map(|(_, ext)| ext)
    }

    /// Returns a display label for the entry.
    #[must_use]
    pub fn display_label(&self) -> String {
        if self.is_dir {
            format!("{}/", self.name)
        } else {
            self.name.clone()
        }
    }

    /// Returns a human-readable file size.
    #[must_use]
    pub fn size_human(&self) -> String {
        if self.is_dir {
            return String::new();
        }
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;
        if self.size >= GB {
            format!("{:.1} GB", self.size as f64 / GB as f64)
        } else if self.size >= MB {
            format!("{:.1} MB", self.size as f64 / MB as f64)
        } else if self.size >= KB {
            format!("{:.1} KB", self.size as f64 / KB as f64)
        } else {
            format!("{} B", self.size)
        }
    }
}

/// A remote directory listing.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirListing {
    /// The directory path that was listed.
    pub path: String,
    /// Entries in the directory.
    pub entries: Vec<RemoteEntry>,
}

impl DirListing {
    /// Creates a new directory listing.
    #[must_use]
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
            entries: Vec::new(),
        }
    }

    /// Adds an entry to the listing.
    pub fn add(&mut self, entry: RemoteEntry) {
        self.entries.push(entry);
    }

    /// Returns only the directories.
    #[must_use]
    pub fn directories(&self) -> Vec<&RemoteEntry> {
        self.entries.iter().filter(|e| e.is_dir).collect()
    }

    /// Returns only the files.
    #[must_use]
    pub fn files(&self) -> Vec<&RemoteEntry> {
        self.entries.iter().filter(|e| !e.is_dir).collect()
    }

    /// Sorts entries: directories first, then alphabetically.
    pub fn sort(&mut self) {
        self.entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remote_entry_new_file() {
        let e = RemoteEntry::new("test.txt", "/home/user/test.txt", false, 1024, 0, "644");
        assert_eq!(e.name, "test.txt");
        assert!(!e.is_dir);
        assert_eq!(e.size, 1024);
    }

    #[test]
    fn remote_entry_new_dir() {
        let e = RemoteEntry::new("docs", "/home/user/docs", true, 0, 0, "755");
        assert!(e.is_dir);
        assert_eq!(e.size, 0);
    }

    #[test]
    fn extension_file() {
        let e = RemoteEntry::new("file.tar.gz", "/p", false, 100, 0, "644");
        assert_eq!(e.extension(), Some("gz"));
    }

    #[test]
    fn extension_dir_is_none() {
        let e = RemoteEntry::new("mydir", "/p", true, 0, 0, "755");
        assert_eq!(e.extension(), None);
    }

    #[test]
    fn extension_no_dot() {
        let e = RemoteEntry::new("Makefile", "/p", false, 100, 0, "644");
        assert_eq!(e.extension(), None);
    }

    #[test]
    fn display_label_dir_has_trailing_slash() {
        let e = RemoteEntry::new("docs", "/p", true, 0, 0, "755");
        assert_eq!(e.display_label(), "docs/");
    }

    #[test]
    fn display_label_file_no_slash() {
        let e = RemoteEntry::new("test.txt", "/p", false, 100, 0, "644");
        assert_eq!(e.display_label(), "test.txt");
    }

    #[test]
    fn size_human_bytes() {
        let e = RemoteEntry::new("f", "/p", false, 512, 0, "644");
        assert_eq!(e.size_human(), "512 B");
    }

    #[test]
    fn size_human_kilobytes() {
        let e = RemoteEntry::new("f", "/p", false, 2048, 0, "644");
        assert_eq!(e.size_human(), "2.0 KB");
    }

    #[test]
    fn size_human_megabytes() {
        let e = RemoteEntry::new("f", "/p", false, 1_048_576, 0, "644");
        assert_eq!(e.size_human(), "1.0 MB");
    }

    #[test]
    fn size_human_dir_is_empty() {
        let e = RemoteEntry::new("d", "/p", true, 0, 0, "755");
        assert_eq!(e.size_human(), "");
    }

    #[test]
    fn dir_listing_new_is_empty() {
        let l = DirListing::new("/home");
        assert_eq!(l.path, "/home");
        assert!(l.entries.is_empty());
    }

    #[test]
    fn dir_listing_add_and_split() {
        let mut l = DirListing::new("/home");
        l.add(RemoteEntry::new(
            "file.txt",
            "/home/file.txt",
            false,
            100,
            0,
            "644",
        ));
        l.add(RemoteEntry::new(
            "subdir",
            "/home/subdir",
            true,
            0,
            0,
            "755",
        ));
        assert_eq!(l.directories().len(), 1);
        assert_eq!(l.files().len(), 1);
    }

    #[test]
    fn dir_listing_sort_dirs_first() {
        let mut l = DirListing::new("/p");
        l.add(RemoteEntry::new("zfile", "/p/zfile", false, 10, 0, "644"));
        l.add(RemoteEntry::new("adir", "/p/adir", true, 0, 0, "755"));
        l.add(RemoteEntry::new("bfile", "/p/bfile", false, 10, 0, "644"));
        l.add(RemoteEntry::new("zdir", "/p/zdir", true, 0, 0, "755"));
        l.sort();
        assert_eq!(l.entries[0].name, "adir");
        assert_eq!(l.entries[1].name, "zdir");
        assert_eq!(l.entries[2].name, "bfile");
        assert_eq!(l.entries[3].name, "zfile");
    }

    #[test]
    fn dir_listing_serde_round_trip() {
        let mut l = DirListing::new("/home");
        l.add(RemoteEntry::new("f", "/home/f", false, 42, 0, "644"));
        let json = serde_json::to_string(&l).unwrap();
        let back: DirListing = serde_json::from_str(&json).unwrap();
        assert_eq!(l, back);
    }

    #[test]
    fn remote_entry_serde_round_trip() {
        let e = RemoteEntry::new("test", "/p/test", true, 0, 12345, "755");
        let json = serde_json::to_string(&e).unwrap();
        let back: RemoteEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(e, back);
    }
}
