#!/bin/sh
# Assign's names to usb tty devices based on which usb port they're plugged into
# Trigger by mdev on hot plug events

PHYSICAL_LOCATION=$(basename $(readlink -f "/sys/class/tty/${MDEV}/device/.."))

DEVNAME=""
case "${PHYSICAL_LOCATION}" in
  "1-1.4:1.0") DEVNAME="ttyUSBkeyboard" ;;
  "1-1.2:1.0") DEVNAME="ttyUSBdials" ;;
  *) exit 0 ;;
esac

case "${ACTION}" in
  "add") ln -sf "/dev/${MDEV}" "/dev/${DEVNAME}" ;;
  "remove") rm -f "/dev/${DEVNAME}" ;;
esac