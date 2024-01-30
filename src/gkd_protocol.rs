/// Contains the protocol definition for gnome keyring daemon.
/// The protocol is used while communicating via unix domain sockets.
/// 
/// The protocol is defined here:
/// https://gitlab.gnome.org/GNOME/gnome-keyring/-/blob/master/daemon/control/gkd-control-codes.h?ref_type=heads

/// CONTROL CODES
pub(crate) enum ControlCodes {
    #[allow(unused)]
    Init = 0x00,
    Unlock = 0x01,
    #[allow(unused)]
    Change = 0x02,
    #[allow(unused)]
    Quit = 0x03,
}

/// RESULT CODES
pub(crate) enum ResultCodes {
    Ok = 0x00,
    Denied = 0x01,
    Failed = 0x02,
    NoDaemon = 0x03,
}

impl From<u32> for ResultCodes {
    fn from(code: u32) -> Self {
        match code {
            0x00 => ResultCodes::Ok,
            0x01 => ResultCodes::Denied,
            0x02 => ResultCodes::Failed,
            0x03 => ResultCodes::NoDaemon,
            _ => panic!("Unknown result code: {}", code),
        }
    }
}