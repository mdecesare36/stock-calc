use std::fs::File;
use std::io::Read;
use std::io::Write;

const FILE: &str = "portfolio";

#[tauri::command]
pub fn get_portfolio() -> Result<Vec<String>, String> {
    let mut file = File::open(FILE)
        .or_else(|_| File::create(FILE))
        .map_err(|e| e.to_string())?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|e| e.to_string())?;
    Ok(contents.lines().map(|s| s.to_string()).collect())
}

#[tauri::command]
pub fn set_portfolio(portfolio: Vec<String>) -> Result<(), String> {
    let mut file = File::create(FILE).map_err(|e| e.to_string())?;
    for s in portfolio.iter() {
        writeln!(file, "{}", s).map_err(|e| e.to_string())?;
    }
    Ok(())
}
