use core::fmt::Display;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The ID of a device onboard the CanSat, as specified by the protocol
///
/// TODO: Autogenerate the enum variants from the protocol mapping
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(u8)]
pub enum DeviceId {
    System = 1,
    PressureSensor = 2,
    TemperatureSensor = 3,
    HumiditySensor = 4,
    Gps = 5,
    Accelerometer = 6,
    Unknown = 31,
}

impl Display for DeviceId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            DeviceId::System => write!(f, "System Device (ID {})", *self as u8),
            DeviceId::PressureSensor => write!(f, "Pressure Sensor Device (ID {})", *self as u8),
            DeviceId::TemperatureSensor => {
                write!(f, "Temperature Sensor Device (ID {})", *self as u8)
            }
            DeviceId::HumiditySensor => write!(f, "Humidity Sensor Device (ID {})", *self as u8),
            DeviceId::Gps => write!(f, "GPS Device (ID {})", *self as u8),
            DeviceId::Accelerometer => write!(f, "Accelerometer Device (ID {})", *self as u8),
            DeviceId::Unknown => write!(f, "Unknown Device (ID {})", *self as u8),
        }
    }
}
