# ZFS Dataset GUI

Quick experiment for rendering nested `zfs list` output with [vgtk].

## Run

Use `nix-shell` or install [whatever gtk needs][req], then `cargo run`.

[vgtk]: https://github.com/bodil/vgtk/
[req]: https://gtk-rs.org/docs/requirements

## Those TreeViews, though

Yes, I was annoyed by the amount of boilerplate
and made a macro and traits and all that around it.
Not sure it'll become something more generic,
but feel free to hack on it
and send some sweet sweet PRs.

## License

MIT
