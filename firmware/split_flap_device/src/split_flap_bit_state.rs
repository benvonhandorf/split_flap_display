

const CHARACTER_SET: [u8;55] = [
    b'A',
    b'B',
    b'C',
    b'D',
    b'E',
    b'F',
    b'G',
    b'H',
    b'I',
    b'J',
    b'K',
    b'L',
    b'M',
    b'N',
    b'O',
    b'P',
    b'Q',
    b'R',
    b'S',
    b'T',
    b'U',
    b'V',
    b'W',
    b'X',
    b'Y',
    b'Z',
    b'0',
    b'1',
    b'2',
    b'3',
    b'4',
    b'5',
    b'6',
    b'7',
    b'8',
    b'9',
    b':',
    b'-',
    b'_',
    b'.',
    b'%',
    b'@',
    b'/',
    b' ',
    0x01,
    0x02,
    0x03,
    0x04,
    0x05,
    0x06,
    0x07,
    0x08,
    0x09,
    0x0A,
    0x0B,
];

pub struct SplitFlapBitState {
    steps_per_flap: u32,
    homing_offset_steps: u32,
    steps_since_home: u32,
    target_steps: u32,
}

impl SplitFlapBitState {
    pub fn new(steps_per_flap: u32, homing_offset_steps: u32) -> SplitFlapBitState {
        SplitFlapBitState { 
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
        let position = CHARACTER_SET.iter().position(|&c| c == target_character);


    }

    pub fn set_target_character(&mut self, target_character: u8) {
        self.target_steps = self.lookup_target_character_steps(target_character);
    }

    pub fn process(&mut self) -> bool {
        let needs_step = !(self.steps_since_home == self.target_steps);

        if needs_step {
            self.steps_since_home += 1;
        }

        needs_step
    }

}

mod test {

    #[test]
    fn new_initializes_steps_since_home() {
        let result = super::SplitFlapBitState::new(10, 10);

        assert_eq!(result.steps_since_home, 0)
    }

    #[test]
    fn new_initializes_target_steps() {
        let result = super::SplitFlapBitState::new(10, 10);

        assert_eq!(result.steps_since_home, 0)
    }
}