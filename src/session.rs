use std::{fs, io, path::Path};
use uuid::Uuid;

/// Persist before connecting so a closed guest terminal can reclaim its seat.
pub fn identity(path: &Path) -> io::Result<String> {
    match fs::read_to_string(path) {
        Ok(token) => {
            Uuid::parse_str(token.trim()).map_err(io::Error::other)?;
            Ok(token.trim().into())
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            let token = Uuid::new_v4().to_string();
            let mut options = fs::OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            use std::io::Write;
            options.open(path)?.write_all(token.as_bytes())?;
            Ok(token)
        }
        Err(e) => Err(e),
    }
}
