use std::{
    fs::File,
    io,
    os::fd::OwnedFd,
    path::PathBuf,
    fmt,
};

pub enum FileMode {
    Open,
    Write,
    Append,
    Read,
}

#[derive(Debug)]
pub enum FileError {
    UnknownFileMode,
    FileNotFound,
    PathError,
    IOError(io::Error),
}

impl fmt::Display for FileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownFileMode => write!(f, "UnknownFileMode provided"),
            Self::FileNotFound => write!(f, "FileNotFound: verify Path"),
            Self::PathError => write!(f, "PathError: Invalid Path"),
            Self::IOError(error) => write!(f, "File IOError: {}", error),
        }
    }
}

pub fn open_file(path: PathBuf, mode: FileMode) -> Result<OwnedFd, FileError> {
    let fd = match mode {
        FileMode::Write => {
            File::options()
                .write(true)
                .create(true)
                .open(path.as_path())
        },
        FileMode::Append => {
            File::options()
                .write(true)
                .create(true)
                .append(true)
                .open(path.as_path())
        },
        FileMode::Read => {
            File::options()
                .read(true)
                .open(path.as_path())
        },
        _ => {return Err(FileError::UnknownFileMode)}
    };
    match fd {
        Ok(file) => Ok(OwnedFd::from(file)),
        Err(e) => {return Err(FileError::IOError(e))}
    }
}
