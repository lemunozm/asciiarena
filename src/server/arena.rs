pub struct Arena {
    id: usize,
}

impl Arena {
    pub fn new(id: usize) -> Arena {
        Arena { id }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn step(&mut self) {
        //TODO
    }

    pub fn has_finished(&self) -> bool {
        true //TODO
    }
}
