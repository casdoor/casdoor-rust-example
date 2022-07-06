// get a absolute path from a relative path
pub fn abs_path(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let absolute_path = std::env::current_dir()?.join(path);
    Ok(absolute_path.to_str().unwrap().to_string())
}
