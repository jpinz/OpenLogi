//! Semantic haptic waveforms supported by Logitech's HID++ haptic feature.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A named waveform playable by a device with HID++ haptic feedback.
///
/// Variant order is wire format because this type crosses agent IPC. New
/// waveforms must be appended.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HapticWaveform {
    /// Sharp click for a state transition.
    SharpStateChange,
    /// Dampened click for a state transition.
    DampStateChange,
    /// Sharp boundary collision.
    SharpCollision,
    /// Dampened boundary collision.
    DampCollision,
    /// Light boundary collision.
    SubtleCollision,
    /// Pleasant notification pattern.
    HappyAlert,
    /// Urgent notification pattern.
    AngryAlert,
    /// Task-completion pattern.
    Completed,
    /// Repeating square pattern.
    Square,
    /// Rising and falling wave pattern.
    Wave,
    /// Burst pattern.
    Firework,
    /// Intense repeated pattern.
    Mad,
    /// Repeated knock pattern.
    Knock,
    /// Musical jingle pattern.
    Jingle,
    /// Ringing notification pattern.
    Ringing,
    /// Very light boundary collision.
    WhisperCollision,
}

impl HapticWaveform {
    /// Every waveform OpenLogi knows how to request.
    pub const ALL: [Self; 16] = [
        Self::SharpStateChange,
        Self::DampStateChange,
        Self::SharpCollision,
        Self::DampCollision,
        Self::SubtleCollision,
        Self::HappyAlert,
        Self::AngryAlert,
        Self::Completed,
        Self::Square,
        Self::Wave,
        Self::Firework,
        Self::Mad,
        Self::Knock,
        Self::Jingle,
        Self::Ringing,
        Self::WhisperCollision,
    ];

    /// Stable kebab-case name accepted by the CLI.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::SharpStateChange => "sharp-state-change",
            Self::DampStateChange => "damp-state-change",
            Self::SharpCollision => "sharp-collision",
            Self::DampCollision => "damp-collision",
            Self::SubtleCollision => "subtle-collision",
            Self::HappyAlert => "happy-alert",
            Self::AngryAlert => "angry-alert",
            Self::Completed => "completed",
            Self::Square => "square",
            Self::Wave => "wave",
            Self::Firework => "firework",
            Self::Mad => "mad",
            Self::Knock => "knock",
            Self::Jingle => "jingle",
            Self::Ringing => "ringing",
            Self::WhisperCollision => "whisper-collision",
        }
    }
}

impl fmt::Display for HapticWaveform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// A string that does not name a known haptic waveform.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
#[error("unknown haptic waveform `{value}`")]
pub struct ParseHapticWaveformError {
    value: String,
}

impl FromStr for HapticWaveform {
    type Err = ParseHapticWaveformError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let normalized = value.to_ascii_lowercase().replace('_', "-");
        Self::ALL
            .into_iter()
            .find(|waveform| waveform.name() == normalized)
            .ok_or_else(|| ParseHapticWaveformError {
                value: value.to_string(),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_names_round_trip_and_accept_sdk_style_underscores() {
        for waveform in HapticWaveform::ALL {
            assert_eq!(waveform.name().parse(), Ok(waveform));
        }
        assert_eq!(
            "SUBTLE_COLLISION".parse(),
            Ok(HapticWaveform::SubtleCollision)
        );
    }

    #[test]
    fn unknown_names_are_rejected() {
        assert_eq!(
            "buzz".parse::<HapticWaveform>(),
            Err(ParseHapticWaveformError {
                value: "buzz".to_string()
            })
        );
    }
}
