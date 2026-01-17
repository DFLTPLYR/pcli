use std::process::Command;

pub fn generate_color_palette(paths: Vec<String>) {
    if let Some(path) = combine_wallpaper(paths) {
        let mut cmd = Command::new("matugen");
        cmd.arg("image")
            .arg(path)
            .spawn()
            .expect("shiiii you got an issue");
    } else {
        eprintln!("Config path not found.");
    }
}

fn combine_wallpaper(paths: Vec<String>) -> Option<String> {
    let output = "/tmp/combined_wallpaper.png".to_string();
    let mut cmd = Command::new("magick");
    for path in paths {
        cmd.arg("(")
            .arg(&path)
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
