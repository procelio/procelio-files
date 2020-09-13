use serde::{Serialize, Deserialize};
use serde;
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct UpdateInfo {
  pub title: String,
  pub version: [u32; 3], // ([major, minor, patch])
  pub dev: bool, // true iff dev build
  pub description: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub hyperlink: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub image: Option<String>
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct LauncherConfiguration {
    #[serde(rename = "websiteUrl")] 
    pub website_url: String, // URL of procelio website
    pub updates: Vec<UpdateInfo>, // List of updates of the game
    #[serde(rename = "launcherVersion")] 
    pub launcher_version: Vec<u32>,
    #[serde(rename = "quoteOfTheDay")] 
    pub quote_of_the_day: String,
    #[serde(rename = "quoteAuthor")] 
    pub quote_author: String,
}
