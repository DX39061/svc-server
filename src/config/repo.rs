use serde::Deserialize;

#[derive(Deserialize)]
pub struct RepoConfig {
    pub root_path: String
}