use anyhow::Result;
use serde::Deserialize;

const BGS_JSON_URL: &str = "https://a-mackay.github.io/bgs/bgs.json";

pub(crate) async fn get_bg_names() -> Result<Vec<String>> {
    let bgs: Vec<BgDto> = reqwest::get(BGS_JSON_URL).await?.json().await?;
    Ok(bgs.into_iter().map(|bg_dto| bg_dto.name).collect())
}

#[derive(Debug, Deserialize)]
struct BgDto {
    name: String,
}
