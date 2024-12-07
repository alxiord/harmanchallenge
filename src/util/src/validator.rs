use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use super::Error;

pub fn parse_fname(fnamestr: &str) -> Result<PathBuf, Error> {
    let fname = PathBuf::from(fnamestr);
    if !fname.exists() || !fname.is_file() {
        return Err(Error::Io(io::Error::new(
            io::ErrorKind::NotFound,
            "File not found",
        )));
    }

    // must be a file and must be readable = O_R**
    let fmeta = fs::metadata(fname.clone()).map_err(Error::Io)?;
    if fmeta.permissions().mode() >= 0o600 {
        return Ok(fname);
    }
    Err(Error::Io(io::Error::new(
        io::ErrorKind::PermissionDenied,
        "File not readable",
    )))
}

pub fn parse_format(format: &str) -> Result<super::VideoFormat, Error> {
    if format.eq_ignore_ascii_case("h264") {
        return Ok(super::VideoFormat::H264);
    }
    Err(super::Error::Format(format.to_string()))
}
