use std::mem::transmute;
use std::ops::{Deref, DerefMut};

#[repr(C)]
pub struct Packet {
    // packet type: 0 => ping, 1 => pong, 2 => status
    packet_type: u8,

    // 7 byte payload.
    // ping: 7 bytes random data
    // pong: 7 bytes random data (copied from ping)
    // status: 4 byte u32 in big endian byteorder for node id, 3x1 byte status
    data: [u8; 7],
}

pub struct PingPacket(Packet);
pub struct PingRef<'a>(&'a Packet);
pub struct PingMutRef<'a>(&'a mut Packet);

pub struct PongPacket(Packet);
pub struct PongRef<'a>(&'a Packet);
pub struct PongMutRef<'a>(&'a mut Packet);

pub struct StatusPacket(Packet);
pub struct StatusRef<'a>(&'a Packet);
pub struct StatusMutRef<'a>(&'a mut Packet);

impl Packet {
    // conversion functions
    pub fn get_status_ref(&self)-> StatusRef {
        assert!(self.packet_type == 2);
        StatusRef(self)
    }

    pub fn get_status_mut_ref(&mut self) -> StatusMutRef {
        assert!(self.packet_type == 2);
        StatusMutRef(self)
    }

    pub fn get_raw_payload(&self) -> &[u8] {
        &self.data
    }

    pub fn new(packet_type: u8, data: [u8; 7]) -> Packet {
        Packet {
            packet_type: packet_type,
            data: data,
        }
    }
}

impl<'a> StatusRef<'a> {
    pub fn get_status_2(&self) -> u8 {
        self.0.data[6]
    }
}

impl<'a> StatusMutRef<'a> {
    pub fn set_status_2(&mut self, v: u8) {
        self.0.data[6] = v
    }
}

impl<'a> Deref for StatusRef<'a> {
    type Target = Packet;

    fn deref(&self) -> &Packet {
        self.0
    }
}


impl<'a> Deref for StatusMutRef<'a> {
    type Target = StatusRef<'a>;

    fn deref(&self) -> &StatusRef<'a> {
        // DOESNT WORK
        // &StatusRef(&self.0)
        unsafe {
            &*((self as *const StatusMutRef) as *const StatusRef)
        }
    }
}

// we do not implement DerefMut here, because altering the packet like a base
// packet would result in being able to create an invalid base
// impl<'a> DerefMut for StatusMutRef<'a> {
//     type Target = Packet;

//     fn deref(&mut self) -> &mut Packet {
//         self.0
//     }
// }

// impl<'a> Deref for StatusMutRef<'a> {
//     type Target = StatusRef<'a>;

//     fn deref(&self) -> &Packet {
//         self.0
//     }
// }

#[cfg(test)]
mod test {
    use super::*;

    /// send takes a raw packet to send
    fn send(packet: &Packet) {
        let _ = packet.get_raw_payload();
        // ...
    }

    #[test]
    fn test_send() {
        let mut owned = Packet::new(2, b"0123456".to_owned());

        send(&owned);

        {
            let status_view = owned.get_status_ref();
            send(&status_view);
        }

        let mut status_mut_view = owned.get_status_mut_ref();
        status_mut_view.set_status_2(0x12);
        assert_eq!(status_mut_view.get_status_2(), 0x12);
        send(&status_mut_view);
    }
}

