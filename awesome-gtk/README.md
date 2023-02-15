# Awesome GTK

[![Build](https://github.com/andy128k/awesome-gtk/actions/workflows/build.yml/badge.svg)](https://github.com/andy128k/awesome-gtk/actions/workflows/build.yml)
[![codecov](https://codecov.io/gh/andy128k/awesome-gtk/branch/main/graph/badge.svg)](https://codecov.io/gh/andy128k/awesome-gtk)
[![Crates.io](https://img.shields.io/crates/v/awesome-gtk.svg)](https://crates.io/crates/awesome-gtk)
[![Docs.rs](https://img.shields.io/docsrs/awesome-gtk.svg)](https://docs.rs/awesome-gtk)

Assorted utilities for usage with gtk-rs

## Traverse widgets

```rust
use awesome_gtk::prelude::*;

// iterate over direct children
for child in widget.children() {

}

// iterate over direct children in reverse order
for child in widget.children_rev() {

}

// iterate over all children (depth-first)
for child in widget.traverse() {

}
```
