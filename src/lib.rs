use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use fs_extra::dir::{copy as copy_dir, CopyOptions};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillRecord {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub tags: Vec<String>,
    pub source: String,
    pub popularity: u64,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolTarget {
    pub tool: String,
    pub path: PathBuf,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceRepo {
    pub name: String,
    pub git_url: String,
    pub local_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub global_store: PathBuf,
    pub tools: Vec<ToolTarget>,
    pub sources: Vec<SourceRepo>,
}

impl Default for AppConfig {
    fn default() -> Self {
        let root = default_root_dir();
        Self {
            global_store: root.join("store"),
            tools: vec![],
            sources: default_sources(root.join("sources")),
        }
    }
}

fn default_sources(base: PathBuf) -> Vec<SourceRepo> {
    vec![
        SourceRepo {
            name: "awesome-ai-skills".to_string(),
            git_url: "https://github.com/example/awesome-ai-skills.git".to_string(),
            local_path: base.join("awesome-ai-skills"),
        },
        SourceRepo {
            name: "open-skill-registry".to_string(),
            git_url: "https://github.com/example/open-skill-registry.git".to_string(),
            local_path: base.join("open-skill-registry"),
        },
    ]
}

pub fn default_root_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".skills-manage")
}

pub fn config_path() -> PathBuf {
    default_root_dir().join("config.toml")
}

pub fn load_or_init_config() -> Result<AppConfig> {
    let path = config_path();
    if path.exists() {
        let text = fs::read_to_string(&path).context("读取配置失败")?;
        let cfg: AppConfig = toml::from_str(&text).context("配置格式错误")?;
        Ok(cfg)
    } else {
        let cfg = AppConfig::default();
        save_config(&cfg)?;
        Ok(cfg)
    }
}

pub fn save_config(cfg: &AppConfig) -> Result<()> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).context("创建配置目录失败")?;
    }
    let text = toml::to_string_pretty(cfg).context("序列化配置失败")?;
    fs::write(path, text).context("写入配置失败")?;
    Ok(())
}

fn manifest_path(store: &Path) -> PathBuf {
    store.join("manifest.json")
}

pub fn load_manifest(store: &Path) -> Result<HashMap<String, SkillRecord>> {
    let path = manifest_path(store);
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let text = fs::read_to_string(path)?;
    let map: HashMap<String, SkillRecord> = serde_json::from_str(&text)?;
    Ok(map)
}

pub fn save_manifest(store: &Path, skills: &HashMap<String, SkillRecord>) -> Result<()> {
    fs::create_dir_all(store)?;
    let text = serde_json::to_string_pretty(skills)?;
    fs::write(manifest_path(store), text)?;
    Ok(())
}

pub fn import_skill(cfg: &AppConfig, source_dir: &Path, id: Option<String>) -> Result<SkillRecord> {
    if !source_dir.exists() {
        return Err(anyhow!("skill 目录不存在: {}", source_dir.display()));
    }
    if !source_dir.join("SKILL.md").exists() {
        return Err(anyhow!("不是有效 skill，缺少 SKILL.md"));
    }

    let skill_id = id.unwrap_or_else(|| {
        source_dir
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase()
            .replace(' ', "-")
    });

    let dest_root = cfg.global_store.join("skills");
    fs::create_dir_all(&dest_root)?;
    let dest = dest_root.join(&skill_id);
    if dest.exists() {
        fs::remove_dir_all(&dest)?;
    }
    let mut opt = CopyOptions::new();
    opt.copy_inside = true;
    copy_dir(source_dir, &dest, &opt).context("复制 skill 失败")?;

    let record = SkillRecord {
        id: skill_id.clone(),
        name: skill_id.clone(),
        description: "imported skill".to_string(),
        version: "0.1.0".to_string(),
        tags: vec![],
        source: "local".to_string(),
        popularity: 0,
        updated_at: Utc::now(),
    };

    let mut manifest = load_manifest(&cfg.global_store)?;
    manifest.insert(skill_id, record.clone());
    save_manifest(&cfg.global_store, &manifest)?;
    Ok(record)
}

pub fn register_tool(
    cfg: &mut AppConfig,
    tool: &str,
    custom_path: Option<PathBuf>,
) -> Result<ToolTarget> {
    let path =
        custom_path.unwrap_or_else(|| default_tool_path(tool).unwrap_or_else(default_root_dir));
    let item = ToolTarget {
        tool: tool.to_string(),
        path,
        enabled: true,
    };
    cfg.tools.retain(|x| x.tool != tool);
    cfg.tools.push(item.clone());
    Ok(item)
}

pub fn sync_tools(cfg: &AppConfig) -> Result<Vec<String>> {
    let skills_root = cfg.global_store.join("skills");
    if !skills_root.exists() {
        return Ok(vec!["no skills to sync".to_string()]);
    }
    let mut logs = vec![];
    for tool in cfg.tools.iter().filter(|t| t.enabled) {
        fs::create_dir_all(&tool.path)?;
        for entry in fs::read_dir(&skills_root)? {
            let entry = entry?;
            let src = entry.path();
            if !src.is_dir() {
                continue;
            }
            let name = src.file_name().unwrap().to_string_lossy().to_string();
            let dst = tool.path.join(&name);
            if dst.exists() {
                fs::remove_dir_all(&dst)?;
            }
            let mut opt = CopyOptions::new();
            opt.copy_inside = true;
            copy_dir(&src, &dst, &opt)?;
            logs.push(format!("synced {name} -> {}", tool.tool));
        }
    }
    Ok(logs)
}

pub fn pull_sources(cfg: &AppConfig) -> Result<Vec<String>> {
    let mut logs = vec![];
    for s in &cfg.sources {
        if s.local_path.exists() {
            let output = std::process::Command::new("git")
                .arg("-C")
                .arg(&s.local_path)
                .arg("pull")
                .output()
                .context("git pull 执行失败")?;
            logs.push(format!(
                "{}: {}",
                s.name,
                String::from_utf8_lossy(&output.stdout).trim()
            ));
        } else {
            if let Some(parent) = s.local_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let output = std::process::Command::new("git")
                .arg("clone")
                .arg(&s.git_url)
                .arg(&s.local_path)
                .output()
                .context("git clone 执行失败")?;
            logs.push(format!(
                "{}: {}",
                s.name,
                String::from_utf8_lossy(&output.stdout).trim()
            ));
        }
    }
    Ok(logs)
}

pub fn search_skills(cfg: &AppConfig, keyword: &str) -> Result<Vec<String>> {
    let mut out = vec![];
    let manifest = load_manifest(&cfg.global_store)?;
    for s in manifest.values() {
        if s.id.contains(keyword) || s.name.contains(keyword) || s.description.contains(keyword) {
            out.push(format!("local:{} [{}]", s.name, s.version));
        }
    }

    for src in &cfg.sources {
        if !src.local_path.exists() {
            continue;
        }
        for ent in walkdir::WalkDir::new(&src.local_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file() && e.file_name() == "SKILL.md")
        {
            if ent
                .path()
                .to_string_lossy()
                .to_lowercase()
                .contains(&keyword.to_lowercase())
            {
                out.push(format!("source:{} -> {}", src.name, ent.path().display()));
            }
        }
    }
    Ok(out)
}

pub fn recommend(cfg: &AppConfig) -> Result<Vec<SkillRecord>> {
    let mut all: Vec<SkillRecord> = load_manifest(&cfg.global_store)?.into_values().collect();
    all.sort_by(|a, b| b.popularity.cmp(&a.popularity));
    Ok(all.into_iter().take(10).collect())
}

pub fn default_tool_path(tool: &str) -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let path = match tool.to_ascii_lowercase().as_str() {
        "cursor" => home.join(".cursor").join("skills"),
        "claude-code" => home.join(".claude").join("skills"),
        "codex" => home.join(".codex").join("skills"),
        "opencode" => home.join(".opencode").join("skills"),
        "antigravity" => home.join(".antigravity").join("skills"),
        "openclaw" => home.join(".openclaw").join("skills"),
        "kiro-cli" => home.join(".kiro").join("skills"),
        "trae" => home.join(".trae").join("skills"),
        "gemini-cli" => home.join(".gemini").join("skills"),
        "droid" => home.join(".droid").join("skills"),
        "windsurf" => home.join(".windsurf").join("skills"),
        _ => return None,
    };
    Some(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_paths_work() {
        assert!(default_tool_path("cursor").is_some());
        assert!(default_tool_path("unknown").is_none());
    }
}
