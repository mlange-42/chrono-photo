//! Lists files by pattern

use std::ffi::OsString;
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub struct FileLister {
    path: OsString,
    pattern: OsString,
}

impl FileLister {
    pub fn new(pattern: &str) -> Result<Self, std::io::Error> {
        let pp = Path::new(pattern);

        let path: OsString = pp
            .parent()
            .expect("Path required")
            .as_os_str()
            .to_os_string();
        let pattern: OsString = pp.file_name()
            .expect("A search pattern requires at least a file pattern, optionally preceeded by a path.")
            .to_os_string();

        Ok(FileLister { path, pattern })
    }

    pub fn list_files(&self) -> () {
        let paths = fs::read_dir(path);
        println!("{:?}", paths);
    }
}

#[cfg(test)]
mod test {
    use crate::flist::FileLister;

    #[test]
    fn parse_pattern() {
        let pattern = "test/path/*.jpg";
        let lister = FileLister::new(&pattern);

        let pattern = "*.jpg";
        let lister = FileLister::new(&pattern);
    }
}
