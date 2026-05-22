use crate::data;

#[derive(Debug, Clone)]
pub struct Project {
    pub name: String,
    pub minutes: u32,
    pub marked: bool,
}

#[derive(Debug)]
pub struct AppState {
    pub projects: Vec<Project>,
    pub active_index: Option<usize>,
    pub marked_index: Option<usize>,
    pub font_size: i32,
    pub paused: bool,
    pub last_tick: std::time::Instant,
}

impl AppState {
    pub fn new(font_size: i32) -> Self {
        Self {
            projects: Vec::new(),
            active_index: None,
            marked_index: None,
            font_size,
            paused: true,
            last_tick: std::time::Instant::now(),
        }
    }

    pub fn load_today(&mut self) {
        let path = data::today_file();
        self.projects = data::read_dayfile(&path);
    }

    pub fn save(&self) {
        let path = data::today_file();
        let _ = data::ensure_data_dir();
        let _ = data::write_dayfile(&path, &self.projects);
    }

    /// Advance the active project's time counter by elapsed minutes.
    /// Elapsed time greater than 2 minutes is discarded to avoid counting
    /// system suspend time.
    pub fn tick(&mut self) {
        if self.paused {
            self.last_tick = std::time::Instant::now();
            return;
        }
        let elapsed = self.last_tick.elapsed();
        self.last_tick = std::time::Instant::now();
        let secs = elapsed.as_secs();
        // Guard: ignore gaps > 2 minutes (e.g. system suspend)
        if secs < 120 {
            if let Some(idx) = self.active_index {
                if idx < self.projects.len() {
                    self.projects[idx].minutes += (secs / 60) as u32;
                }
            }
        }
    }

    pub fn select_project(&mut self, index: usize) {
        if index < self.projects.len() {
            self.active_index = Some(index);
            self.paused = false;
            self.last_tick = std::time::Instant::now();
        }
    }

    pub fn deselect(&mut self) {
        self.active_index = None;
        self.paused = true;
    }

    pub fn add_project(&mut self, name: String) {
        if !name.is_empty() && !self.projects.iter().any(|p| p.name == name) {
            self.projects.push(Project {
                name,
                minutes: 0,
                marked: false,
            });
        }
    }

    pub fn delete_project(&mut self, index: usize) {
        if index < self.projects.len() {
            if self.active_index == Some(index) {
                self.active_index = None;
                self.paused = true;
            }
            if self.marked_index == Some(index) {
                self.marked_index = None;
            }
            self.projects.remove(index);
            // Adjust indices after removal
            if let Some(ai) = self.active_index {
                if ai > index {
                    self.active_index = Some(ai - 1);
                }
            }
            if let Some(mi) = self.marked_index {
                if mi > index {
                    self.marked_index = Some(mi - 1);
                }
            }
        }
    }

    /// Toggle the mark on a project. Only one project can be marked at a time.
    pub fn mark_source(&mut self, index: usize) {
        if index < self.projects.len() {
            if self.marked_index == Some(index) {
                self.projects[index].marked = false;
                self.marked_index = None;
            } else {
                if let Some(mi) = self.marked_index {
                    if mi < self.projects.len() {
                        self.projects[mi].marked = false;
                    }
                }
                self.projects[index].marked = true;
                self.marked_index = Some(index);
            }
        }
    }

    pub fn transfer_minutes(&mut self, from: usize, to: usize, minutes: u32) {
        if from < self.projects.len() && to < self.projects.len() && from != to {
            let available = self.projects[from].minutes;
            let transfer = minutes.min(available);
            self.projects[from].minutes -= transfer;
            self.projects[to].minutes += transfer;
        }
    }

    pub fn set_time(&mut self, index: usize, minutes: u32) {
        if index < self.projects.len() {
            self.projects[index].minutes = minutes;
        }
    }

    pub fn move_project(&mut self, from: usize, to: usize) {
        if from < self.projects.len() && to < self.projects.len() && from != to {
            let project = self.projects.remove(from);
            self.projects.insert(to, project);
            self.active_index = adjust_index(self.active_index, from, to);
            self.marked_index = adjust_index(self.marked_index, from, to);
        }
    }
}

fn adjust_index(idx: Option<usize>, from: usize, to: usize) -> Option<usize> {
    idx.map(|i| {
        if i == from {
            to
        } else if from < to && i > from && i <= to {
            i - 1
        } else if from > to && i >= to && i < from {
            i + 1
        } else {
            i
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_project_no_duplicates() {
        let mut state = AppState::new(12);
        state.add_project("Alpha".to_string());
        state.add_project("Alpha".to_string());
        assert_eq!(state.projects.len(), 1);
    }

    #[test]
    fn test_delete_adjusts_active_index() {
        let mut state = AppState::new(12);
        state.add_project("A".to_string());
        state.add_project("B".to_string());
        state.add_project("C".to_string());
        state.select_project(2); // C is active
        state.delete_project(0); // delete A
        assert_eq!(state.active_index, Some(1)); // C is now at index 1
    }

    #[test]
    fn test_transfer_minutes_capped_at_available() {
        let mut state = AppState::new(12);
        state.add_project("A".to_string());
        state.add_project("B".to_string());
        state.projects[0].minutes = 10;
        state.transfer_minutes(0, 1, 999);
        assert_eq!(state.projects[0].minutes, 0);
        assert_eq!(state.projects[1].minutes, 10);
    }

    #[test]
    fn test_mark_source_toggle() {
        let mut state = AppState::new(12);
        state.add_project("A".to_string());
        state.mark_source(0);
        assert_eq!(state.marked_index, Some(0));
        state.mark_source(0);
        assert_eq!(state.marked_index, None);
    }
}
