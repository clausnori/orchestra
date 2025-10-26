use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config{
    pub project: ProjectConfig,
    pub employee: Vec<Employee>,
    pub manager: Vec<Manager>,
}

#[derive(Deserialize, Debug)]
pub struct ProjectConfig {
  pub dir: String,
  pub ignore_dir: Vec<String>,
  pub max_size: u64
}

#[derive(Debug, Deserialize,Clone)]
pub struct Employee {
    pub dir: String,
    pub name: String,
    pub task: String,
}

#[derive(Debug, Deserialize)]
pub struct Manager {
    pub dir: String,
    pub name: String,
    pub level: String,
    pub team: Vec<String>,
}