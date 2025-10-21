use tokio::time::sleep;

use crate::getdata;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use tauri::{AppHandle, Emitter};

#[derive(serde::Serialize, Clone)]
struct AnalyseSymbolInfo {
    name: String,
    symbol: String,
    progress: i32,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct StockData {
    name: String,
    symbol: String,
    data: Vec<(String, f32)>,
}

#[derive(serde::Serialize)]
pub struct AnalysedStockData {
    name: String,
    symbol: String,
    data: Vec<(String, f32)>,
    movingavg: Vec<(String, f32)>,
    monthinc: f32,
    growthscore: f32,
    volatility: f32,
    unpredictability: f32,
    score: f32,
}

const FILE: &str = "historical";

#[tauri::command]
pub async fn get_analysed_results(
    app: AppHandle,
    use_cache: bool,
) -> Result<Vec<AnalysedStockData>, String> {
    if !use_cache {
        std::fs::remove_file(FILE).map_err(|e| e.to_string())?;
    };
    let data = get_data_from_cache(&app).await?;
    let analysed_data = analyse_data(data).await;
    Ok(analysed_data)
}

async fn download_data(app: &AppHandle) -> Result<Vec<StockData>, String> {
    let symbols = get_symbols().await?;
    let mut progress = 0;
    let login_token = getdata::get_login_token().await?;
    let mut data: Vec<StockData> = Vec::new();
    for s in symbols.iter() {
        // tell frontend the progress
        progress += 1;
        app.emit(
            "downloading-symbol",
            AnalyseSymbolInfo {
                name: s.0.clone(),
                symbol: s.1.clone(),
                progress: progress,
            },
        )
        .map_err(|e| e.to_string())?;

        let parsed = get_parsed_data(&s.1, &login_token).await?;
        data.push(StockData {
            name: s.0.clone(),
            symbol: s.1.clone(),
            data: parsed,
        });
    }
    Ok(data)
}

async fn get_data_from_cache(app: &AppHandle) -> Result<Vec<StockData>, String> {
    let file = File::open(FILE);
    match file {
        Ok(mut f) => {
            // read from file
            let mut contents = String::new();
            f.read_to_string(&mut contents).map_err(|e| e.to_string())?;
            let data: Vec<StockData> =
                serde_json::from_str(&contents).map_err(|e| e.to_string())?;
            Ok(data)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // Download the data
            let fresh_data = download_data(app).await?;
            let s = serde_json::to_string(&fresh_data).map_err(|e| e.to_string())?;
            let mut f = File::create(FILE).map_err(|e| e.to_string())?;
            f.write_all(s.as_bytes()).map_err(|e| e.to_string())?;
            Ok(fresh_data)
        }
        Err(e) => Err(e.to_string()),
    }
}

async fn get_symbols() -> Result<Vec<(String, String)>, String> {
    let mut symbols = Vec::new();
    for x in 0..5 {
        let page = get_symbol_page(x).await?;
        symbols.extend(page);
    }
    Ok(symbols)
}

async fn get_symbol_page(n: i32) -> Result<Vec<(String, String)>, String> {
    const URL: &str = "https://api.londonstockexchange.com/api/v1/components/refresh";
    let data = format!(
        "{{\"path\": \"ftse-constituents\", \"parameters\":\
        \"indexname%3Dftse-100%26tab%3Dtable%26page%3D{}%26tabId%3D1602cf04-c25b\
        -4ea0-a9d6-64040d217877\", \"components\": [{{\"componentId\": \"block_conte\
        nt%3Aafe540a2-2a0c-46af-8497-407dc4c7fd71\"}}]}}",
        n
    );

    let response = reqwest::Client::new()
        .post(URL)
        .body(data)
        .header("Content-Type", "application/json")
        .send()
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;

    let json: serde_json::Value = serde_json::from_str(&response).map_err(|e| e.to_string())?;
    let array = json
        .get(0)
        .and_then(|v| v.get("content"))
        .and_then(|v| v.get(0))
        .and_then(|v| v.get("value"))
        .and_then(|v| v.get("content"))
        .and_then(|v| v.as_array())
        .ok_or("Failed to parse response")?
        .iter()
        .map(|v| {
            let Some(name) = v.get("issuername").and_then(|v| v.as_str()) else {
                return None;
            };
            let Some(symbol) = v.get("tidm").and_then(|v| v.as_str()) else {
                return None;
            };
            Some((name.to_string(), symbol.to_string()))
        })
        .flatten()
        .collect();

    Ok(array)
}

async fn get_parsed_data(
    symbol: &String,
    login_token: &String,
) -> Result<Vec<(String, f32)>, String> {
    let data = {
        const TRIES: i32 = 3;
        let mut i = 0;
        loop {
            let res = getdata::make_request_token(symbol, login_token).await;
            if res.is_ok() || i >= TRIES {
                break res;
            }
            println!("Trying again");
            sleep(std::time::Duration::from_secs(1)).await;
            i += 1;
        }
    }
    .map_err(|e| e.to_string())?;

    let json: serde_json::Value = serde_json::from_str(&data.data).map_err(|e| e.to_string())?;
    json.get("data")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .map(|v| {
                    let a = v
                        .get("_DATE_END")
                        .and_then(|d| d.as_str())
                        .map(|d| d.to_string());
                    let b = v
                        .get("CLOSE_PRC")
                        .and_then(|p| p.as_str())
                        .and_then(|p| p.parse::<f32>().ok());
                    a.and_then(|a| b.map(|b| (a, b)))
                })
                .flatten()
                .collect()
        })
        .ok_or("Failed to parse response".to_string())
}

async fn analyse_data(data: Vec<StockData>) -> Vec<AnalysedStockData> {
    let mut results: Vec<AnalysedStockData> = data
        .into_iter()
        .map(|s| {
            let trimmed_data = {
                let len = s.data.len();
                const NO_ENTRIES: usize = 365 * 10;
                if NO_ENTRIES > len {
                    s.data
                } else {
                    s.data.into_iter().skip(len - NO_ENTRIES).collect()
                }
            };
            let ma = moving_average(&trimmed_data, 50);
            if ma.len() < 30 {
                return None;
            }
            let month_start = ma[ma.len() - 30].1;
            let month_end = ma.last().unwrap().1;
            let year_inc = if ma.len() >= 365 {
                ma.last().unwrap().1 / ma[ma.len() - 365].1
            } else {
                1.0
            };
            let gs = year_inc * stock_growth_score(&ma);
            let vol = stock_volatility(&trimmed_data);
            let unpredictability = unpredictability(&trimmed_data);
            Some(AnalysedStockData {
                name: s.name,
                symbol: s.symbol,
                data: trimmed_data,
                movingavg: ma,
                monthinc: ((month_end - month_start) / month_start) * 100.0,
                growthscore: gs,
                volatility: vol,
                unpredictability,
                score: gs / (vol * unpredictability),
            })
        })
        .flatten()
        .collect();
    results.sort_by(|a, b| b.score.total_cmp(&a.score));
    results
}

fn moving_average(data: &Vec<(String, f32)>, period: usize) -> Vec<(String, f32)> {
    let mut sum: f32 = 0.0;
    let mut result = Vec::new();
    for (i, (d, v)) in data.iter().enumerate() {
        if i + 1 < period {
            // cannot compute avg yet
            sum += v;
        } else {
            // move average
            sum += v;
            result.push((d.clone(), sum / period as f32));
            sum -= data[i + 1 - period].1;
        }
    }
    return result;
}

fn stock_growth_score(ma: &Vec<(String, f32)>) -> f32 {
    // for as long as ma is going up, sum the difference
    let mut prev: f32 = ma.last().unwrap().1;
    let length = ma.len();
    let dir = ma[length - 1].1.total_cmp(&ma[length - 2].1);
    let mut i = 0;
    for (_, p) in ma.iter().rev().skip(1) {
        i += 1;
        if prev.total_cmp(p) != dir {
            break;
        }
        prev = *p;
    }
    let ans = 100.0 * (ma.last().unwrap().1 - prev) / prev;
    ans * (f32::log2(i as f32) + 1.0)
}

fn stock_volatility(ma: &Vec<(String, f32)>) -> f32 {
    let normed: Vec<f32> = ma.windows(2).map(|v| 100.0 * v[1].1 / v[0].1).collect();
    let n = normed.len();
    let mean: f32 = normed.iter().map(|v| v).sum::<f32>() / n as f32;
    let std_dev = (normed.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / n as f32).sqrt();
    100.0 * std_dev / (n as f32 / 365.0).sqrt()
}

fn unpredictability(data: &Vec<(String, f32)>) -> f32 {
    const HALF_WINDOW: usize = 50;
    const WINDOW: usize = 2 * HALF_WINDOW;
    let len = data.len();
    let mut res: Vec<f32> = Vec::new();
    let first = data.first().unwrap().1;
    let data: Vec<f32> = data.into_iter().map(|v| v.1 / first).collect();
    let ma = {
        // a centred moving average
        let mut left: f32 = data.iter().take(HALF_WINDOW).sum();
        let mut right: f32 = data.iter().skip(HALF_WINDOW + 1).take(HALF_WINDOW).sum();
        for (i, p) in data.iter().enumerate().skip(HALF_WINDOW).take(len - WINDOW) {
            let avg = (left + p + right) / ((1 + WINDOW) as f32);
            res.push(avg);
            // update left and right
            left += p - data[i - HALF_WINDOW];
            if i + HALF_WINDOW < data.len() {
                right += data[i + HALF_WINDOW] - data[i + 1];
            }
        }
        res
    };

    let n = ma.len();
    let trimmed_data: Vec<f32> = data
        .into_iter()
        .skip(HALF_WINDOW)
        .take(n - HALF_WINDOW)
        .collect();
    let var = trimmed_data
        .iter()
        .enumerate()
        .map(|(i, v)| (ma[i] - v).exp2())
        .sum::<f32>()
        / n as f32;
    var
}
