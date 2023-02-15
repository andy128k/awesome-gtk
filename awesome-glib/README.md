# Awesome GLib

[![Build](https://github.com/andy128k/awesome-gtk/actions/workflows/build.yml/badge.svg)](https://github.com/andy128k/awesome-gtk/actions/workflows/build.yml)
[![codecov](https://codecov.io/gh/andy128k/awesome-gtk/branch/main/graph/badge.svg)](https://codecov.io/gh/andy128k/awesome-gtk)
[![Crates.io](https://img.shields.io/crates/v/awesome-glib.svg)](https://crates.io/crates/awesome-glib)
[![Docs.rs](https://img.shields.io/docsrs/awesome-glib.svg)](https://docs.rs/awesome-glib)

Supplemental macros for glib/gio

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
