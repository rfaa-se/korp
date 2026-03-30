pub struct Random {
    state: u64,
}

impl Random {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn next(&mut self) -> u64 {
        let mut x = self.state;

        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;

        self.state = x;

        x
    }

    pub fn range(&mut self, min: u64, max: u64) -> u64 {
        let range = max - min;
        min + (self.next() % range)
    }
}
