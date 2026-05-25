use crate::data;

#[derive(Debug, Clone)]
pub struct Project {
    pub name: String,
    pub minutes: u32,
}

#[derive(Debug)]
pub struct AppState {
    pub projects: Vec<Project>,
    pub active_index: Option<usize>,
    pub current_day: String,
    pub font_size: i32,
    pub total_minutes: u32,
    pub adjusted_minutes: i32,
    pub paused: bool,
    pub last_tick: std::time::Instant,
}

impl AppState {
    pub fn new(font_size: i32) -> Self {
        Self {
            projects: Vec::new(),
            active_index: None,
            current_day: data::today_string(),
            font_size,
            total_minutes: 0,
            adjusted_minutes: 0,
            paused: true,
            last_tick: std::time::Instant::now(),
        }
    }

    pub fn load_today(&mut self) {
        self.current_day = data::today_string();

        // 1. Read the master project list for ordered project names.
        let pl_names = data::read_projectlist(&data::projectlist_file());

        // 2. Read today's day file for time data.
        let today_projects = data::read_dayfile(&data::today_file());

        // 3. Read the most recent previous day file for project names that
        //    may be missing from projectlist (e.g. when projectlist is stale
        //    or was never written by the old titrax).
        let recent_projects = data::find_most_recent_dayfile()
            .map(|p| data::read_dayfile(&p))
            .unwrap_or_default();

        // 4. Build a merged, ordered name list:
        //    projectlist order first, then any names from the most recent day
        //    file, then any names from today's day file not yet seen.
        let mut names: Vec<String> = pl_names;
        for p in recent_projects.iter().chain(today_projects.iter()) {
            if !names.iter().any(|n| n == &p.name) {
                names.push(p.name.clone());
            }
        }

        // 5. Build projects with today's times overlaid.
        self.projects = names
            .into_iter()
            .map(|name| {
                let minutes = today_projects
                    .iter()
                    .find(|p| p.name == name)
                    .map(|p| p.minutes)
                    .unwrap_or(0);
                Project { name, minutes }
            })
            .collect();
        self.total_minutes = self.projects.iter().map(|p| p.minutes).sum();
        self.adjusted_minutes = 0;
    }

    /// When the wall date changes, flush previous day and start a fresh day
    /// while keeping project names/order and active selection.
    pub fn rollover_if_new_day(&mut self) {
        let today = data::today_string();
        if today == self.current_day {
            return;
        }

        let _ = data::ensure_data_dir();
        let _ = data::write_dayfile(&data::day_file_for(&self.current_day), &self.projects);

        for project in &mut self.projects {
            project.minutes = 0;
        }
        self.total_minutes = 0;
        self.adjusted_minutes = 0;
        self.current_day = today;
    }

    /// Save time data to today's day file only.
    /// Called frequently: auto-save timer, on close, on time edits/transfers.
    /// Does NOT touch projectlist.
    pub fn save_times(&self) {
        let _ = data::ensure_data_dir();
        let _ = data::write_dayfile(&data::today_file(), &self.projects);
    }

    /// Save project structure (names, order) to projectlist, then save times.
    /// Called only when project structure changes: add, delete, reorder, sort.
    pub fn save_projects(&self) {
        let _ = data::ensure_data_dir();
        let _ = data::write_projectlist(&data::projectlist_file(), &self.projects);
        let _ = data::write_dayfile(&data::today_file(), &self.projects);
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
                    let inc = (secs / 60) as u32;
                    self.projects[idx].minutes += inc;
                    self.total_minutes += inc;
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
            });
        }
    }

    pub fn delete_project(&mut self, index: usize) {
        if index < self.projects.len() {
            if self.active_index == Some(index) {
                self.active_index = None;
                self.paused = true;
            }
            let removed = self.projects.remove(index);
            self.total_minutes = self.total_minutes.saturating_sub(removed.minutes);
            self.adjusted_minutes -= removed.minutes as i32;
            // Adjust indices after removal
            if let Some(ai) = self.active_index {
                if ai > index {
                    self.active_index = Some(ai - 1);
                }
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
            let old = self.projects[index].minutes;
            self.projects[index].minutes = minutes;
            if minutes >= old {
                let delta = minutes - old;
                self.total_minutes += delta;
                self.adjusted_minutes += delta as i32;
            } else {
                let delta = old - minutes;
                self.total_minutes = self.total_minutes.saturating_sub(delta);
                self.adjusted_minutes -= delta as i32;
            }
        }
    }

    pub fn increment_minutes(&mut self, index: usize, delta: i32) {
        if index >= self.projects.len() {
            return;
        }
        let old = self.projects[index].minutes as i32;
        let new = (old + delta).max(0) as u32;
        self.projects[index].minutes = new;
        if new as i32 >= old {
            let inc = (new as i32 - old) as u32;
            self.total_minutes += inc;
            self.adjusted_minutes += inc as i32;
        } else {
            let dec = (old - new as i32) as u32;
            self.total_minutes = self.total_minutes.saturating_sub(dec);
            self.adjusted_minutes -= dec as i32;
        }
    }

    pub fn move_project(&mut self, from: usize, to: usize) {
        if from < self.projects.len() && to < self.projects.len() && from != to {
            let project = self.projects.remove(from);
            self.projects.insert(to, project);
            self.active_index = adjust_index(self.active_index, from, to);
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
}
