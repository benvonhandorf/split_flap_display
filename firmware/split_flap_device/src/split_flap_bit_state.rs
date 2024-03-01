// use std::convert::From;
use core::cmp::PartialEq;
use core::ops::{self, Add, Mul, Rem};

const CHARACTER_SET: [u8; 55] = [
    b' ', b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H', b'I', b'J', b'K', b'L', b'M', b'N', b'O',
    b'P', b'Q', b'R', b'S', b'T', b'U', b'V', b'W', b'X', b'Y', b'Z', b'0', b'1', b'2', b'3', b'4',
    b'5', b'6', b'7', b'8', b'9', b':', b'-', b'_', b'.', b'%', b'@', b'/', 0x01, 0x02, 0x03, 0x04,
    0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B,
];

#[derive(Clone, Copy)]
struct Steps {
    steps: u32,
}

impl Mul<u32> for Steps {
    type Output = Self;
    fn mul(self, rhs: u32) -> Self::Output {
        Steps {
            steps: self.steps * rhs,
        }
    }
}

impl Mul<usize> for Steps {
    type Output = Self;
    fn mul(self, rhs: usize) -> Self::Output {
        self.mul(rhs as u32)
    }
}

#[derive(Clone, Copy)]
struct HomedSteps {
    homed_steps: u32,
}

impl Add<Steps> for HomedSteps {
    type Output = Self;
    fn add(self, rhs: Steps) -> Self::Output {
        HomedSteps {
            homed_steps: self.homed_steps + rhs.steps,
        }
    }
}

impl Rem<Steps> for HomedSteps {
    type Output = Self;
    fn rem(self, rhs: Steps) -> Self::Output {
        HomedSteps {
            homed_steps: self.homed_steps % rhs.steps,
        }
    }
}


impl PartialEq<HomedSteps> for HomedSteps {
    fn eq(&self, other: &HomedSteps) -> bool {
        self.homed_steps == other.homed_steps
    }
}

impl HomedSteps {
    fn from_offset(homing_offset: u32) -> HomedSteps {
        HomedSteps {
            homed_steps: homing_offset,
        }
    }

    fn from(steps: Steps, homed_steps_offset: HomedSteps) -> HomedSteps {
        HomedSteps {
            homed_steps: steps.steps + homed_steps_offset.homed_steps,
        }
    }

    fn empty() -> HomedSteps {
        HomedSteps { homed_steps: 0 }
    }

    fn clear(&mut self) {
        self.homed_steps = 0;
    }

    fn inc(&mut self) {
        self.homed_steps += 1;
    }
}

#[derive(PartialEq)]
pub enum BitState {
    UNINITIALIZED,
    SEEKING,
    SETTLED,
}

#[derive(Clone, Copy)]
pub struct SensorCalibration {
    pub trigger_value: u32,
    pub untrigger_value: u32,
}

#[derive(PartialEq)]
enum SensorState {
    Triggered,
    Untriggered,
}

pub struct SplitFlapBitState {
    sensor_calibration: SensorCalibration,
    bit_state: BitState,
    steps_per_flap: Steps,
    offset_steps_to_first_position: HomedSteps,
    steps_since_home: HomedSteps,
    target_steps: HomedSteps,
    sensor_state: SensorState,
}

impl SplitFlapBitState {
    pub fn new(
        sensor_calibration: SensorCalibration,
        steps_per_flap: u32,
        offset_steps_to_first_position: u32,
    ) -> SplitFlapBitState {
        SplitFlapBitState {
            sensor_calibration: sensor_calibration,
            bit_state: BitState::UNINITIALIZED,
            steps_per_flap: Steps {
                steps: steps_per_flap,
            },
            offset_steps_to_first_position: HomedSteps::from_offset(offset_steps_to_first_position),
            steps_since_home: HomedSteps::empty(),
            target_steps: HomedSteps::empty(),
            sensor_state: SensorState::Untriggered,
        }
    }

    pub fn set_homed(&mut self, is_homed: bool) {
        // self.steps_since_home = 0;
    }

    fn lookup_target_character_position(&self, target_character_code: u8) -> u32 {
        let position = CHARACTER_SET
            .iter()
            .position(|&c| c == target_character_code)
            .unwrap_or(0);

        position as u32
    }

    fn lookup_target_character_steps(&self, target_character_code: u8) -> HomedSteps {
        let target_position = self.lookup_target_character_position(target_character_code);

       (self.offset_steps_to_first_position + (self.steps_per_flap * target_position)) % (self.steps_per_flap * CHARACTER_SET.len())
    }

    pub fn set_target_character(&mut self, target_character: u8) {
        self.target_steps = self.lookup_target_character_steps(target_character);
    }

    fn process_sensor(&mut self, sensor_value: u32) {
        if sensor_value > self.sensor_calibration.trigger_value {
            if self.sensor_state == SensorState::Untriggered {
                self.sensor_state = SensorState::Triggered;
                self.steps_since_home.clear();

                if self.bit_state == BitState::UNINITIALIZED {
                    //First homing of the bit.  Set us up to seek to the home position.
                    self.target_steps = self.offset_steps_to_first_position;
                    self.bit_state = BitState::SEEKING;
                }
            }
        } else if sensor_value < self.sensor_calibration.untrigger_value {
            self.sensor_state = SensorState::Untriggered;
        }
    }

    pub fn process(&mut self, sensor_value: u32) -> bool {
        self.process_sensor(sensor_value);

        if self.bit_state == BitState::UNINITIALIZED {
            return true;
        }

        let needs_step = !(self.steps_since_home == self.target_steps);

        if needs_step {
            self.bit_state = BitState::SEEKING;
            //Assume the step will be taken
            self.steps_since_home.inc();
        } else {
            self.bit_state = BitState::SETTLED;
        }

        needs_step
    }
}

mod test {
    use crate::split_flap_bit_state::{BitState, HomedSteps};

    #[test]
    fn new_starts_uninitialized() {
        let calibration = super::SensorCalibration {
            trigger_value: 2000,
            untrigger_value: 1800,
        };
        let result = super::SplitFlapBitState::new(calibration, 58, 58 * 6);

        assert_eq!(result.bit_state as u32, BitState::UNINITIALIZED as u32)
    }

    #[test]
    fn new_initializes_steps_since_home() {
        let calibration = super::SensorCalibration {
            trigger_value: 2000,
            untrigger_value: 1800,
        };
        let result = super::SplitFlapBitState::new(calibration, 58, 58 * 6);

        assert!(
            result.steps_since_home == HomedSteps::empty(),
            "steps_since_home is not empty"
        )
    }

    #[test]
    fn new_initializes_target_steps() {
        let calibration = super::SensorCalibration {
            trigger_value: 2000,
            untrigger_value: 1800,
        };
        let result = super::SplitFlapBitState::new(calibration, 58, 58 * 6);

        assert!(
            result.target_steps == HomedSteps::empty(),
            "target_steps is not empty"
        )
    }

    #[test]
    fn passing_sensor_value_above_trigger_sets_status() {
        let calibration = super::SensorCalibration {
            trigger_value: 2000,
            untrigger_value: 1800,
        };
        let mut result = super::SplitFlapBitState::new(calibration, 58, 58 * 6);

        let sensor_value: u32 = 2100;

        result.process(sensor_value);

        assert_eq!(result.bit_state as u32, BitState::SEEKING as u32);
        assert_eq!(result.target_steps.homed_steps, 58 * 6);
    }

    #[test]
    fn passing_sensor_value_above_trigger_again_does_not_reset_steps_since_home() {
        let calibration = super::SensorCalibration {
            trigger_value: 2000,
            untrigger_value: 1800,
        };
        let mut result = super::SplitFlapBitState::new(calibration, 58, 58 * 6);

        let sensor_value: u32 = 2100;

        result.process(sensor_value);
        result.process(sensor_value);

        assert_eq!(result.steps_since_home.homed_steps, 2);
    }

    #[test]
    fn process_returns_false_after_offset_reached() {
        let calibration = super::SensorCalibration {
            trigger_value: 2000,
            untrigger_value: 1800,
        };
        let mut result = super::SplitFlapBitState::new(calibration, 58, 3);

        let sensor_value: u32 = 2100;

        let process = result.process(sensor_value);
        assert!(process, "Target not yet reached, process returned false");

        let process = result.process(sensor_value);
        assert!(process, "Target not yet reached, process returned false");

        let process = result.process(sensor_value);
        assert!(process, "Target not yet reached, process returned false");

        let process = result.process(sensor_value);
        assert!(!process, "Target reached, process returned true");
    }

    #[test]
    fn bit_becomes_settled_after_offset_reached() {
        let calibration = super::SensorCalibration {
            trigger_value: 2000,
            untrigger_value: 1800,
        };
        let mut result = super::SplitFlapBitState::new(calibration, 58, 3);

        let sensor_value: u32 = 2100;

        let process = result.process(sensor_value);
        let process = result.process(sensor_value);
        let process = result.process(sensor_value);

        let process = result.process(sensor_value);
        assert!(
            result.bit_state == BitState::SETTLED,
            "Target reached, bit state not settled"
        );
    }

    #[test]
    fn after_homing_setting_character_changes_target_steps() {
        let calibration = super::SensorCalibration {
            trigger_value: 2000,
            untrigger_value: 1800,
        };
        let mut result = super::SplitFlapBitState::new(calibration, 58, 3);

        let sensor_value: u32 = 2100;

        let process = result.process(sensor_value);
        let process = result.process(sensor_value);
        let process = result.process(sensor_value);

        let process = result.process(sensor_value);

        result.set_target_character('A' as u8);

        assert_eq!(result.target_steps.homed_steps, 3 + 58);
    }

    #[test]
    fn process_after_homing_setting_character_becomes_seeking() {
        let calibration = super::SensorCalibration {
            trigger_value: 2000,
            untrigger_value: 1800,
        };
        let mut result = super::SplitFlapBitState::new(calibration, 58, 3);

        let sensor_value: u32 = 2100;

        let process = result.process(sensor_value);

        let sensor_value: u32 = 100;
        let process = result.process(sensor_value);
        let process = result.process(sensor_value);

        let process = result.process(sensor_value);

        result.set_target_character('A' as u8);

        let process = result.process(sensor_value);

        assert!(process, "Bit is not seeking for target character");
    }

    #[test]
    fn process_after_setting_character_twice_becomes_seeking() {
        let calibration = super::SensorCalibration {
            trigger_value: 2000,
            untrigger_value: 1800,
        };
        let mut result = super::SplitFlapBitState::new(calibration, 58, 3);

        let sensor_value: u32 = 2100;

        let process = result.process(sensor_value);

        let sensor_value: u32 = 100;
        let process = result.process(sensor_value);
        let process = result.process(sensor_value);

        let process = result.process(sensor_value);

        result.set_target_character('V' as u8);
        result.set_target_character('H' as u8);

        let process = result.process(sensor_value);

        assert_eq!(
            result.target_steps.homed_steps,
            (8 * 58) + 3,
            "Target steps is not as expected"
        );
    }

    #[test]
    fn after_setting_character_between_home_and_end_of_drum_target_steps_is_less_than_offset() {
        let calibration = super::SensorCalibration {
            trigger_value: 2000,
            untrigger_value: 1800,
        };
        let mut result = super::SplitFlapBitState::new(calibration, 58, 6 * 58 );

        let sensor_value: u32 = 2100;

        //Leads to home position found
        let process = result.process(sensor_value);

        result.set_target_character(0x0A as u8);

        assert_eq!(
            result.target_steps.homed_steps,
            (4 * 58),
            "Target steps is not as expected"
        );
    }

    #[test]
    fn space_returns_position_0() {
        let calibration = super::SensorCalibration {
            trigger_value: 2000,
            untrigger_value: 1800,
        };
        let mut result = super::SplitFlapBitState::new(calibration, 58, 3);

        let position = result.lookup_target_character_position(' ' as u8);

        assert_eq!(position, 0);
    }

    #[test]
    fn A_returns_position_1() {
        let calibration = super::SensorCalibration {
            trigger_value: 2000,
            untrigger_value: 1800,
        };
        let mut result = super::SplitFlapBitState::new(calibration, 58, 3);

        let position = result.lookup_target_character_position('A' as u8);

        assert_eq!(position, 1);
    }

    #[test]
    fn H_returns_position_8() {
        let calibration = super::SensorCalibration {
            trigger_value: 2000,
            untrigger_value: 1800,
        };
        let mut result = super::SplitFlapBitState::new(calibration, 58, 3);

        let position = result.lookup_target_character_position('H' as u8);

        assert_eq!(position, 8);
    }
}
