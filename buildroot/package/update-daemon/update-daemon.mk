UPDATE_DAEMON_VERSION = 0.0.1
UPDATE_DAEMON_SITE = $(BR2_EXTERNAL_KBS_PATH)/../system-components
UPDATE_DAEMON_SITE_METHOD = local
UPDATE_DAEMON_SUBDIR = update-daemon
UPDATE_DAEMON_INSTALL_TARGET = YES
UPDATE_DAEMON_DEPENDENCIES = host-rustc

define UPDATE_DAEMON_INSTALL_SERVICE
	$(INSTALL) -D -m 755 $(UPDATE_DAEMON_PKGDIR)/S05update-daemon $(TARGET_DIR)/etc/init.d/S05update-daemon
endef
UPDATE_DAEMON_POST_INSTALL_TARGET_HOOKS += UPDATE_DAEMON_INSTALL_SERVICE

UPDATE_DAEMON_POST_RSYNC_HOOKS += POST_RSYNC_CARGO_VENDOR

$(eval $(cargo-package))