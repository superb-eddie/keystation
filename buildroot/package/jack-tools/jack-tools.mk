JACK_TOOLS_VERSION = 7cf014d3b3b75ad88a0785957b0f2cffad243b6b
JACK_TOOLS_SITE = $(call github,jackaudio,jack-example-tools,$(JACK_TOOLS_VERSION))
JACK_TOOLS_INSTALL_TARGET = YES
JACK_TOOLS_DEPENDENCIES = host-pkgconf jack2 libsndfile libsamplerate alsa-lib

$(eval $(meson-package))