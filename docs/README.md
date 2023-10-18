# Pi Boot Chain

This repository implements a boot chain loader for the Raspberry Pi 3 Model B.

There are three components:

- The **host** application (in the `host` directory)
- The **client** boot image (in the `client` directory)
- The **kernel image** (that you provide)

The client boot image is installed onto an SD card and loaded onto a Raspberry
Pi. On every boot, the client downloads the kernel image from the host and
executes it. This eliminates the need to unplug and modify the SD card.

## 🏃 Quick start

You will most likely need a USB-to-TTL serial cable to facilitate communication
between the host and the client.

You will need to install [Nix][nix-installer] before proceeding.

Start the Nix development shell:

```sh
nix develop
```

Build the client image:

```sh
cd client
cargo make
```

Create an SD card with the Raspberry Pi firmware. The easiest way to do this is
by installing [Raspberry Pi OS Lite (64-bit)][raspberrypi-os].

Copy the boot image
(`client/target/aarch64-unknown-none-softfloat/release/kernel8.img`) onto the
SD card (`/boot/kernel8.img`).

Run the host application, providing the path to your serial port and the path
to the kernel image you want to load:

```sh
cd host
cargo make
./target/release/host /dev/ttyUSB0 path/to/kernel8.img
```

Finally, boot the Raspberry Pi. A progress bar is shown as the kernel image is
transferred, followed by any output as the kernel image is executed.

At this point, you are connected to the serial device. Anything you type is
written to the serial device, and any output from the serial device is printed
to the screen.

You can reset the Raspberry Pi at any time to restart the boot process. The
kernel image is re-read from disk on every boot, so restarting the host
application should not be necessary.

## 📚 Resources

This project is heavily inspired by [raspbootin][raspbootin].

[nix-installer]: https://github.com/DeterminateSystems/nix-installer
[raspberrypi-os]: https://www.raspberrypi.com/software/operating-systems/
[raspbootin]: https://github.com/mrvn/raspbootin
