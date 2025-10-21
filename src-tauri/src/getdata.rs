#[derive(serde::Serialize)]
pub struct StockResponse {
    pub data: String,
    pub display_name: String,
}

#[tauri::command]
pub async fn make_request(code: String) -> Result<StockResponse, String> {
    let login_token = get_login_token().await?;
    make_request_token(&code, &login_token).await
}

pub async fn make_request_token(code: &String, token: &String) -> Result<StockResponse, String> {
    let ric = get_ric_from_tidm(code).await?;
    let hist_data = get_historical(&token, &ric).await?;
    let display_name = get_display_name(&ric, &token).await?;
    Ok(StockResponse {
        data: hist_data,
        display_name: display_name,
    })
}

async fn get_historical(login_token: &String, code: &String) -> Result<String, String> {
    let now = chrono::Utc::now()
        .with_time(chrono::NaiveTime::from_hms_opt(23, 59, 59).unwrap())
        .single()
        .unwrap()
        .format("%Y-%m-%dT%H:%M:%S")
        .to_string();
    let url = format!(
        "https://refinitiv-widgets.financial.com/rest/\
    api/timeseries/historical?ric={}&fids=_DATE_END,CLOSE_PRC,HIGH_1,\
    OPEN_PRC,LOW_1&samples=D&appendRecentData=all&toDate={}&\
    fromDate=1970-01-01T00:00:00",
        code, now
    );
    let client = reqwest::Client::new();
    let res = client
        .get(url)
        .header("jwt", login_token)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;
    Ok(res)
}

async fn get_saml() -> Result<String, String> {
    const URL: &str = "https://api.londonstockexchange.com/api/gw/feedhandler/token/saml";
    let response = reqwest::get(URL).await.map_err(|e| e.to_string())?;
    let json = response.text().await.map_err(|e| e.to_string())?;
    let saml_response: serde_json::Value =
        serde_json::from_str(&json).map_err(|e| e.to_string())?;
    Ok(saml_response
        .get("encodedToken")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or("Failed to parse SAML response".to_string())?)
}

pub async fn get_login_token() -> Result<String, String> {
    let saml = get_saml().await?;
    let client = reqwest::Client::new();
    let body = format!("SAMLResponse={}", urlencoding::encode(&saml));
    const URL: &str =
        "https://refinitiv-widgets.financial.com/auth/api/v1/sessions/samllogin?fetchToken=true";
    let res = client
        .post(URL)
        .header("content-type", "application/x-www-form-urlencoded")
        .header("referrer", "https://www.londonstockexchange.com/")
        .header("referrerPolicy", "strict-origin-when-cross-origin")
        .body(body)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let json = res.text().await.map_err(|e| e.to_string())?;
    let json: serde_json::Value = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    Ok(json
        .get("token")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or("Failed to parse JSON".to_string())?)
}

async fn get_ric_from_tidm(tidm: &String) -> Result<String, String> {
    let url = format!(
        "https://api.londonstockexchange.com/api/gw/feedhandler/translate/ric?category=EQUITY&tidm={}",
        tidm
    );

    let response = reqwest::get(url).await.map_err(|e| e.to_string())?;
    let json = response.text().await.map_err(|e| e.to_string())?;
    let json: serde_json::Value = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    json.get(0)
        .and_then(|v| v.get("ric"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or("Failed to parse response".to_string())
}

async fn get_display_name(code: &String, token: &String) -> Result<String, String> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://refinitiv-widgets.financial.com/rest/api/quote/info?rics=\
        {}&fids=x._DSPLY_NAME",
        code
    );
    let res = client
        .get(url)
        .header("jwt", token)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;
    let json: serde_json::Value = serde_json::from_str(&res).map_err(|e| e.to_string())?;
    Ok(json
        .get("data")
        .and_then(|v| v.get(0))
        .and_then(|v| v.get("x._DSPLY_NAME"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or(String::new()))
}
