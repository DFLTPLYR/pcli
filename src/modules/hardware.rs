use std::{
    io::{BufRead, BufReader},
    os::unix::net::UnixStream,
};

pub fn get_hardware_info() -> Result<(), Box<dyn std::error::Error>> {
    let stream = UnixStream::connect("/tmp/sysinfo.sock")?;
    let reader = BufReader::new(stream);
    for line in reader.lines() {
        let line = line?;
        println!("{}", line);
    }
    Ok(())
}
