extern crate ultraviolet as glm;

pub mod display;

pub struct ImguiId {
    ids: Vec<String>,
    curr_index: usize,
}

impl ImguiId {
    pub fn new(start: u32) -> Self {
        let mut ids = vec![];
        let curr_index = 0;
        for i in 0..start {
            ids.push(i.to_string());
        }

        Self { ids, curr_index }
    }

    pub fn get_next_id(&mut self) -> &str {
        let len = self.curr_index;

        if self.ids.len() > self.curr_index as usize {
        } else {
            self.ids.push((self.curr_index).to_string());
        }

        self.curr_index += 1;
        self.ids[len].as_str()
    }

    pub fn start_frame(&mut self) {
        self.curr_index = 0;
    }
}
