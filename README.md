# COLLABORATE [![Build Status](https://travis-ci.org/excaliburHisSheath/COLLABORATE.svg?branch=master)](https://travis-ci.org/excaliburHisSheath/collaborate)

COLLADA document parsing for Rust.

> NOTE: The current focus of COLLABORATE is to support the `1.4.1` COLLADA specification. Support
> for the `1.5.1` spec is planned, but if you'd like to help implement it, feel free to open issues
> in the issue tracker or make a pull request.

[COLLADA] is a COLLAborative Design Activity that defines an XML-based schema to
enable 3D authoring applications to freely exchange digital assets. It supports a vast array of
features used in 3D modeling, animation, and VFX work, and provides an open, non-proprietary
alternative to common formats like [FBX].

This crate provides functionality for parsing a COLLADA document and utilities for processing the
contained data, with the intention of enable direct usage of COLLADA data as well as
interchange of document data into other formats.

The easiest way to parse a COLLADA document is to load it from a file using
[`VersionedDocument::read`]:

```rust
use std::fs::File;
use collaborate::VersionedDocument;

let file = File::open("resources/blender_cube.dae")?;
match VersionedDocument::read(file)? {
    VersionedDocument::V1_4(document) => {
        println!("Loaded a 1.4.1 document: {:?}", document);
    }

    VersionedDocument::V1_5(document) => {
        println!("Loaded a 1.5.0 document: {:?}", document);
    }
}
```

[COLLADA]: https://www.khronos.org/collada/
[FBX]: https://en.wikipedia.org/wiki/FBX
[`VersionedDocument`]: https://docs.rs/collaborate/0.1/collaborate/enum.VersionedDocument.html
[`VersionedDocument::read`]: https://docs.rs/collaborate/0.1/collaborate/enum.VersionedDocument.html#method.read
