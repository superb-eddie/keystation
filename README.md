# Keystation
An attempt to replace the guts of a non-functioning M-Audio Keystation 49e

## Hardware plans
- A microcontroller takes input from keys/switches and outputs a stream of serial messages
- An SBC translates the serial message into a midi messages, or runs a software instrument directly

## Structure
- hardware/
  - Documentation/artifacts from 
    - reverse engineering old hardware
    - engineering new hardware
- keystation-firmware/
  - Firmware for translating inputs to serial commands
- keystation-host/
  - Host software for translating serial commands to midi commands

# Developing:

## Host computer setup

Any linux system should work, install dependencies for:
- [Crosstool-NG](https://crosstool-ng.github.io/docs/os-setup/)
- [Buildroot](https://buildroot.org/downloads/manual/manual.html#requirement-mandatory)

Also install:
- avrdude
- jq

### Debian
```
sudo apt install git build-essential gcc g++ gperf bison \ 
  flex texinfo help2man make libncurses5-dev python3-dev \
  autoconf automake libtool libtool-bin gawk wget bzip2 \
  xz-utils unzip patch libstdc++6 rsync meson ninja-build \
  pkg-config libudev-dev avrdude jq bc
```
