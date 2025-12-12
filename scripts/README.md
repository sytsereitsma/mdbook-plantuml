# Scripts

## MSI installer

See the [msi_installer/README.md](msi_installer/README.md) for instructions on building the windows MSI installer.

## Linux debian package

Install the cargo debian package:
```
cargo install cargo-deb --no-default-features
```

To build the debian package, run the following command:
```
cargo deb
```

This will create a debian package in the `target/debian` directory.