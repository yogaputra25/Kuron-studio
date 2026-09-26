use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::State;
use uuid::Uuid;

use crate::AppState;

const IMAGE_EXTS: [&str; 4] = ["jpg", "jpeg", "png", "webp"];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Page {
    pub id: String,
    pub path: String,
    pub width: u32,
    pub height: u32,
    pub status: PageStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub enum PageStatus {
    #[default]
    #[serde(rename = "idle")]
    Idle,
    #[serde(rename = "detecting")]
    Detecting,
    #[serde(rename = "detected")]
    Detected,
    #[serde(rename = "noBubbles")]
    NoBubbles,
    #[serde(rename = "translating")]
    Translating,
    #[serde(rename = "translated")]
    Translated,
    #[serde(rename = "failed")]
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    pub pages: Vec<Page>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub pages: Vec<Page>,
    pub skipped: usize,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectStore {
    #[serde(skip)]
    dir: PathBuf,
    pub projects: HashMap<String, Project>,
}

impl ProjectStore {
    pub fn load(dir: PathBuf) -> Result<Self, String> {
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let file = dir.join("projects.json");
        if !file.exists() {
            return Ok(Self {
                dir,
                projects: HashMap::new(),
            });
        }
        let raw = fs::read_to_string(&file).map_err(|e| e.to_string())?;
        let mut store: ProjectStore =
            serde_json::from_str(&raw).map_err(|e| format!("projects.json corrupt: {e}"))?;
        store.dir = dir;
        Ok(store)
    }

    fn save(&self) -> Result<(), String> {
        let raw = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(self.dir.join("projects.json"), raw).map_err(|e| e.to_string())
    }

    fn get_mut(&mut self, project_id: &str) -> Result<&mut Project, String> {
        self.projects
            .get_mut(project_id)
            .ok_or_else(|| format!("project not found: {project_id}"))
    }
}

pub fn is_image_file(path: &Path) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => IMAGE_EXTS.contains(&ext.to_ascii_lowercase().as_str()),
        None => false,
    }
}

fn is_archive(path: &Path) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => matches!(ext.to_ascii_lowercase().as_str(), "zip" | "cbz"),
        None => false,
    }
}

fn page_from_file(path: &Path) -> Page {
    let (width, height) = image::image_dimensions(path).unwrap_or((0, 0));
    Page {
        id: Uuid::new_v4().to_string(),
        path: path.to_string_lossy().to_string(),
        width,
        height,
        status: PageStatus::Idle,
    }
}

/// Expand one dropped/selected path into image files.
/// Dir -> sorted image children. zip/cbz -> extracted under store dir, then sorted.
/// Single image -> itself. Anything else counts as skipped.
fn expand_input(path: &Path, extract_dir: &Path) -> Result<(Vec<PathBuf>, usize), String> {
    if path.is_dir() {
        let mut files: Vec<PathBuf> = fs::read_dir(path)
            .map_err(|e| e.to_string())?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.is_file())
            .collect();
        let total = files.len();
        files.retain(|p| is_image_file(p));
        let skipped = total - files.len();
        files.sort();
        return Ok((files, skipped));
    }
    if path.is_file() {
        if is_archive(path) {
            return extract_archive(path, extract_dir);
        }
        if is_image_file(path) {
            return Ok((vec![path.to_path_buf()], 0));
        }
        return Ok((vec![], 1));
    }
    Err(format!("path not found: {}", path.display()))
}

fn extract_archive(path: &Path, extract_dir: &Path) -> Result<(Vec<PathBuf>, usize), String> {
    let file = fs::File::open(path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    fs::create_dir_all(extract_dir).map_err(|e| e.to_string())?;
    let mut files = Vec::new();
    let mut skipped = 0;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = entry.name().to_string();
        let out = extract_dir.join(&name);
        if entry.is_dir() {
            continue;
        }
        if !is_image_file(Path::new(&name)) {
            skipped += 1;
            continue;
        }
        if let Some(parent) = out.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut out_file = fs::File::create(&out).map_err(|e| e.to_string())?;
        std::io::copy(&mut entry, &mut out_file).map_err(|e| e.to_string())?;
        files.push(out);
    }
    files.sort();
    Ok((files, skipped))
}

#[tauri::command]
pub fn list_projects(state: State<'_, AppState>) -> Result<Vec<Project>, String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    let mut projects: Vec<Project> = store.projects.values().cloned().collect();
    projects.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(projects)
}

#[tauri::command]
pub fn create_project(name: String, state: State<'_, AppState>) -> Result<Project, String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("project name is empty".to_string());
    }
    let mut store = state.store.lock().map_err(|e| e.to_string())?;
    let project = Project {
        id: Uuid::new_v4().to_string(),
        name,
        pages: Vec::new(),
    };
    store.projects.insert(project.id.clone(), project.clone());
    store.save()?;
    Ok(project)
}

#[tauri::command]
pub fn get_project(project_id: String, state: State<'_, AppState>) -> Result<Project, String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    store
        .projects
        .get(&project_id)
        .cloned()
        .ok_or_else(|| format!("project not found: {project_id}"))
}

#[tauri::command]
pub fn import_pages(
    project_id: String,
    paths: Vec<String>,
    state: State<'_, AppState>,
) -> Result<ImportResult, String> {
    let mut store = state.store.lock().map_err(|e| e.to_string())?;
    let extract_dir = store.dir.join(format!("{project_id}_extracted"));
    let mut all_files: Vec<PathBuf> = Vec::new();
    let mut skipped = 0;
    for raw in &paths {
        let (mut files, skip) = expand_input(Path::new(raw), &extract_dir)?;
        all_files.append(&mut files);
        skipped += skip;
    }
    let pages: Vec<Page> = all_files.iter().map(|p| page_from_file(p)).collect();
    let project = store.get_mut(&project_id)?;
    project.pages.extend(pages.clone());
    project.pages.sort_by(|a, b| a.path.cmp(&b.path));
    store.save()?;
    Ok(ImportResult { pages, skipped })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_ext_case_insensitive() {
        assert!(is_image_file(Path::new("p.JPG")));
        assert!(is_image_file(Path::new("p.webp")));
        assert!(!is_image_file(Path::new("p.txt")));
        assert!(!is_image_file(Path::new("noext")));
    }

    #[test]
    fn archive_detected() {
        assert!(is_archive(Path::new("ch.zip")));
        assert!(is_archive(Path::new("ch.CBZ")));
        assert!(!is_archive(Path::new("p.png")));
    }

    #[test]
    fn missing_path_errors() {
        let dir = std::env::temp_dir().join("kuron-studio-missing-probe");
        let res = expand_input(&dir.join("nope"), &dir);
        assert!(res.is_err());
    }
}
