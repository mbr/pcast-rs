Unions without tags
===================

Assume the following:

* A network packet type with a fixed size of 8 bytes for all network packets
* The first byte determines packet type
* The remaining seven bytes are packet-type specific.

A library already implements the underlying type, `Packet`, for us:

```
#[repr(C)]
pub struct Packet {
    // packet type: 0x02 is "status"
    packet_type: u8,

    // 7 byte payload.
    // status: 4 byte u32 in big endian byteorder for node id, 3x1 byte status
    data: [u8; 7],
}
```

We now want to add a way to parse high-level protocol parsing.

We will use a status-packet for the example. Its packet type is `0x02`, and it will contain an unspecified 32-bit ID, followed by three status bytes as the payload.

For simplicity, all we want is checked casting (i.e. fail if packet type is wrong), no matching on packet type for now.


Possible solutions
------------------

Given a Packet `Packet` type for a network packet, possible solutions include:

* Use transmuting to convert a `Packet` to a `StatusPacket` type
* Use transmuting to convert a &Packet to a &StatusPacket
* Use a StatusView(Packet) wrapper.
* Use a StatusRef(&Packet) wrapper and a StatusMutRef(&mut Packet) wrapper.

Use cases
---------

There are a few use cases that we specifically want to be able to handle, preferably with no overhead:

1. A send function (requires a `&Packet` to read). It should be convenient
   to pass a `StatusPacket` or `&StatusPacket` to it somehow.
2. Methods on StatusPacket and Packet should ideally both work, if immutable.
   While Packet is borrowed/embedded in/converted to a StatusPacket, writes
   using the underlying `Packet` should be prevent, because they might destroy
   the `StatusPacket` structure.
3. Parsing (obtaining a `&StatusPacket`) should be possible using just a
   reference `&Packet`. This is important when multiple `Packet`s are stored
   in a collection
4. Passing a parsed packet to a function that only accepts parsed (e.g.
   `StatusPacket`) -- done to preserve information that the packet has already
   been parsed as valid.

Expressed differently, roughly these calls:

1. send_packet(&p)
2. s.base_method(), s.status_method()
3. let s: & = log.first()...  // Log is a Vec<Packet>
4. process_status_packet(s)
5. Optional: Allow direct access to fields (i.e. regular struct). Often not
   useful because of byteorder issues.

Functionality
-------------

* P->S: transmuting Packet into StatusPacket
* &P->&S: transmuting &Packet into &StatusPacket
* View: A StatusView owning a Packet
* RefView: A StatusRefView/StatusMutRefView owning a &Packet/&mut Packet

```
      P->S       &P->&S        View          RefView
 1.   Deref      Deref        Deref          Deref
 2.   Deref      Deref        Deref          Deref
 3.     X          ✓            X              ✓
 4.     ✓          X            ✓              X
 5.     ✓          ✓            X              X
```

Deref: Can be done using Deref
X: Not possible?
✓: Works

Implementations
---------------

The `viewtest.rs` implementation contains an possible implementation for the View-based version, while `casttest.rs` is a transmute/cast based one that uses types directly.

Open questions
--------------

* Is `Deref` being abused here, as we're trying to use the `*`-operator to get
  back two different types?
