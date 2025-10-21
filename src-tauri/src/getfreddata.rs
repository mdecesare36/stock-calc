const API_KEY: &str = "8b59534636cbd01760496ecfbfa4962b";

#[derive(serde::Serialize)]
pub struct FredResponse {
    title: String,
    data: Vec<(String, String)>,
}

#[tauri::command]
pub async fn get_fred_data(series_code: &str) -> Result<FredResponse, String> {
    let response = reqwest::get(format!(
        "https://api.stlouisfed.org/fred/series/observations?series_id={}&api_key={}&file_type=json",
        series_code, API_KEY
    )).await.map_err(|e| e.to_string())?;
    let text = response.text().await.map_err(|e| e.to_string())?;
    let json: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    let data = json
        .get("observations")
        .and_then(|v| v.as_array())
        .ok_or("Failed to parse observations".to_string())?
        .iter()
        .map(|v| -> Option<(String, String)> {
            let date = v.get("date").and_then(|d| d.as_str())?.to_string();
            let value = v.get("value").and_then(|v| v.as_str())?.to_string();
            Some((date, value))
        })
        .filter(|x| x.is_some())
        .map(|x| x.unwrap())
        .collect();
    let title = get_series_title(series_code).await?;
    Ok(FredResponse { title, data })
}

async fn get_series_title(series_code: &str) -> Result<String, String> {
    let url = format!(
        "https://api.stlouisfed.org/fred/series?series_id={}&api_key={}&file_type=json",
        series_code, API_KEY
    );
    let response = reqwest::get(&url)
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;

    serde_json::from_str::<serde_json::Value>(&response)
        .map_err(|e| e.to_string())?
        .get("seriess")
        .and_then(|s| s.as_array())
        .and_then(|s| s.first())
        .and_then(|s| s.get("title"))
        .and_then(|t| t.as_str())
        .map(|t| t.to_string())
        .ok_or("Failed to get series title".to_string())
}
