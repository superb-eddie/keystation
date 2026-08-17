MDEV_CONF_VERSION = 0.0.1
MDEV_CONF_SITE = $(BR2_EXTERNAL_KBS_PATH)/package/mdev-conf
MDEV_CONF_SITE_METHOD = local
MDEV_CONF_INSTALL_TARGET = YES
KEYBOARD_DAEMON_DEPENDENCIES = mdev

define MDEV_CONF_INSTALL_TARGET_CMDS
	$(INSTALL) -D -m 755 $(MDEV_CONF_PKGDIR)/usb-tty.sh $(TARGET_DIR)/lib/mdev/usb-tty.sh; \
	sed -i -r 's?^(ttyUSB.*)$$?\1 @/lib/mdev/usb-tty.sh?' $(TARGET_DIR)/etc/mdev.conf;
endef

$(eval $(generic-package))