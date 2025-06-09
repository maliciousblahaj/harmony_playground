use std::fmt::Display;

// A 12-TET Note
pub enum NoteName {
    C,
    CSharp,
    D,
    DSharp,
    E,
    F,
    FSharp,
    G,
    GSharp,
    A,
    ASharp,
    B,
}

impl NoteName {
    pub fn from_index(i: isize) -> Self {
        match i {
            0 => Self::C,
            1 => Self::CSharp,
            2 => Self::D,
            3 => Self::DSharp,
            4 => Self::E,
            5 => Self::F,
            6 => Self::FSharp,
            7 => Self::G,
            8 => Self::GSharp,
            9 => Self::A,
            10 => Self::ASharp,
            11 => Self::B,
            n => {
                panic!("Invalid notename index \"{n}\" (try mod 12)");
            }
        }
    }

    pub fn get_str(&self) -> &'static str {
        match self {
            NoteName::C => "C",
            NoteName::CSharp => "C#",
            NoteName::D => "D",
            NoteName::DSharp => "D#",
            NoteName::E => "E",
            NoteName::F => "F",
            NoteName::FSharp => "F#",
            NoteName::G => "G",
            NoteName::GSharp => "G#",
            NoteName::A => "A",
            NoteName::ASharp => "A#",
            NoteName::B => "B",
        }
    }
}

pub struct Note {
    note_name: NoteName,
    octave: i8,
    cent_offset: f32,
}

impl Note {
    const C4_FREQUENCY: f32 = 261.635_56;
    pub fn from_frequency(frequency: f32) -> Self {
        // frequency ratio from C4
        let ratio = frequency / Self::C4_FREQUENCY;
        let ratio_log_2 = ratio.log2();
        let cents_from_c4 = ratio_log_2 * 1200.0;
        let closest_note_half_steps = (ratio_log_2 * 12.0).round();
        let note_name = NoteName::from_index((closest_note_half_steps as isize).rem_euclid(12));
        let cent_offset = cents_from_c4 - closest_note_half_steps * 100.0;
        let octave = 4 + (closest_note_half_steps / 12.0).floor() as i8;
        Self {
            note_name,
            octave,
            cent_offset,
        }
    }
}

impl Display for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cent_offset = self.cent_offset.round();
        write!(
            f,
            "{}{} ({}{}c)",
            self.note_name.get_str(),
            self.octave,
            if cent_offset == 0.0 {
                ""
            } else if cent_offset.is_sign_positive() {
                "+"
            } else {
                "-"
            },
            cent_offset.abs(),
        )
    }
}
