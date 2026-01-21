use reqwest::blocking;
use std::{env, io::Write, os::unix::net::UnixStream, thread, time::Duration};
use urlencoding::encode;

pub fn get_weather_info(mut stream: UnixStream, client_ip: Option<String>) {
    let client_ip = client_ip.unwrap_or_else(|| "auto:ip".to_string());
    let api_key = match env::var("WEATHER_API") {
        Ok(k) => k,
        Err(_) => {
            let _ = writeln!(stream, r#"{{"error":"WEATHER_API env var not set"}}"#);
            return;
        }
    };

    loop {
        // Build the WeatherAPI URL
        let url = format!(
            "https://api.weatherapi.com/v1/forecast.json?key={}&q={}&days=3&aqi=no&alerts=no",
            api_key,
            encode(&client_ip)
        );

        // Fetch the weather
        let resp = blocking::get(&url);

        match resp {
            Ok(r) => {
                if !r.status().is_success() {
                    let _ = writeln!(
                        stream,
                        r#"{{"error":"Failed to fetch weather data: {}"}}"#,
                        r.status()
                    );
                } else {
                    match r.text() {
                        Ok(text) => {
                            let _ = writeln!(stream, "{}", text);
                        }
                        Err(_) => {
                            let _ = writeln!(
                                stream,
                                r#"{{"error":"Failed to read weather response"}}"#
                            );
                        }
                    }
                }
            }
            Err(e) => {
                let _ = writeln!(stream, r#"{{"error":"Request failed: {}"}}"#, e);
            }
        }

        // Sleep between updates (e.g., every 60 seconds)
        thread::sleep(Duration::from_secs(60));
    }
}
