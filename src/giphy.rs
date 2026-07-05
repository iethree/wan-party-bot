//! Port of `giphy.py`. Uses the Giphy "random" REST endpoint directly (the Python
//! `giphy_client` library does the same under the hood). Not called by the live
//! bot, but reproduced faithfully including the fallback URL on any error.

const FALLBACK: &str = "http://gph.is/2efdN3V";

pub async fn random_gif(search: &str) -> String {
    let api_key = std::env::var("GIPHY_TOKEN").unwrap_or_default();
    let url = "https://api.giphy.com/v1/gifs/random";
    let result: Result<String, ()> = async {
        let client = reqwest::Client::new();
        let resp = client
            .get(url)
            .query(&[
                ("api_key", api_key.as_str()),
                ("tag", search),
                ("rating", "g"),
                ("fmt", "json"),
            ])
            .send()
            .await
            .map_err(|_| ())?;
        let json: serde_json::Value = resp.json().await.map_err(|_| ())?;
        json.get("data")
            .and_then(|d| d.get("image_url"))
            .and_then(|u| u.as_str())
            .map(|s| s.to_string())
            .ok_or(())
    }
    .await;

    result.unwrap_or_else(|_| FALLBACK.to_string())
}
