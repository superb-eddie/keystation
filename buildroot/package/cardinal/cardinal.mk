CARDINAL_VERSION = 24.05
CARDINAL_SITE = git@github.com:DISTRHO/Cardinal.git
CARDINAL_SITE_METHOD = git
CARDINAL_GIT_SUBMODULES = YES
CARDINAL_DL_OPTS = --recurse-submodules
CARDINAL_INSTALL_STAGING = NO
CARDINAL_INSTALL_TARGET = YES
CARDINAL_DEPENDENCIES = liblo fftw-single libsndfile libarchive libsamplerate jansson speexdsp

define CARDINAL_BUILD_CMDS
    $(MAKE) $(TARGET_CONFIGURE_OPTS) CROSS_COMPILING=true HEADLESS=true -C $(@D)
endef

define CARDINAL_INSTALL_TARGET_CMDS
	$(MAKE) $(TARGET_CONFIGURE_OPTS) SYSDEPS=true CROSS_COMPILING=true HEADLESS=true DESTDIR=$(TARGET_DIR)  -C $(@D) install; \
	$(INSTALL) -D -m 755 $(CARDINAL_PKGDIR)/S99cardinal $(TARGET_DIR)/etc/init.d/S99cardinal
endef

$(eval $(generic-package))