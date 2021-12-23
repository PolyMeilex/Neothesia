pub struct Carousel<T> {
    outputs: Vec<T>,
    id: usize,
}

impl<T> Carousel<T> {
    pub fn new() -> Self {
        Self {
            outputs: Vec::new(),
            id: 0,
        }
    }

    pub fn select(&mut self, id: usize) {
        self.outputs.get(id).unwrap();
        self.id = id;
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn update(&mut self, outs: Vec<T>) {
        self.outputs = outs;
    }

    pub fn check_next(&self) -> bool {
        self.id < self.outputs.len() - 1
    }

    pub fn check_prev(&self) -> bool {
        self.id > 0
    }

    pub fn next(&mut self) {
        if self.check_next() {
            self.id += 1;
        } else {
            self.id = 0;
        }
    }

    pub fn prev(&mut self) {
        if self.check_prev() {
            self.id -= 1;
        } else {
            self.id = self.outputs.len() - 1;
        }
    }

    pub fn get_item(&self) -> Option<&T> {
        self.outputs.get(self.id)
    }
}
