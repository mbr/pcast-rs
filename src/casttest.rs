use std::convert::TryFrom;
use std::ops::Deref;
use std::mem::transmute;

trait SubtypeCheck<F, T, E> {
    fn check_is_valid_subtype(&self) -> Result<(), E>;
}

macro_rules! subtype_of {
    ($base:path => $sub:path | $cerr:ty $check_fn:block) => (
        impl SubtypeCheck<$base, $sub, $cerr> for $base {
            fn check_is_valid_subtype(&self) -> Result<(), $cerr> $check_fn
        }

        impl Deref for $sub {
            type Target = $base;

            #[inline(always)]
            fn deref(&self) -> &$base {
                unsafe { transmute::<&$sub, &$base>(self) }
            }
        }

        impl TryFrom<$base> for $sub {
            type Err = $cerr;

            #[inline(always)]
            fn try_from(base: $base) -> Result<Self, Self::Err> {
                try!(SubtypeCheck::<$base, $sub, $cerr>::check_is_valid_subtype(&base));
                Ok(unsafe { transmute::<$base, $sub>(base) })
            }
        }

        impl<'a> TryFrom<&'a $base> for &'a $sub {
            type Err = $cerr;

            #[inline(always)]
            fn try_from(base_ref: &$base) -> Result<Self, Self::Err> {
                try!(SubtypeCheck::<$base, $sub, $cerr>::check_is_valid_subtype(base_ref));
                Ok(unsafe { transmute::<&$base, &$sub>(base_ref) })
            }
        }

        impl<'a> TryFrom<&'a mut $base> for &'a mut $sub {
            type Err = $cerr;

            #[inline(always)]
            fn try_from(base_ref: &mut $base) -> Result<Self, Self::Err> {
                try!(SubtypeCheck::<$base, $sub, $cerr>::check_is_valid_subtype(base_ref));
                Ok(unsafe { transmute::<&mut $base, &mut $sub>(base_ref) })
            }
        }

    )
}

#[repr(C)]
pub struct Packet {
    // packet type: 0x02 is "status"
    packet_type: u8,

    // 7 byte payload.
    // status: 4 byte u32 in big endian byteorder for node id, 3x1 byte status
    data: [u8; 7],
}

#[repr(C)]
pub struct StatusPacket {
    packet_type: u8,
    ts: [u8; 4],
    status_0: u8,
    status_1: u8,
    status_2: u8,
}

#[repr(C, packed)]
pub struct PingPacket {
    packet_type: u8,
    dummy: u32,
    unused: [u8; 3],
}

#[repr(C, packed)]
pub struct PongPacket {
    packet_type: u8,
    dummy: u32,
    unused: [u8; 3],
}

pub struct PongConvError {

}

subtype_of!(Packet => PingPacket | ConversionError {
    Ok(())
});
subtype_of!(Packet => PongPacket | PongConvError {
    Err(PongConvError {})
});

#[derive(Debug)]
pub enum ConversionError{}

impl TryFrom<Packet> for StatusPacket {
    type Err = ConversionError;

    fn try_from(packet: Packet) -> Result<Self, Self::Err> {
        // FIXME: check if packet_type matches
        Ok(unsafe { transmute(packet) })
    }
}

impl<'a> TryFrom<&'a Packet> for &'a StatusPacket {
    type Err = ConversionError;

    fn try_from(packet_ref: &Packet) -> Result<Self, Self::Err> {
        // FIXME: check here
        Ok(unsafe { transmute(packet_ref) })
    }
}

impl<'a> TryFrom<&'a mut Packet> for &'a mut StatusPacket {
    type Err = ConversionError;

    fn try_from(packet_ref: &mut Packet) -> Result<Self, Self::Err> {
        // FIXME: check here
        Ok(unsafe { transmute(packet_ref) })
    }
}

impl Packet {
    pub fn get_raw_payload(&self) -> &[u8] {
        &self.data
    }

    pub fn set_raw_payload(&mut self, data: [u8; 7]) {
        self.data = data
    }

    pub fn new(packet_type: u8, data: [u8; 7]) -> Packet {
        Packet {
            packet_type: packet_type,
            data: data,
        }
    }
}

impl StatusPacket {
    pub fn get_status_2(&self) -> u8 {
        self.status_2
    }

    pub fn set_status_2(&mut self, v: u8) {
        self.status_2 = v
    }

}

impl Deref for StatusPacket {
    type Target = Packet;

    fn deref(&self) -> &Packet {
        unsafe { transmute(self) }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::convert::TryInto;

    /// send takes a raw packet to send
    fn send(packet: &Packet) {
        let _ = packet.get_raw_payload();
        // ...
    }

    fn swallow_status_packet(_: StatusPacket) {
        // goodbye, s!
    }

    #[test]
    fn test_send() {
        let mut owned = Packet::new(2, b"0123456".to_owned());
        send(&owned);

        {
            let status_view: &StatusPacket = (&owned).try_into().unwrap();
            send(status_view);
        }

        let mut status_mut_ref: &mut StatusPacket = (&mut owned).try_into().unwrap();
        status_mut_ref.set_status_2(0x12);
        assert_eq!(status_mut_ref.get_status_2(), 0x12);
        send(&status_mut_ref);
    }

    #[test]
    fn send_from_ref() {
        let mut owned = Packet::new(2, b"0123456".to_owned());

        let pref: &mut Packet = &mut owned;

        send(pref);

        {
            // FIXME: DerefMut not a good idea beacause we don't want
            //        to allow manipulations on a reference -- it might
            //        invalidate StatusPacket
            //        &(*pref) seems kind of silly though to drop the mut
            let status_view: &StatusPacket = (&(*pref)).try_into().unwrap();
            send(&status_view);
        }

        let mut status_mut_view: &mut StatusPacket = pref.try_into().unwrap();
        status_mut_view.set_status_2(0x12);
        assert_eq!(status_mut_view.get_status_2(), 0x12);
        send(&status_mut_view);
    }

    #[test]
    fn call_base_and_sub_methods() {
        let mut owned = Packet::new(2, b"0123456".to_owned());
        owned.set_raw_payload(b"xxxxxxx".to_owned());

        {
            let status_view: &StatusPacket = (&owned).try_into().unwrap();
            status_view.get_status_2();
            status_view.get_raw_payload();
        }

        let mut status_mut_view: &mut StatusPacket = (&mut owned).try_into().unwrap ();
        status_mut_view.get_status_2();
        status_mut_view.get_raw_payload();
        status_mut_view.set_status_2(0x34);

        // does not work (and shouldn't):
        // status_mut_view.set_raw_payload(b"xxxxxxx".to_owned());
    }

    #[test]
    fn create_from_immutable_ref() {
        let v = vec![Packet::new(2, b"0123456".to_owned())];

        for p in v.iter() {
            let status_view: &StatusPacket = (&(*p)).try_into().unwrap();
            status_view.get_status_2();
        }
    }

    #[test]
    fn use_owned_status() {
        let p = Packet::new(2, b"0123456".to_owned());

        let s: StatusPacket = p.try_into().unwrap();

        swallow_status_packet(s);
    }
}
