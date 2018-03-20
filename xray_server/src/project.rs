use std::cell::{Ref, RefCell};
use std::ffi::{OsStr, OsString};
use std::fs::FileType;
use std::rc::Rc;
use std::sync::mpsc;
use std::path::{Path, PathBuf};
use ignore::Walk;

#[derive(Clone)]
pub struct ProjectHandle(Rc<RefCell<Project>>);

pub struct Project {
    paths: Vec<PathBuf>,
    entries: Vec<Entry>
}

#[derive(Debug)]
enum Entry {
    File {
        name: OsString
    },
    Dir {
        name: OsString,
        children: Vec<Entry>
    }
}

impl ProjectHandle {
    pub fn new(paths: Vec<PathBuf>) -> ProjectHandle {
        ProjectHandle(Rc::new(RefCell::new(Project::new(paths))))
    }

    pub fn rescan(&self) {
        let mut inner = self.0.borrow_mut();
        let inner = &mut *inner;
        inner.entries.truncate(0);

        let mut stack: Vec<Entry> = Vec::new();
        for path in &inner.paths {
            stack.truncate(0);
            for walk_entry in Walk::new(path) {
                if let Ok(walk_entry) = walk_entry {
                    {
                        let mut entry = None;
                        while walk_entry.depth() < stack.len() {
                            let mut parent_entry = stack.pop().unwrap();
                            entry.map(|entry| parent_entry.push_child(entry));
                            entry = Some(parent_entry);
                        }
                        entry.map(|entry| stack.last_mut().unwrap().push_child(entry));
                    }

                    let file_type = walk_entry.file_type().unwrap();
                    let file_name = walk_entry.file_name();
                    if file_type.is_dir() {
                        stack.push(Entry::new_dir(file_name));
                    } else {
                        stack.last_mut().unwrap().push_child(Entry::new_file(file_name));
                    }
                }
            }

            let mut entry = stack.pop().unwrap();
            while let Some(mut parent_entry) = stack.pop() {
                parent_entry.push_child(entry);
                entry = parent_entry;
            }
            inner.entries.push(entry);
        }
    }
}

impl Project {
    pub fn new(paths: Vec<PathBuf>) -> Project {
        Project {
            entries: Vec::with_capacity(paths.len()),
            paths
        }
    }
}

impl Entry {
    fn new_file(name: &OsStr) -> Entry {
        Entry::File { name: name.to_os_string() }
    }

    fn new_dir(name: &OsStr) -> Entry {
        Entry::Dir { name: name.to_os_string(), children: Vec::new() }
    }

    fn push_child(&mut self, child: Entry) {
        match self {
            &mut Entry::Dir { ref mut children, .. } => {
                children.push(child);
            },
            &mut Entry::File { .. } => panic!("Cannot push child to a file")
        }
    }
}
