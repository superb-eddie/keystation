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

## Debian deps:
```
sudo apt install git build-essential gcc g++ gperf bison \ 
  flex texinfo help2man make libncurses5-dev python3-dev \
  autoconf automake libtool libtool-bin gawk wget bzip2 \
  xz-utils unzip patch libstdc++6 rsync meson ninja-build
```

## Next steps
- arduino side:
    - More options should be moved into the target.json (panic strategy)
  - increase velocity measurement accuracy
  - send keystrokes + velocity out the serial port
- can be done on pi 2
  - get arm build of vcv-rack working
  - get linux build + bootload + sdcard formatter working for PI
  - build small init process to run vcvrack + screen + midi
- requires pi 4
  - test macos m1 vcv rack plugins (newer pi == hopefully closer instruction sets)
    - if they don't work, get a set of plugins to be bundled
  - get usb-hid working
    - https://forums.raspberrypi.com/viewtopic.php?t=341244
  - get oled working
    - https://www.adafruit.com/product/3531
  - OLED screen should be able to switch between internal synth and midi
  - expose block storage over usb when in "instrument" mode to upload patchs
  - expose midi when in "midi" mode
  - Do we have enough I/O to connect volume/pitch and buttons to the pi?
- Janko conversion
  - replicate key mechanism in openscad
  - build janko keys on top
  - 3d print