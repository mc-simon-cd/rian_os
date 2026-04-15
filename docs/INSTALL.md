# Installation & Setup Guide for R-OS

This guide provides step-by-step instructions to set up the development environment for the R-OS microkernel.

## 1. Toolchain Setup

R-OS is built using the Rust **nightly** toolchain due to its reliance on unstable features for bare-metal development.

```bash
# Install the nightly toolchain
rustup toolchain install nightly

# Add the bare-metal target for x86_64
rustup target add x86_64-unknown-none

# Install rust-src component (required for building the kernel core)
rustup component add rust-src
```

## 2. Bootloader & Kernel Tools

We use the `bootloader` crate and the `bootimage` tool to package our kernel into a bootable disk image.

```bash
# Install the bootimage tool
cargo install bootimage
```

## 3. Hardware Emulator (QEMU)

To run the kernel, you need QEMU installed on your host system.

### Linux (Arch/Manjaro)
```bash
sudo pacman -S qemu-system-x86
```

### Linux (Ubuntu/Debian)
```bash
sudo apt-get install qemu-system-x86
```

### macOS
```bash
brew install qemu
```

## 4. Building and Running

Once the environment is set up, you can build and run the kernel with a single command:

```bash
# Build the kernel and launch QEMU
cargo run
```

If you only want to build the kernel binary:
```bash
cargo build --target x86_64-unknown-none
```

## 5. Troubleshooting

- **Error: `linker 'rust-lld' not found`**: Ensure you have the `lld` linker installed or that your Rust installation is complete.
- **Error: `target not found`**: Double check that you added `x86_64-unknown-none` via `rustup`.
- **QEMU not launching**: Ensure `qemu-system-x86_64` is in your PATH.

---
*For more information on the architecture, see [README.md](file:///home/can/Masaüstü/projeler/rian_cekirdek/rian_cekirdek/README.md).*
