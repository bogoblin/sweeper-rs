#[cfg(target_arch = "wasm32")]
pub mod url {
    #[derive(Debug, Clone)]
    pub struct UrlInfo {
        url: web_sys::Url,
        last_updated: Option<DateTime<Utc>>
    }

    use chrono::{DateTime, Utc};
    use wasm_bindgen::JsValue;

    impl UrlInfo {
        pub fn new() -> Self {
            let window = web_sys::window().ok_or("no window").unwrap();
            let href = window.location().href().unwrap();
            let url = web_sys::Url::new(&href).unwrap();
            Self { url, last_updated: None }
        }

        pub fn get_url(&self) -> String {
            self.url.to_string().as_string().unwrap()
        }

        pub fn update_url(&mut self) -> bool {
            let should_update = match self.last_updated {
                None => {
                    true
                }
                Some(last_updated) => {
                    let since = Utc::now() - last_updated;
                    since.num_milliseconds() > 1000
                }
            };
            
            if should_update {
                self.last_updated = Some(Utc::now());
                let _ = web_sys::window().ok_or("no window").unwrap()
                    .history().unwrap()
                    .replace_state_with_url(&JsValue::null(), "", Some(self.get_url().as_str()));
            }
            
            should_update
        }

        pub fn get_f64(&self, key: &str) -> Option<f64> {
            self.url.search_params().get(key).unwrap_or_default().parse::<f64>().ok()
        }

        pub fn set_f64(&mut self, key: &str, value: f64) {
            self.url.search_params().set(key, &format!("{:.2}", value));
        }
    }
}