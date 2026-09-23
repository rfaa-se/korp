pub struct Random {
    state: u64,
}

impl Random {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }

    pub fn range_u64(&mut self, min: u64, max: u64) -> u64 {
        let range = max - min;
        min + (self.next_u64() % range)
    }

    pub fn range_u16(&mut self, min: u16, max: u16) -> u16 {
        self.range_u64(min as u64, max as u64) as u16
    }

    pub fn range_usize(&mut self, min: usize, max: usize) -> usize {
        self.range_u64(min as u64, max as u64) as usize
    }
}
