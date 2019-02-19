use std::time::{SystemTime, Duration};
use std::thread::sleep;
use std::collections::HashMap;
use walkdir::{DirEntry, WalkDir};
use std::env::current_dir;
use std::path::PathBuf;
use std::path::Path;
use ansi_term::Colour;

enum FileStatus {
    Created,
    Modified,
    Erased
}

struct FileWatcher {
    path: PathBuf,
    duration: Duration,
    running: bool,
    paths: HashMap<PathBuf, SystemTime>
}

impl FileWatcher {
    fn new(path: &PathBuf, duration: u64) -> FileWatcher {
        FileWatcher {
            path: Path::new(path).to_path_buf(),
            duration: Duration::new(duration, 0),
            running: true,
            paths: HashMap::new()
        }
    }

    fn start(&mut self, action: fn(&PathBuf, FileStatus)) {
        while self.running {
            sleep(self.duration);
            
            let keys: Vec<PathBuf> = self.paths.keys().cloned().collect();  
            for key in keys {
                if !Path::new(&key).exists() {
                    action(&key, FileStatus::Erased);
                    self.paths.remove(&key);
                }
            }

            for entry in WalkDir::new(&self.path) {
                let entry_dir: &DirEntry = &entry.unwrap();
                let path_buf = &entry_dir.path().to_path_buf();
        
                if let Ok(last_write_time) = entry_dir.metadata().unwrap().modified() {
                    if !self.paths.contains_key(entry_dir.path()) {
                        self.paths.insert(entry_dir.path().to_path_buf(), last_write_time);
                        action(path_buf, FileStatus::Created);
                    } else  {
                        let current_write_time = self.paths.get(path_buf).unwrap();
                        if !current_write_time.eq(&last_write_time) {
                            *self.paths.get_mut(path_buf).unwrap() = last_write_time;
                            action(path_buf, FileStatus::Modified);
                        }
                    }
                }
            }
        }
    }
    
    fn get_path(&mut self) -> std::io::Result<PathBuf> {
        let path = current_dir()?;
        for entry in WalkDir::new(&path) {
            let entry_dir: &DirEntry = &entry?;
            if let Ok(last_write_time) = entry_dir.metadata()?.modified() {
                self.paths.insert(entry_dir.path().to_path_buf(), last_write_time);
            }
        }
        Ok(path)
    }
}

fn process(path: &PathBuf, status: FileStatus) {
    let path_str = Path::new(&path).to_str().unwrap();
    match status {
        FileStatus::Erased => 
            println!("Erased: {}", Colour::Red.paint(path_str)),
        FileStatus::Created =>
            println!("Created: {}", Colour::Green.paint(path_str)),
        FileStatus::Modified => 
            println!("Modified: {}", Colour::Yellow.paint(path_str))
    }
}

fn main() {
    let enabled = ansi_term::enable_ansi_support(); // for Windows 10

    let path = PathBuf::from(current_dir().unwrap());
    let mut file_watcher = FileWatcher::new(&path, 1);

    let func_ptr: fn(&PathBuf, FileStatus) = process;
    file_watcher.get_path().unwrap();
    file_watcher.start(func_ptr); // pass func_ptr as fn
}