pub struct Random {
    state: u64,
}

impl Random {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn next(&mut self) -> u64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }

    pub fn range(&mut self, min: u64, max: u64) -> u64 {
        let range = max - min;
        min + (self.next() % range)
    }
}
