# r3d-pmx

This crate provides PMX 2.0 parser.

```rust
use pmx::{Pmx, PmxParseError};
use std::{fs::read, path::{Path}};

fn parse_pmx(path: impl AsRef<Path>) -> Result<Pmx, PmxParseError> {
  let buf = read(path).unwrap();
  let pmx = Pmx::parse(buf)?;

  println!("{}", pmx);

  Ok(pmx)
}
```
