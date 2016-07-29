// The story behind this: Assume that you are being passed network packets
// that have all the same size, for example from a kernel UDP interface.
//
// The payload size is always the same and they are packets of a protocol
// that prefixes every payload with 2 bytes to indicate the packet type, while
// the remainder is all sorts of data, unique to every packet.
//
// There should be accessor methods to read the packet data, i.e. if one packet
// has player movement, a call to `player()` would return the player id,
// while a system message packet contains just contains a string.
//
// We're writing a generic networking library for our protocol and our first
// application is a network dumper that collects all UDP packets coming in
// (we're getting all the traffic passed to us), storing them in a vector
// for base packets.
//
// Now when it comes to parsing, we want a type that safely provides access
// only to methods the packet type supports, but does not incur overhead
// -- we want to keep the original C structure of the packet passed to us from
// the kernel.
//
// Here a some ideas. `Base` is a base packet, `type_field` is there but not
// used, `Pair` and `Quad` are standins for more specific packets.

use std::mem;

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
struct Base {
    type_field: u8,   // not used, but this is where type info could be
    data_field: u64,
}

#[repr(C, packed)]
#[derive(Debug)]
struct Pair {
    type_field: u8,
    elems: [u32; 2],
}

#[repr(C, packed)]
#[derive(Debug)]
struct Quad {
    type_field: u8,
    elems: [u16; 4]
}

impl Base {
    fn owned_pair(self) -> Pair {
        unsafe {
            mem::transmute(self)
        }
    }

    fn owned_quad(self) -> Quad {
        unsafe {
            mem::transmute(self)
        }
    }

    fn pair_view(self) -> PairView {
        PairView(self)
    }

    fn quad_view(self) -> QuadView {
        QuadView(self)
    }

    fn pair_ref(&self) -> &Pair {
        unsafe {
            mem::transmute(self)
        }
    }

    fn quad_ref(&self) -> &Quad {
        unsafe {
            mem::transmute(self)
        }
    }

    fn pair_view_ref(&self) -> &PairView {
        unsafe {
            mem::transmute(self)
        }
    }

    fn quad_view_ref(&self) -> &QuadView {
        unsafe {
            mem::transmute(self)
        }
    }
}

#[derive(Debug)]
struct PairView(Base);

#[derive(Debug)]
struct QuadView(Base);

// better: using generics
impl From<Base> for Pair {
    fn from(base: Base) -> Pair {
        unsafe { mem::transmute(base) }
    }
}

impl From<Base> for Quad {
    fn from(base: Base) -> Quad {
        unsafe { mem::transmute(base) }
    }
}

impl<'a> From<&'a Base> for &'a Pair {
    fn from(base: &'a Base) -> &'a Pair {
        unsafe { mem::transmute(base) }
    }
}

impl<'a> From<&'a Base> for &'a PairView {
    fn from(base: &'a Base) -> &'a PairView {
        unsafe { mem::transmute(base) }
    }
}

fn main() {
    let v = Base {
        type_field: 0x12,

        // our machine is big-endian though!
        data_field: 0x0123456789abcdef
    };

    println!("v {:>016X}", v.data_field);

    println!("\nowned conversion");
    let p = v.clone().owned_pair();
    println!("p {:>08X} {:>08X}", p.elems[0], p.elems[1]);

    let q = v.clone().owned_quad();
    println!("q {:>04X} {:>04X} {:>04X} {:>04X}", q.elems[0], q.elems[1], q.elems[2], q.elems[3]);

    println!("\nview conversion");
    let pv = PairView(v.clone());
    println!("pv {:>016X}", pv.0.data_field);

    let qv = QuadView(v.clone());
    println!("qv {:>016X}", qv.0.data_field);

    println!("\nref conversions");
    let pr = v.pair_ref();
    println!("pr {:>08X} {:>08X}", pr.elems[0], pr.elems[1]);

    let qr = v.quad_ref();
    println!("qr {:>04X} {:>04X} {:>04X} {:>04X}", qr.elems[0], qr.elems[1], qr.elems[2], qr.elems[3]);

    println!("\nref view conversions");
    let pvr = v.pair_view_ref();
    println!("pvr {:>016X}", pvr.0.data_field);

    let qvr = v.quad_view_ref();
    println!("qvr {:>016X}", qvr.0.data_field);


    // the generic variant
    println!("\nUsing generics");
    let gp: Pair = v.clone().into();
    println!("gp {:>08X} {:>08X}", gp.elems[0], gp.elems[1]);

    let gpr: &Pair = (&v).into();
    println!("gpr {:>08X} {:>08X}", gpr.elems[0], gpr.elems[1]);

    let gpvr: &PairView = (&v).into();
    println!("pvr {:>016X}", gpvr.0.data_field);

    let base = v.clone();
    let sub: &Pair = v.pair_ref();
    // let x = *sub;
}

