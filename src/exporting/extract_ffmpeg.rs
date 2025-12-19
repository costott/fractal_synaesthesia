#[cfg(target_os = "windows")]
const FFMPEG_BYTES: &[u8] = include_bytes!("../../ffmpeg/windows-x86_64/bin/ffmpeg.exe");

#[cfg(target_os = "linux")]
const FFMPEG_BYTES: &[u8] = todo!();

#[cfg(target_os = "macos")]
const FFMPEG_BYTES: &[u8] = todo!();

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

pub fn extract_ffmpeg() -> std::io::Result<PathBuf> {
    let mut path = env::temp_dir();
    path.push("ffmpeg_embedded");

    #[cfg(target_os = "windows")]
    path.set_extension("exe");

    let mut file = File::create(&path)?;
    file.write_all(FFMPEG_BYTES)?;

    // On Unix, make it executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o755);
        std::fs::set_permissions(&path, perms)?;
    }

    Ok(path)
}
