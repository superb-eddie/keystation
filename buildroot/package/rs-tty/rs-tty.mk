RS_TTY_VERSION = 0.0.1
RS_TTY_SITE = $(BR2_EXTERNAL_KBS_PATH)/../rs-tty
RS_TTY_SITE_METHOD = local
RS_TTY_DEPENDENCIES = host-rustc

define RS_TTY_CARGO_FETCH
# Hacky way to fetch deps
     cd $(RS_TTY_SRCDIR) && \
         PATH=$(HOST_DIR)/bin:$(PATH) $(PKG_CARGO_ENV) cargo fetch
endef

RS_TTY_POST_RSYNC_HOOKS += RS_TTY_CARGO_FETCH

define RS_TTY_INSTALL_TARGET_CMDS
	echo ""
endef

$(eval $(cargo-package))