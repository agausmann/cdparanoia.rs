# cdparanoia.rs

Rust bindings for libcdparanoia

## System dependencies

- libcdparanoia libraries and headers (tested with cdparanoia III 10.2)

### Void Linux

```
sudo xbps-install libcdparanoia-devel
```

## Usage

See [`cdparanoia/examples/rip_track.rs`](cdparanoia/examples/rip_track.rs) for 
an example of ripping a single track to a WAV file.

This crate is mostly undocumented, and unfortunately there is not much reference
material for libcdparanoia itself. The best references I've found are existing
applications using libcdparanoia. Here are a few that I've used:

- [the cdparanoia CLI](https://www.xiph.org/paranoia/)
- [ripright](https://www.mcternan.me.uk/ripright/)

The `cdparanoia` crate's API closely mirrors the C interface, except that most
of the functions are converted into member functions of `CdromDrive` and
`CdromParanoia`.
