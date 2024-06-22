# Keystation
Replacing the internals of an M-Audio Keystation 49e midi keyboard.

# Developing
## Host computer setup

Any linux system should work, install dependencies for:
- [Crosstool-NG](https://crosstool-ng.github.io/docs/os-setup/)
- [Buildroot](https://buildroot.org/downloads/manual/manual.html#requirement-mandatory)

Also install:
- avrdude

### Debian
```
sudo apt install git build-essential gcc g++ gperf bison \ 
  flex texinfo help2man make libncurses5-dev python3-dev \
  autoconf automake libtool libtool-bin gawk wget bzip2 \
  xz-utils unzip patch libstdc++6 rsync meson ninja-build \
  pkg-config libudev-dev avrdude jq bc
```
