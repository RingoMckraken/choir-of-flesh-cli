
const HUMANITY_MAX_DEFAULT: u8 = 20;

pub struct Humanity {
    current: u8,
    max: u8,
}

impl Humanity {
    pub fn new() -> Humanity {
        Humanity {
            current: HUMANITY_MAX_DEFAULT,
            max: HUMANITY_MAX_DEFAULT,
        }
    }

    /// This constructor covers the Empathic skill which raises maximum humanity to 22
    pub fn new_with_max(max: u8) -> Humanity {
        Humanity {
            current: max,
            max: max,
        }
    }

    pub fn current(&self) -> u8 {
        self.current
    }

    pub fn max(&self) -> u8 {
        self.max
    }

    pub fn lose(&mut self, amount: u8) {
        self.current = self.current.saturating_sub(amount);
    }

    /// Adds humanity without going over the maximum value
    pub fn regain(&mut self, amount: u8) {
        self.current = self.current.saturating_add(amount).min(self.max);
    }

    pub fn is_lost(&self) -> bool {
        self.current == 0
    }
}