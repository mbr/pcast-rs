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
  + Gives free Base -> Sub conversion through DeRef maybe?
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
