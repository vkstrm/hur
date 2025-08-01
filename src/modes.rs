#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, clap::ValueEnum)]
#[serde(rename_all = "UPPERCASE")]
pub enum RedirectMode {
    Follow,
    NoFollow,
    Interactive,
}
