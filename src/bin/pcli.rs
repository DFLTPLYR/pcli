use std::{
    io::{BufRead, BufReader},
    os::unix::net::UnixStream,
};

fn main() {
    let stream = UnixStream::connect("/tmp/sysinfo.sock")
        .expect("daemon not running");

    let reader = BufReader::new(stream);

    for line in reader.lines() {
        println!("{}", line.unwrap());
    }
}

