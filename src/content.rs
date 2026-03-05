use std::{fs, error::Error, path::{Path, PathBuf}};
use walkdir::WalkDir;
use crate::models::Lesson;

pub fn load_lessons() -> Result<Vec<Lesson>, Box<dyn Error>> {
    let mut lessons = Vec::new();
    let book_src_dir = Path::new("./book");

    if !book_src_dir.exists() {
        return Ok(lessons);
    }
    
    // Read SUMMARY.md for order
    let summary_path = book_src_dir.join("SUMMARY.md");
    if !summary_path.exists() {
        // Fallback to unstructured dump
        return fallback_load(book_src_dir);
    }
    
    let summary_content = fs::read_to_string(&summary_path)?;
    let mut expected_paths = Vec::new();
    
    // Parse [](my_file.md) from summary
    for line in summary_content.lines() {
        if let Some(start) = line.find("](") {
            let rest = &line[start + 2..];
            if let Some(end) = rest.find(')') {
                let md_file = &rest[..end];
                if md_file.ends_with(".md") {
                    expected_paths.push(md_file.to_string());
                }
            }
        }
    }
    
    for relative_path in expected_paths {
        let path = book_src_dir.join(&relative_path);
        if path.exists() && path.is_file() {
            if let Ok(content) = fs::read_to_string(&path) {
                let file_stem = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
                let title = extract_title(&content).unwrap_or(file_stem);
                
                let text_lines = crate::markdown::parse_to_lines(&content);

                lessons.push(Lesson {
                    path: path.to_string_lossy().to_string(), // use absolute or relative path consistently
                    title,
                    content,
                    text_lines,
                });
            }
        }
    }

    Ok(lessons)
}

fn fallback_load(book_src_dir: &Path) -> Result<Vec<Lesson>, Box<dyn Error>> {
    let mut lessons = Vec::new();
    
    for entry in WalkDir::new(book_src_dir).sort_by_file_name() {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().unwrap_or_default() == "md" {
            let content = fs::read_to_string(path)?;
            
            let file_stem = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
            let title = extract_title(&content).unwrap_or(file_stem);
            
            let text_lines = crate::markdown::parse_to_lines(&content);

            lessons.push(Lesson {
                path: path.to_string_lossy().to_string(),
                title,
                content,
                text_lines,
            });
        }
    }

    Ok(lessons)
}

fn extract_title(content: &str) -> Option<String> {
    for line in content.lines() {
        if line.starts_with("# ") {
            return Some(line[2..].trim().to_string());
        }
    }
    None
}
