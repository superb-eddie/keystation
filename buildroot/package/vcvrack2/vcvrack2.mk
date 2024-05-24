VCVRACK2_VERSION = v2.5.2
VCVRACK2_SITE = git@github.com:VCVRack/Rack.git
VCVRACK2_SITE_METHOD = git
VCVRACK2_GIT_SUBMODULES = YES
VCVRACK2_DL_OPTS = --recurse-submodules
VCVRACK2_INSTALL_STAGING = NO
VCVRACK2_INSTALL_TARGET = YES
VCVRACK2_DEPENDENCIES = libgl libglfw
VCVRACK2_TARGET_ARCH = $(call qstrip,$(BR2_PACKAGE_VCVRACK2_TARGET_ARCH))

# $(eval $(autotools-package))
#  CROSS_COMPILE=$(TARGET_CROSS)
define VCVRACK2_BUILD_CMDS
    $(MAKE) $(TARGET_CONFIGURE_OPTS) CROSS_COMPILE=$(VCVRACK2_TARGET_ARCH) -C $(@D) dep
    $(MAKE) $(TARGET_CONFIGURE_OPTS) CROSS_COMPILE=$(VCVRACK2_TARGET_ARCH) -C $(@D)
endef

# define VCVRACK2_INSTALL_STAGING_CMDS
#     $(INSTALL) -D -m 0755 $(@D)/libfoo.a $(STAGING_DIR)/usr/lib/libfoo.a
#     $(INSTALL) -D -m 0644 $(@D)/foo.h $(STAGING_DIR)/usr/include/foo.h
#     $(INSTALL) -D -m 0755 $(@D)/libfoo.so* $(STAGING_DIR)/usr/lib
# endef
#
# define VCVRACK2_INSTALL_TARGET_CMDS
# endef

$(eval $(generic-package))