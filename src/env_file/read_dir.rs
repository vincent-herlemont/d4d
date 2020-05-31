use std::fs;
use std::path::PathBuf;

use crate::env_file::Env;
use crate::env_file::Result;

pub fn read_dir(dir: &PathBuf) -> Vec<Result<Env>> {
    let mut envs = vec![];
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                if !entry.path().is_file() {
                    continue;
                }

                // Ignore files that not start by "."
                if let Ok(file_name) = entry.file_name().into_string() {
                    if !file_name.starts_with(".") {
                        continue;
                    }
                } else {
                    continue;
                }

                let env = Env::from_file_reader(entry.path());
                envs.append(&mut vec![env]);
            }
        }
    }
    envs
}
