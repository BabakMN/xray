use futures::{Future, Stream};
use futures::future::Executor;
use std::cell::RefCell;
use std::error;
use std::ffi::OsString;
use std::fmt;
use std::io;
use std::path::PathBuf;
use std::rc::Rc;

pub trait FileTreeIo {
    fn updates(&self) -> Box<'static + Stream<Item=(PathBuf, Option<Entry>), Error=io::Error>>;
}

pub struct FileTree(Rc<RefCell<Inner>>);

struct Inner {
    root: Entry,
    path: PathBuf
}

#[derive(Debug)]
pub enum Entry {
    File {
        name: OsString
    },
    Dir {
        name: OsString,
        children: Vec<Entry>
    }
}

#[derive(Debug)]
pub enum Error {
    NoDirectoryName,
}

impl FileTree {
    pub fn new<E, I>(path: PathBuf, executor: E, io: I) -> Result<Self, Error>
    where
        E: Executor<Box<Future<Item=(), Error=()>>>,
        I: FileTreeIo
    {
        let updates = io.updates();
        let inner = Rc::new(RefCell::new(Inner {
            root: Entry::Dir {
                name: path.file_name().ok_or(Error::NoDirectoryName)?.into(),
                children: Vec::new()
            },
            path
        }));

        let inner_clone = inner.clone();
        executor.execute(
            Box::new(updates.for_each(move |(path, entry)| {
                let mut inner = inner_clone.borrow_mut();
                if let Some(entry) = entry {
                    inner.update_entry(path, entry);
                } else {
                    inner.delete_entry(path);
                }
                Ok(())
            }).then(|_| Ok(())))
        ).expect("Should always be able to executure future on executor");

        Ok(FileTree(inner))
    }
}

impl Inner {
    fn update_entry(&mut self, path: PathBuf, new_entry: Entry) {
        let mut entry = &mut self.root;
        for component in &path {
            if let &mut Entry::Dir {ref mut children, ..} = {entry} {
                if let Some(child_entry) = children.iter_mut().find(|child| child.name() == component) {
                    entry = child_entry;
                } else {
                    panic!("Invalid path in FileTree update");
                }
            } else {
                panic!("Invalid path in FileTree update");
            }
        }
        *entry = new_entry;
    }

    fn delete_entry(&mut self, path: PathBuf) {
        let mut entry = &mut self.root;
        for component in path.iter().take(path.iter().count() - 1) {
            if let &mut Entry::Dir {ref mut children, ..} = {entry} {
                if let Some(child_entry) = children.iter_mut().find(|child| child.name() == component) {
                    entry = child_entry;
                } else {
                    panic!("Invalid path in FileTree update");
                }
            } else {
                panic!("Invalid path in FileTree update");
            }
        }

        let file_name = path.file_name().unwrap();
        entry.children_mut().retain(|child| child.name() != file_name);
    }
}

impl Entry {
    fn name(&self) -> &OsString {
        match self {
            &Entry::Dir { ref name, ..} => name,
            &Entry::File { ref name } => name,
        }
    }

    fn children_mut(&mut self) -> &mut Vec<Entry> {
        match self {
            &mut Entry::Dir { ref mut children, ..} => children,
            &mut Entry::File { .. } => panic!("Tried to get children of a file entry"),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self {
            &Error::NoDirectoryName => "Root directories cannot end in '..'"
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error: {}", error::Error::description(self))
    }
}
