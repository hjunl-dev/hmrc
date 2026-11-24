use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MidiMessage {
    NoteOn { channel: u8, key: u8, velocity: u8 },
    NoteOff { channel: u8, key: u8, velocity: u8 },
    ControlChange { channel: u8, controller: u8, value: u8 },
    ActiveSensing,
    Other { status: u8 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    Empty,
    NotStatusByte(u8),
    WrongLength { status: u8, data_len: usize },
    InvalidDataByte(u8),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "empty message"),
            Self::NotStatusByte(b) => write!(f, "first byte {b:02X} is not a status byte"),
            Self::WrongLength { status, data_len } => {
                write!(f, "status {status:02X} has wrong data length {data_len}")
            }
            Self::InvalidDataByte(b) => write!(f, "invalid data byte {b:02X}"),
        }
    }
}

impl Error for ParseError {}

impl MidiMessage {
    pub fn parse(bytes: &[u8]) -> Result<Self, ParseError> {
        let (&status, data) = bytes.split_first().ok_or(ParseError::Empty)?;

        if status & 0x80 == 0 {
            return Err(ParseError::NotStatusByte(status));
        }

        if status >= 0xF0 {
            return Ok(match status {
                0xFE => Self::ActiveSensing,
                _ => Self::Other { status },
            });
        }

        if let Some(&bad) = data.iter().find(|&&b| b & 0x80 != 0) {
            return Err(ParseError::InvalidDataByte(bad));
        }

        let kind = status >> 4;
        let channel = status & 0x0F;

        match (kind, data) {
            (0x9, &[key, 0]) => Ok(Self::NoteOff { channel, key, velocity: 0 }),
            (0x9, &[key, velocity]) => Ok(Self::NoteOn { channel, key, velocity }),
            (0x8, &[key, velocity]) => Ok(Self::NoteOff { channel, key, velocity }),
            (0xB, &[controller, value]) => Ok(Self::ControlChange { channel, controller, value }),

            (0xA | 0xE, &[_, _]) | (0xC | 0xD, &[_]) => Ok(Self::Other { status }),

            _ => Err(ParseError::WrongLength { status, data_len: data.len() }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn note_on() {
        assert_eq!(
            MidiMessage::parse(&[0x90, 0x3C, 0x64]),
            Ok(MidiMessage::NoteOn { channel: 0, key: 60, velocity: 100 })
        );
    }

    #[test]
    fn note_on_with_zero_velocity_is_note_off() {
        assert_eq!(
            MidiMessage::parse(&[0x90, 0x3C, 0x00]),
            Ok(MidiMessage::NoteOff { channel: 0, key: 60, velocity: 0 })
        );
    }

    #[test]
    fn note_off() {
        assert_eq!(
            MidiMessage::parse(&[0x80, 0x3C, 0x40]),
            Ok(MidiMessage::NoteOff { channel: 0, key: 60, velocity: 64 })
        );
    }

    #[test]
    fn channel_is_low_nibble() {
        assert_eq!(
            MidiMessage::parse(&[0x93, 0x3C, 0x64]),
            Ok(MidiMessage::NoteOn { channel: 3, key: 60, velocity: 100 })
        );
    }

    #[test]
    fn sustain_pedal() {
        assert_eq!(
            MidiMessage::parse(&[0xB0, 0x40, 0x7F]),
            Ok(MidiMessage::ControlChange { channel: 0, controller: 64, value: 127 })
        );
    }

    #[test]
    fn active_sensing() {
        assert_eq!(MidiMessage::parse(&[0xFE]), Ok(MidiMessage::ActiveSensing));
    }

    #[test]
    fn unhandled_messages_are_other() {
        // Program Change: data 1개
        assert_eq!(MidiMessage::parse(&[0xC0, 0x05]), Ok(MidiMessage::Other { status: 0xC0 }));
        // Timing Clock: 시스템 실시간 메시지
        assert_eq!(MidiMessage::parse(&[0xF8]), Ok(MidiMessage::Other { status: 0xF8 }));
    }

    #[test]
    fn empty_is_error() {
        assert_eq!(MidiMessage::parse(&[]), Err(ParseError::Empty));
    }

    #[test]
    fn data_byte_first_is_error() {
        assert_eq!(MidiMessage::parse(&[0x3C, 0x64]), Err(ParseError::NotStatusByte(0x3C)));
    }

    #[test]
    fn truncated_note_on_is_error() {
        assert_eq!(
            MidiMessage::parse(&[0x90, 0x3C]),
            Err(ParseError::WrongLength { status: 0x90, data_len: 1 })
        );
    }

    #[test]
    fn data_byte_with_high_bit_is_error() {
        assert_eq!(
            MidiMessage::parse(&[0x90, 0x80, 0x10]),
            Err(ParseError::InvalidDataByte(0x80))
        );
    }
}