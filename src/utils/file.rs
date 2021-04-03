use std::{
    convert::AsRef,
    fmt::Debug,
    fs::{self, File, OpenOptions},
    io::{self, BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
};

use fs_extra::{copy_items, dir::CopyOptions};
use itertools::Itertools;

use crate::utils::anyhow_err::ErrConversion;
use crate::utils::error::ErrorExt;

/// Extensions for common Files operations
pub trait PathExt where Self: AsRef<Path> + Debug {
    /// Replaces line in a file by `replace_by`. If `to_match` matches, replaces where it matches else appends or prepends to file.
    /// * `to_match` - line of the file to search for replacement
    /// * `replace_by` - if a line matching `to_match` is found ; replaces it with this
    /// * `strict` - If `strict` is true, `to_match` must contain the whole line else just a portion
    /// * `append` - If no line matching `to_match` is found, then either append or prepend
    fn merge(&self, to_match: &str, replace_by: &str, strict: bool, append: bool) {
        let mut has_line_matching = false;
        let mut file_modified = self.lines().iter_mut()
            .map(|it| if Self::line_matches_predicate(to_match, strict, it) {
                has_line_matching = true;
                replace_by.to_string()
            } else {
                (*it).to_string()
            })
            .filter(|it| !it.is_empty())
            .collect_vec();
        if !has_line_matching {
            if append {
                file_modified.append(vec![replace_by.to_string()].as_mut());
                self.write(file_modified);
            } else {
                let mut prepended_file = vec![replace_by.to_string()];
                prepended_file.append(file_modified.as_mut());
                self.write(prepended_file);
            }
        } else {
            self.write(file_modified);
        }
    }

    fn line_matches_predicate(to_match: &str, strict: bool, it: &str) -> bool {
        let to_match_is_not_empty = !to_match.is_empty();
        let strictly_matches = strict && it.eq(to_match);
        let just_contains_to_match = !strict && it.contains(to_match);
        to_match_is_not_empty && (strictly_matches || just_contains_to_match)
    }

    /// Append to the file the modified lines
    fn prepend_line(&self, matched: bool, line: &str) {
        let file_modified = self.lines();
        let mut file = self.open_replace();
        if !matched { self.write_to_file(&mut file, line) };
        file_modified.iter()
            .for_each(|line| { self.write_to_file(&mut file, line); });
    }

    /// Append to the file the modified lines
    fn append_line(&self, line: &str) {
        let file_modified = self.lines();
        let mut file = self.open_replace();
        file_modified.iter().for_each(|l| self.write_to_file(&mut file, l));
        self.write_to_file(&mut file, line)
    }

    /// Reads a file and join its lines into a string
    fn read_to_string(&self) -> String {
        self.lines().into_iter().collect()
    }

    /// Reads a file lines
    fn read_lines(&self) -> Vec<String> { self.lines() }

    /// Reads a file and join its lines into a string preserving new lines
    fn read_pretty(&self) -> String {
        self.lines().iter_mut().map(|it| format!("{}\n", it)).collect()
    }

    /// Reads a file and join its lines into a string
    fn read_binary(&self) -> Vec<u8> {
        self.open_read().map(|mut file| {
            let mut buf = vec![];
            file.read_to_end(&mut buf)
                .fail(format!("Failed reading file {:?} completely", self));
            buf
        }).unwrap_or_default()
    }

    /// Writes to file with new line at the end of each line
    fn write_to_file(&self, file: &mut File, value: &str) {
        file.write_fmt(format_args!("{}\n", value))
            .fail(format!("Failed writing {} to {:?}", value, self));
    }

    /// Writes to file with new line at the end of each line
    fn write_to(&self, value: &str) {
        self.write_to_file(&mut self.open_replace(), value)
    }

    /// Writes to file with new line at the end of each line
    fn write(&self, lines: Vec<String>) {
        let mut file = self.open_replace();
        lines.iter()
            .for_each(|line| { self.write_to_file(&mut file, line) });
    }

    /// Opens a file with read only access
    fn open_read(&self) -> Option<File> {
        OpenOptions::new().read(true).open(self).ok()
    }

    /// Opens a file with write access
    fn open_write(&self) -> File {
        OpenOptions::new().write(true).open(self)
            .fail(format!("Failed opening file {:?} with write access", self))
    }

    /// Opens a file with write access and erase previous content from it
    fn open_replace(&self) -> File {
        OpenOptions::new().write(true).truncate(true).open(self)
            .fail(format!("Failed opening file {:?} with write and truncate access", self))
    }

    /// Opens a file with write access and creates it
    fn open_new(&self) -> anyhow::Result<File> {
        OpenOptions::new().read(true).write(true).create_new(true).open(self).wrap()
    }

    /// Deletes the file at given path
    fn rename(&self, to: &Self) -> anyhow::Result<()> {
        fs::rename(self, to).wrap()
    }

    /// Deletes the file at given path
    fn delete(&self) -> io::Result<()> { fs::remove_file(self) }

    /// Deletes a directory and all its content at given path
    fn delete_dir(&self) -> anyhow::Result<()> { fs::remove_dir_all(self).wrap() }

    /// Creates an empty file at given path
    fn create(&self) -> anyhow::Result<File> { File::create(self).wrap() }

    /// Get file name as str
    fn file_name_str(&self) -> Option<&str> {
        self.as_ref().file_name().and_then(|f| f.to_str())
    }

    /// Creates an empty dir at given path
    fn create_dir(&self) -> anyhow::Result<()> { fs::create_dir(self).wrap() }

    /// Copies to given path by overwriting
    fn copy_to(&self, to: &Self) -> io::Result<u64> { fs::copy(self, to) }

    /// Copies to given path by overwriting else fails
    fn copy_to_or_fail(&self, to: &Self) {
        self.copy_to(to).fail(format!("Failed copying {:?} to {:?}", self, to));
    }

    /// Copies recursively all files under `self` to `to`
    fn copy_all(&self, to: &Self) {
        let copy_options = CopyOptions {
            overwrite: true,
            skip_exist: false,
            buffer_size: 64000, //64kb
            copy_inside: true,
            content_only: false,
            depth: 0,
        };
        copy_items(&[self], to, &copy_options)
            .fail(format!("Failed copying all files from {:?} to {:?}", self, to));
    }

    /// Reads each line of the file and returns them as a vec of string
    fn lines(&self) -> Vec<String> {
        self.open_read()
            .map(|file| BufReader::new(file).lines().filter_map(|it| it.ok()).collect())
            .unwrap_or_default()
    }

    /// Returns path as string slice
    fn path_str(&self) -> &str {
        self.as_ref().to_str().fail(format!("Failed converting {:?} path to str", self))
    }

    /// Creates all non existing directory in this path
    fn create_dir_all_or_fail(&self) {
        fs::create_dir_all(self)
            .fail(format!("Failed creating all directories under {}", self.path_str()))
    }

    /// Creates all non existing directory in this path
    fn create_dir_all(&self) -> anyhow::Result<()> {
        fs::create_dir_all(self).wrap()
    }

    fn children(&self) -> Vec<PathBuf> {
        self.as_ref().read_dir().ok()
            .map(|dir| dir.filter_map(|f| f.ok()).map(|it| it.path()).collect_vec())
            .unwrap_or_default()
    }

    fn find(&self, name: &str) -> Option<PathBuf> {
        self.children().into_iter()
            .find(|it| it.file_name_str() == Some(name))
    }
}

impl PathExt for PathBuf {}

impl PathExt for Path {}

#[cfg(test)]
mod file_ext_tests {
    use std::ops::Not;

    use crate::mocks::MockFs;

    use super::*;

    #[test]
    fn should_create_empty_file() {
        let path = MockFs::home().join("file-create.txt");
        path.create().unwrap();
        assert!(path.exists());
    }

    #[test]
    fn should_create_empty_dir() {
        let path = MockFs::home().join("folder-create");
        path.create_dir().unwrap();
        assert!(path.exists());
    }

    #[test]
    fn should_delete_file() {
        let path = MockFs::home().join("file-delete.txt");
        path.create().unwrap();
        assert!(path.exists());
        path.delete().unwrap();
        assert!(path.exists().not());
    }

    #[test]
    fn should_delete_dir() {
        let path = MockFs::home().join("folder-delete");
        path.create_dir().unwrap();
        assert!(path.exists());
        path.delete_dir().unwrap();
        assert!(path.exists().not());
    }

    #[test]
    fn should_write_to_and_read_from_file() {
        let path = MockFs::home().join("writable_file.txt");
        path.create().unwrap();
        path.write(vec!["Hello", "World", "!"].iter().map(|it| it.to_string()).collect());
        assert_eq!(path.read_to_string(), "HelloWorld!".to_string())
    }

    #[test]
    fn should_merge() {
        let path = MockFs::home().join("merge-props.properties");
        path.create().unwrap();
        let content: Vec<String> = vec!["a=b", "b=c", "x.y=z"].iter().map(|it| it.to_string()).collect();
        path.write(content);

        path.merge("b=c", "i=j", true, true);
        assert_eq!(path.read_to_string(), "a=bi=jx.y=z".to_string());

        path.merge("", "k=l", true, true);
        assert_eq!(path.read_to_string(), "a=bi=jx.y=zk=l".to_string());

        path.merge("", "m=n", false, true);
        assert_eq!(path.read_to_string(), "a=bi=jx.y=zk=lm=n".to_string());

        path.merge("i=", "k=l", false, true);
        assert_eq!(path.read_to_string(), "a=bk=lx.y=zk=lm=n".to_string());

        path.merge("", "1=2", false, true);
        assert_eq!(path.read_to_string(), "a=bk=lx.y=zk=lm=n1=2".to_string());

        path.merge("", "3=4", false, false);
        assert_eq!(path.read_to_string(), "3=4a=bk=lx.y=zk=lm=n1=2".to_string());
    }

    #[test]
    fn should_find_children() {
        let home = MockFs::home();
        let (dir, file) = (home.join("a"), home.join("a.txt"));
        dir.create_dir().unwrap();
        file.create().unwrap();
        assert!(dir.exists());
        assert!(file.exists());

        assert!(home.children().contains(&dir));
        assert!(home.children().contains(&file));
    }

    #[test]
    fn should_find_child() {
        let home = MockFs::home();
        let (dir, file) = (home.join("find"), home.join("find.txt"));
        dir.create_dir().unwrap();
        file.create().unwrap();
        assert!(dir.exists());
        assert!(file.exists());

        assert_eq!(home.find("find"), Some(dir));
        assert_eq!(home.find("find.txt"), Some(file));

        assert!(home.find("unknown").is_none());
        assert!(home.find("unknown.txt").is_none());
    }
}