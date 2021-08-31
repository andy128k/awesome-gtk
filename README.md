# Awesome GTK

[![Build](https://github.com/andy128k/awesome-gtk/actions/workflows/build.yml/badge.svg)](https://github.com/andy128k/awesome-gtk/actions/workflows/build.yml)
[![codecov](https://codecov.io/gh/andy128k/awesome-gtk/branch/main/graph/badge.svg)](https://codecov.io/gh/andy128k/awesome-gtk)
[![Crates.io Status](https://img.shields.io/crates/v/awesome-glib.svg)](https://crates.io/crates/awesome-glib)

Supplemental macros for gtk-rs

## Actions

TLDR;

```rust
struct MyWidget ...

#[awesome_glib::actions]
impl MyWidget {
    fn action1(&self) ...
    fn action2(&self) ...
}

impl ObjectImpl for MyWidgetPrivate {
    fn constructed(&self, obj: &Self::Type) {
        ...
        obj.register_actions(obj);
        ...
    }
}
```
