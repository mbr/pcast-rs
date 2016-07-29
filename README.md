Approaches for discriminating unions without tags
=================================================

* Use case: A network protocol that outputs packets with a few bits indicating packet type.

So far
------

Given a Base type for a network packet, wanting to get a Sub type out of it, because a few bits indicated the type:

* Use transmuting to convert to a Sub type
  + Seemingly "cleaned solution"
* Use transmuting to convert a &Base to a &Sub
  + Need to check if Rust's lifetimes properly support this, but they should
  + Allows use in containers (i.e. Vec<Base>)
* Use a SubView(Base) wrapper.
  + Can hide Base if desired
  + 0-overhead if transmute, but simply using safe code should work as well
  + does NOT work with Vec
* Use a SubRef(&Base) wrapper.
  + Works with Vec
  + Can also work as SubRef(&mut Base)
  + Cannot provide backing store (i.e. Base needs to be stored elsewhere)
  + Properly encodes checking needs to be done (or not), if we have the SubRef
    type, we know checking has been done
  + Cannot use mutated field access (changing of underlying struct)

Use cases
---------

* A send function. send(&Base). Should be possible to pass &Sub to it somehow.
* Methods on Sub and Base. Base methods should ideally still work? However, there may be destructive methods that destroy the Sub types integrity if used. Should not implement DerefMut!
* Ability to get Sub-type from just a &Base (retrieved from Vec).
* Passing an owned but parsed Packet on to another function.

or differently:

1. send(&b)
2. s.base_method(), s.sub_method()
3. let s = log.first().to_sub()
4. sub_process(s)

Applicability
=============

     Base->Sub  &Base->&Sub  SubView(Base)  SubRef(&Base)
 1.   Deref      Deref        Deref          Deref
 2.   Deref      Deref        Deref          Deref
 3.     ?          ✓            ?              ✓
 4.     ✓          X            ✓              X


Verdict
-------

&Base -> &Sub conversion requires Base -> Sub conversion to be present

Optional: Implement Deref<Base> for every Sub? Would allow passing in a Sub packet where a Base one is needed. Alternatively, implement it for the Wrapper SubView/SubRef types? I.e. SubRef(&Base) should implement Deref<&Base>

Oli's proposal
--------------

Generalizes a conversion based on an arbitrary variant discovery function
`t: &Base -> usize` (the discriminant) and a match arm:

Example:

```
match t(package) {
    0b101 => {
        // package is of type SubA
        let p: SubA = unsafe { transmute(package) }
    },
    0b100 => {
        // package is of type SubB
        let p: SubB = unsafe { transmute(package) }
    }
}
```

Can be used using refs as well:

```
match t(package) {
    0b101 => {
        // package is of type SubA
        let p: &SubA = unsafe { transmute(&package) }
    },
    0b100 => {
        // package is of type SubB
        let p: &SubB = unsafe { transmute(&package) }
    }
}
```

Possible improvements include using an ENUM as the discriminant. However, the whole proposal seems to solve a slightly orthogonal problem (a function that can return different types)?
