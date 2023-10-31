pub mod comms;

use clap::ValueEnum;
pub use comms as communication;

mod vec3;
pub use vec3::Vec3;



#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Default)]
pub enum Connector {
    #[default]
    Qemu,
    Kvm,
    Pcileech
}

impl ToString for Connector {
    fn to_string(&self) -> String {
        match self {
            Connector::Qemu => String::from("qemu"),
            Connector::Kvm => String::from("kvm"),
            Connector::Pcileech => String::from("pcileech"),
        }
    }
}

/// Wrapper because log::LevelFilter doesn't implement ValueEnum
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Default)]
pub enum Loglevel {
    Error,
    #[default]
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<Loglevel> for log::LevelFilter {
    fn from(val: Loglevel) -> Self {
        match val {
            Loglevel::Error => log::LevelFilter::Error,
            Loglevel::Warn => log::LevelFilter::Warn,
            Loglevel::Info => log::LevelFilter::Info,
            Loglevel::Debug => log::LevelFilter::Debug,
            Loglevel::Trace => log::LevelFilter::Trace,
        }
    }
}