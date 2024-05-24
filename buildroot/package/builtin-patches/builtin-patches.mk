BUILTIN_PATCHES_VERSION = 0.0.1
BUILTIN_PATCHES_SITE = $(BR2_EXTERNAL_KBS_PATH)/package/builtin-patches
BUILTIN_PATCHES_SITE_METHOD = local
BUILTIN_PATCHES_INSTALL_TARGET = YES

define BUILTIN_PATCHES_INSTALL_TARGET_CMDS
    mkdir -p $(TARGET_DIR)/usr/share/patches
    cp -dpf $(BUILTIN_PATCHES_PKGDIR)/*.vcv $(TARGET_DIR)/usr/share/patches
endef

$(eval $(generic-package))