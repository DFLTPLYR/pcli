use reqwest::blocking::Client;
use std::{
    env,
    io::Write,
    os::unix::net::UnixStream,
    process::Command,
    sync::{Arc, LazyLock, Mutex},
    thread,
    time::{Duration, Instant},
};
use urlencoding::encode;

static WEATHER_CACHE: LazyLock<Arc<Mutex<Option<(String, Instant)>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(None)));

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
        let now = Instant::now();
        let cached_data = {
            let cache = WEATHER_CACHE.lock().unwrap();
            cache.as_ref().and_then(|(data, timestamp)| {
                if now.duration_since(*timestamp) < Duration::from_secs(3600) {
                    Some(data.clone())
                } else {
                    None
                }
            })
        };
        let weather_data = match cached_data {
            Some(data) => data,
            None => {
                // Fetch fresh data
                let url = format!(
                    "https://api.weatherapi.com/v1/forecast.json?key={}&q={}&days=3&aqi=no&alerts=no",
                    api_key,
                    encode(&client_ip)
                );
                match client.get(&url).send() {
                    Ok(resp) if resp.status().is_success() => {
                        match resp.text() {
                            Ok(text) => {
                                // Update cache
                                {
                                    let mut cache = WEATHER_CACHE.lock().unwrap();
                                    *cache = Some((text.clone(), now));
                                }
                                text
                            }
                            Err(_) => {
                                let _ = writeln!(
                                    stream,
                                    r#"{{"error":"Failed to read weather response"}}"#
                                );
                                continue;
                            }
                        }
                    }
                    Ok(resp) => {
                        let _ = writeln!(
                            stream,
                            r#"{{"error":"Failed to fetch weather data: {}"}}"#,
                            resp.status()
                        );
                        continue;
                    }
                    Err(e) => {
                        let _ = writeln!(stream, r#"{{"error":"Request failed: {}"}}"#, e);
                        continue;
                    }
                }
            }
        };
        if writeln!(stream, "{}", weather_data).is_err() {
            break; // client disconnected
        }
        thread::sleep(Duration::from_secs(60)); // Check every minute for new connections
    }
}
