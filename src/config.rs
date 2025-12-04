use std::{collections::HashMap, path::Path};

use {
  anyhow::{Context, Result},
  config::Config as ConfigBuilder,
  grammers_session::defs::PeerId,
  serde::{Deserialize, Serialize},
};

// Constants
pub const DEFAULT_SESSION_FILE: &str = "userbot.session";
pub const DEFAULT_DEBOUNCE_SECONDS: u64 = 1;
pub const DEFAULT_HISTORY_LIMIT: usize = 25;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
  pub telegram: TelegramConfig,
  pub ai: AiConfig,
  pub settings: Settings,
  #[serde(default)]
  pub users: Vec<TrackedUser>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramConfig {
  pub api_id: i32,
  pub api_hash: String,
  pub bot_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
  pub api_key: String,
  pub api_url: String,
  #[serde(default)]
  pub model: String,
  #[serde(default)]
  pub models: Vec<String>,
  #[serde(default = "default_temperature")]
  pub temperature: f32,
}

impl AiConfig {
  /// Returns a list of models to try in priority order
  pub fn models_priority(&self) -> Vec<String> {
    if !self.models.is_empty() {
      self.models.clone()
    } else if !self.model.is_empty() {
      vec![self.model.clone()]
    } else {
      vec![]
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
  #[serde(default = "default_session_file")]
  pub session_file: String,
  #[serde(default = "default_debounce")]
  pub debounce_seconds: u64,
  #[serde(default = "default_history_limit")]
  pub history_limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackedUser {
  pub id: i64,
  pub name: String,
  pub system_prompt: String,
}

impl TrackedUser {
  pub fn user_id(&self) -> PeerId {
    PeerId::user(self.id)
  }

  pub fn chat_id(&self) -> PeerId {
    PeerId::chat(self.id)
  }
}

fn default_temperature() -> f32 {
  1.5
}

fn default_session_file() -> String {
  DEFAULT_SESSION_FILE.to_string()
}

fn default_debounce() -> u64 {
  DEFAULT_DEBOUNCE_SECONDS
}

fn default_history_limit() -> usize {
  DEFAULT_HISTORY_LIMIT
}

impl Config {
  pub fn load(path: impl AsRef<Path>) -> Result<Self> {
    let path = path.as_ref();

    let config = ConfigBuilder::builder()
      .add_source(config::File::from(path))
      .build()
      .with_context(|| {
        format!("Failed to load config file: {}", path.display())
      })?;

    let config: Config = config.try_deserialize().with_context(|| {
      format!("Failed to parse config file: {}", path.display())
    })?;

    Ok(config)
  }

  pub fn users_map(&self) -> HashMap<PeerId, TrackedUser> {
    // Map both user and chat IDs to handle different peer types
    let mut map = HashMap::new();
    for user in &self.users {
      map.insert(user.user_id(), user.clone());
      map.insert(user.chat_id(), user.clone());
    }
    map
  }
}
