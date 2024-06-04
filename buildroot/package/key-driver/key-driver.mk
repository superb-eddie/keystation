KEY_DRIVER_VERSION = 0.0.1
KEY_DRIVER_SITE = $(BR2_EXTERNAL_KBS_PATH)/package/key-driver/source
KEY_DRIVER_SITE_METHOD = local
KEY_DRIVER_INSTALL_TARGET = YES
KEY_DRIVER_DEPENDENCIES = host-rustc key-firmware

define KEY_DRIVER_CARGO_FETCH
# Hacky way to fetch deps
     cd $(KEY_DRIVER_SRCDIR) && \
         PATH=$(HOST_DIR)/bin:$(PATH) $(PKG_CARGO_ENV) cargo fetch
endef

KEY_DRIVER_POST_RSYNC_HOOKS += KEY_DRIVER_CARGO_FETCH

$(eval $(cargo-package))