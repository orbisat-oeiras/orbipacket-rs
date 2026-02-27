use core::fmt::Display;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(thiserror::Error, Debug)]
pub enum DeviceIdError {
    #[error("invalid device id: {0}")]
    InvalidId(u8),
}

/// The ID of a device onboard the CanSat, as specified by the protocol
///
/// TODO: Autogenerate the enum variants from the protocol mapping
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(u8)]
pub enum DeviceId {
    System = 0,
    TimeSync = 1,
    Gps = 2,
    Camera = 3,
    Accelerometer = 4,
    Gyroscope = 5,
    Altimeter = 6,
    Magnetometer = 7,
    PressureSensor = 8,
    TemperatureSensor = 9,
    HumiditySensor = 10,
    RadiationSensor = 11,
    Mission1 = 12,
    Mission2 = 13,
    Mission3 = 14,
    Mission4 = 15,
}

impl Display for DeviceId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            DeviceId::System => write!(f, "System Device (ID {})", *self as u8),
            DeviceId::TimeSync => write!(f, "Time Sync Device (ID {})", *self as u8),
            DeviceId::Gps => write!(f, "GPS Device (ID {})", *self as u8),
            DeviceId::Camera => write!(f, "Camera Device (ID {})", *self as u8),
            DeviceId::Accelerometer => write!(f, "Accelerometer Device (ID {})", *self as u8),
            DeviceId::Gyroscope => write!(f, "Gyroscope Device (ID {})", *self as u8),
            DeviceId::Altimeter => write!(f, "Altimeter Device (ID {})", *self as u8),
            DeviceId::Magnetometer => write!(f, "Magnetometer Device (ID {})", *self as u8),
            DeviceId::PressureSensor => write!(f, "Pressure Sensor Device (ID {})", *self as u8),
            DeviceId::TemperatureSensor => {
                write!(f, "Temperature Sensor Device (ID {})", *self as u8)
            }
            DeviceId::HumiditySensor => write!(f, "Humidity Sensor Device (ID {})", *self as u8),
            DeviceId::RadiationSensor => write!(f, "Radiation Sensor Device (ID {})", *self as u8),
            DeviceId::Mission1 | DeviceId::Mission2 | DeviceId::Mission3 | DeviceId::Mission4 => {
                write!(f, "Mission Device (ID {})", *self as u8)
            }
        }
    }
}

impl TryFrom<u8> for DeviceId {
    type Error = DeviceIdError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(DeviceId::System),
            1 => Ok(DeviceId::TimeSync),
            2 => Ok(DeviceId::Gps),
            3 => Ok(DeviceId::Camera),
            4 => Ok(DeviceId::Accelerometer),
            5 => Ok(DeviceId::Gyroscope),
            6 => Ok(DeviceId::Altimeter),
            7 => Ok(DeviceId::Magnetometer),
            8 => Ok(DeviceId::PressureSensor),
            9 => Ok(DeviceId::TemperatureSensor),
            10 => Ok(DeviceId::HumiditySensor),
            11 => Ok(DeviceId::RadiationSensor),
            12 => Ok(DeviceId::Mission1),
            13 => Ok(DeviceId::Mission2),
            14 => Ok(DeviceId::Mission3),
            15 => Ok(DeviceId::Mission4),
            _ => Err(DeviceIdError::InvalidId(value)),
        }
    }
}
