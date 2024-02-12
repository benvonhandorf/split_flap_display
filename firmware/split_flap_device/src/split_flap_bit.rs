

pub struct SplitFlapBit {
    steps_per_flap: u32,
    homing_offset_steps: u32,
    steps_since_home: u32,
    target_steps: u32,
}

impl SplitFlapBit {
    pub fn new(steps_per_flap: u32, homing_offset_steps: u32) -> SplitFlapBit {
        SplitFlapBit { 
            steps_per_flap: steps_per_flap, 
            homing_offset_steps: homing_offset_steps, 
            steps_since_home: 0,
            target_steps: 0,
        }
    }

    pub fn set_homed(&mut self, is_homed: bool) {
        self.steps_since_home = 0;
    }

    fn lookup_target_character_steps(&self, target_character: u8) -> u32 {
        1
    }

    pub fn set_target_character(&mut self, target_character: u8) {
        self.target_steps = self.lookup_target_character_steps(target_character);
    }

    pub fn process(&mut self) -> bool {
        !(self.steps_since_home == self.target_steps)
    }
}

mod test {
    #[test]
    fn new_initializes_steps_since_home() {
        let result = super::SplitFlapBit::new(10, 10);

        assert_eq!(result.steps_since_home, 0)
    }
}