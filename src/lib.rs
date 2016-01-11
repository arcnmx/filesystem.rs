use std::borrow::Cow;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::io;
use std::fs;

pub trait Filesystem {
    type Error: Error + Send + Sync + 'static;
    type File: io::Read + io::Write + io::Seek;

    fn create_dir(&self, path: &Path) -> Result<(), Self::Error>;
    fn create_dir_all(&self, path: &Path) -> Result<(), Self::Error>;
    fn create(&self, path: &Path) -> Result<Self::File, Self::Error>;
    fn open(&self, path: &Path) -> Result<Self::File, Self::Error>;
}

pub struct StdFilesystem(Option<PathBuf>);

impl StdFilesystem {
    pub fn new<P: Into<PathBuf>>(root: P) -> Self {
        StdFilesystem(Some(root.into()))
    }

    fn path<'a>(&self, path: &'a Path) -> Cow<'a, Path> {
        if let Some(mut root) = self.0.as_ref().map(Clone::clone) {
            root.push(path);
            Cow::Owned(root)
        } else {
            Cow::Borrowed(path)
        }
    }
}

impl Default for StdFilesystem {
    fn default() -> Self {
        StdFilesystem(None)
    }
}

impl Filesystem for StdFilesystem {
    type Error = io::Error;
    type File = fs::File;

    fn create(&self, path: &Path) -> Result<Self::File, Self::Error> {
        fs::File::create(self.path(path))
    }

    fn open(&self, path: &Path) -> Result<Self::File, Self::Error> {
        fs::File::open(self.path(path))
    }

    fn create_dir(&self, path: &Path) -> Result<(), Self::Error> {
        fs::create_dir(self.path(path))
    }

    fn create_dir_all(&self, path: &Path) -> Result<(), Self::Error> {
        fs::create_dir_all(self.path(path))
    }
}

pub type FilesystemObject = Filesystem<Error = io::Error, File = Box<ReadWriteSeek>>;
pub struct FilesystemDynamic<F>(F);
pub struct FileDynamic<F>(F);
pub trait ReadWriteSeek: io::Read + io::Write + io::Seek { }
impl<F: io::Read + io::Write + io::Seek> ReadWriteSeek for F { }

fn io_error<E: Error + Send + Sync + 'static>(e: E) -> io::Error {
    io::Error::new(io::ErrorKind::Other, e)
}

impl<'a, F: Filesystem + 'a> Filesystem for FilesystemDynamic<F> {
    type Error = io::Error;
    type File = Box<ReadWriteSeek + 'a>;

    fn create(&self, path: &Path) -> Result<Self::File, Self::Error> {
        self.0.create(path).map(|f| Box::new(FileDynamic(f)) as Box<_>).map_err(io_error)
    }

    fn open(&self, path: &Path) -> Result<Self::File, Self::Error> {
        self.0.open(path).map(|f| Box::new(FileDynamic(f)) as Box<_>).map_err(io_error)
    }

    fn create_dir(&self, path: &Path) -> Result<(), Self::Error> {
        self.0.create_dir(path).map_err(io_error)
    }

    fn create_dir_all(&self, path: &Path) -> Result<(), Self::Error> {
        self.0.create_dir_all(path).map_err(io_error)
    }
}

impl<F: io::Read> io::Read for FileDynamic<F> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

impl<F: io::Write> io::Write for FileDynamic<F> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

impl<F: io::Seek> io::Seek for FileDynamic<F> {
    fn seek(&mut self, seek: io::SeekFrom) -> io::Result<u64> {
        self.0.seek(seek)
    }
}
