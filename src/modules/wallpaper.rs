use std::{io::Write, os::unix::net::UnixStream, process::Command};

pub fn generate_color_palette(type_: String, paths: Vec<String>, mut stream: UnixStream) {
    if let Some(path) = combine_wallpaper(paths) {
        let mut cmd = Command::new("matugen");
        let mut child = cmd
            .arg("-t")
            .arg(type_)
            .arg("image")
            .arg(path)
            .spawn()
            .expect("Failed to spawn matugen");
        let status = child.wait().expect("Failed to wait for matugen");
        if status.success() {
            writeln!(stream, "true").expect("Failed to write to stream");
        } else {
            writeln!(stream, "false").expect("Failed to write to stream");
        }
    } else {
        writeln!(stream, "false").expect("Failed to write to stream");
    }
    stream
        .shutdown(std::net::Shutdown::Write)
        .expect("Failed to shutdown stream");
}

fn combine_wallpaper(paths: Vec<String>) -> Option<String> {
    let output = "/tmp/combined_wallpaper.png".to_string();
    let mut cmd = Command::new("magick");
    for path in paths {
        let local_path = if let Some(stripped) = path.strip_prefix("file://") {
            stripped
        } else {
            &path
        };
        cmd.arg("(")
            .arg(local_path)
            .arg("-resize")
            .arg("960x1080!")
            .arg(")");
    }
    cmd.arg("+append").arg(&output);
    match cmd.status() {
        Ok(status) if status.success() => Some(output),
        _ => None,
    }
}
