//! Cathedral ARKHE v28.3 — Curriculum
//! Task curriculum for progressive RL training.

pub struct Curriculum {
    tasks: Vec<String>,
    current_idx: usize,
}

impl Curriculum {
    pub fn new(tasks: Vec<String>) -> Self {
        Self {
            tasks,
            current_idx: 0,
        }
    }

    pub fn next_task(&mut self) -> Option<String> {
        if self.current_idx < self.tasks.len() {
            let task = self.tasks[self.current_idx].clone();
            self.current_idx += 1;
            Some(task)
        } else {
            None
        }
    }
}
