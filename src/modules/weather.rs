use reqwest::blocking::Client;
use std::{env, io::Write, os::unix::net::UnixStream, process::Command, thread, time::Duration};
use urlencoding::encode;

pub fn get_weather_info(mut stream: UnixStream, use_curl: bool) {
    let client_ip = if use_curl {
        match Command::new("curl")
            .arg("-s")
            .arg("https://api.ipify.org")
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    String::from_utf8_lossy(&output.stdout).trim().to_string()
                } else {
                    // fallback to auto:ip if curl fails
                    "auto:ip".to_string()
                }
            }
            Err(_) => "auto:ip".to_string(),
        }
    } else {
        "auto:ip".to_string()
    };

    let api_key = match env::var("WEATHER_API") {
        Ok(k) => k,
        Err(_) => {
            let _ = writeln!(stream, r#"{{"error":"WEATHER_API env var not set"}}"#);
            return;
        }
    };

    let client = Client::new();

    loop {
        let url = format!(
            "https://api.weatherapi.com/v1/forecast.json?key={}&q={}&days=3&aqi=no&alerts=no",
            api_key,
            encode(&client_ip)
        );

        match client.get(&url).send() {
            Ok(resp) => {
                if resp.status().is_success() {
                    match resp.text() {
                        Ok(text) => {
                            if writeln!(stream, "{}", text).is_err() {
                                break; // client disconnected
                            }
                        }
                        Err(_) => {
                            let _ = writeln!(
                                stream,
                                r#"{{"error":"Failed to read weather response"}}"#
                            );
                        }
                    }
                } else {
                    let _ = writeln!(
                        stream,
                        r#"{{"error":"Failed to fetch weather data: {}"}}"#,
                        resp.status()
                    );
                }
            }
            Err(e) => {
                let _ = writeln!(stream, r#"{{"error":"Request failed: {}"}}"#, e);
            }
        }

        thread::sleep(Duration::from_hours(1));
    }
}
