# Same-size tagged data structure conversions

This crate provides a few boilerplate macros to enable conversions between types that are unions with a built-in discriminatory field. An example is a network protocol that consists of multiple packet-types with their respective packet-type indicated by a field on the struct:

```rust
#[repr(C)]
pub struct Packet {
    packet_type: u8,
    // an unknown (depends on packet type) payload
    data: [u8; 7],
}

#[repr(C)]
pub struct StatusPacket {
    /// must be 0x02 for a status packet
    packet_type: u8,
    status_0: u8,
    status_1: u8,
    status_2: u8,
    ts: [u8; 4],
}

#[macro_use]
extern crate pcast;

pub enum ConversionError {
    WrongPacketType
}

subtype_of!(Packet => StatusPacket | ConversionError {
    Ok(())
});

```

The StatusPacket has three fields for various flags and a four byte timestamp here; its presence is indicated by a value of 0x02 in packet_type.

The subtype_of macro can now be used to declare express this. As a result, a Packet can be try_into'd into a StatusPacket and references can be passed because &StatusPacket will Deref to &Packet.

A conversion from &mut StatusPacket to &mut Packet is not included, as altering the Packet-structure might violate invariants required by StatusPacket.
