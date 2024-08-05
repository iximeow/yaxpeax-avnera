## yaxpeax-avnera

[![crate](https://img.shields.io/crates/v/yaxpeax-avnera.svg?logo=rust)](https://crates.io/crates/yaxpeax-avnera)
[![documentation](https://docs.rs/yaxpeax-avnera/badge.svg)](https://docs.rs/yaxpeax-avnera)

a decoder for the undocumented instruction set of Avnera processors, implemented as part of the yaxpeax project, including traits provided by [`yaxpeax-arch`](https://git.iximeow.net/yaxpeax-arch/about/).

users of this library will either want to use [quick and dirty APIs](https://docs.rs/yaxpeax-avnera/latest/yaxpeax_avnera/index.html#usage), or more generic decode interfaces from `yaxpeax-arch` - appropriate when mixing `yaxpeax-avnera` with other `yaxpeax` decoders, such as `yaxpeax-x86`.

### features

* it exists
* pretty small?
* `#[no_std]`

### it exists

i'm not aware of many other decoders or attempts to document this instruction set. definitely none as a standalone library.

### pretty small?

it's a small instruction set, the decoder is similarly tiny.

### `#[no_std]`

if, for some reason, you want to disassemble "`avnera`" instructions without the Rust standard library around, that should work. this is primarily for consistency with other decoders than any need, and is not particularly tested.
