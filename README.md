# The SliceVec crate

[![](http://meritbadge.herokuapp.com/slicevec)](https://crates.io/crates/slicevec)

**`SliceVec`** provides a dynamically growing vector using an external slice as the backing storage. This means that `SliceVec` is completely allocation-free and can be used in `no_std` environments.

In contrast to the very similar `arrayvec` crate, the user must provide the backing storage (which can reside anywhere in writable memory), it is not created automatically. While this makes `SliceVec` a bit less ergonomic since it requires more boilerplate code to use, it works with any size of slice (fixed-size array are limited to a fixed set of sizes, due to Rust's lack of integer generics).
