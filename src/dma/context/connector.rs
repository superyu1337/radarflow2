#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum, Default)]
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
