#![no_std]
use defmt::Format;

#[macro_use]
extern crate uom;

pub mod header;
pub mod messages;
pub mod sink;
pub mod source;
pub mod timers;
pub mod token;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Format)]
pub enum CcPin {
    CC1,
    CC2,
}

impl core::ops::Not for CcPin {
    type Output = CcPin;

    fn not(self) -> Self::Output {
        match self {
            CcPin::CC1 => CcPin::CC2,
            CcPin::CC2 => CcPin::CC1,
        }
    }
}

#[derive(Clone, Copy, Debug, Format)]
pub enum PowerRole {
    Source,
    Sink,
}

impl From<bool> for PowerRole {
    fn from(value: bool) -> Self {
        match value {
            false => Self::Sink,
            true => Self::Source,
        }
    }
}

impl From<PowerRole> for bool {
    fn from(role: PowerRole) -> bool {
        match role {
            PowerRole::Sink => false,
            PowerRole::Source => true,
        }
    }
}

#[derive(Clone, Copy, Debug, Format)]
pub enum DataRole {
    Ufp,
    Dfp,
}

impl From<bool> for DataRole {
    fn from(value: bool) -> Self {
        match value {
            false => Self::Ufp,
            true => Self::Dfp,
        }
    }
}

impl From<DataRole> for bool {
    fn from(role: DataRole) -> bool {
        match role {
            DataRole::Ufp => false,
            DataRole::Dfp => true,
        }
    }
}
