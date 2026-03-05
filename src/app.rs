use std::{error::Error, path::PathBuf};
use crate::{models::{Lesson, Progress}, store, content};

#[derive(Debug, Default)]
pub enum Mode {
    #[default]
    List,
    Content,
}

pub struct App {
    /// Is the application running?
    pub running: bool,
    pub mode: Mode,
    pub lessons: Vec<Lesson>,
    pub selected_lesson_index: usize,
    pub progress: Progress,
    pub scroll_offset: u16,
}

impl App {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        // Load lessons first
        let lessons = content::load_lessons()?;
        // Load progress
        let progress = store::load_progress()?;
        
        let start_index = progress.last_index.min(lessons.len().saturating_sub(1));

        Ok(Self {
            running: true,
            mode: Mode::List,
            lessons,
            selected_lesson_index: start_index,
            progress,
            scroll_offset: 0,
        })
    }

    pub fn tick(&self) {}

    pub fn quit(&mut self) {
        self.running = false;
        self.progress.last_index = self.selected_lesson_index;
        if let Err(e) = store::save_progress(&self.progress) {
            eprintln!("Failed to save progress: {}", e);
        }
    }

    pub fn next(&mut self) {
        if self.lessons.is_empty() { return; }
        if self.selected_lesson_index < self.lessons.len() - 1 {
            self.selected_lesson_index += 1;
            self.scroll_offset = 0; // Reset scroll on change
            self.check_auto_complete();
        }
    }

    pub fn previous(&mut self) {
        if self.lessons.is_empty() { return; }
        if self.selected_lesson_index > 0 {
            self.selected_lesson_index -= 1;
            self.scroll_offset = 0; // Reset scroll on change
            self.check_auto_complete();
        }
    }
    
    pub fn toggle_mode(&mut self) {
        self.mode = match self.mode {
            Mode::List => Mode::Content,
            Mode::Content => Mode::List,
        };
    }
    
    pub fn scroll_down(&mut self) {
        if matches!(self.mode, Mode::Content) {
            self.scroll_offset = self.scroll_offset.saturating_add(1);
            
            // Auto completion check
            if let Some(lesson) = self.lessons.get(self.selected_lesson_index) {
                if self.scroll_offset as usize + 20 >= lesson.text_lines.len() {
                    self.mark_completed();
                }
            }
        }
    }

    pub fn scroll_up(&mut self) {
        if matches!(self.mode, Mode::Content) {
            self.scroll_offset = self.scroll_offset.saturating_sub(1);
        }
    }

    pub fn mark_completed(&mut self) {
        if self.lessons.is_empty() { return; }
        let current_path = &self.lessons[self.selected_lesson_index].path;
        if !self.progress.completed_files.contains(current_path) {
            self.progress.completed_files.push(current_path.clone());
        }
    }
    
    fn check_auto_complete(&mut self) {
        if let Some(lesson) = self.lessons.get(self.selected_lesson_index) {
            if lesson.text_lines.len() < 30 {
                self.mark_completed();
            }
        }
    }
}
