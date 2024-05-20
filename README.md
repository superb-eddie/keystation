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