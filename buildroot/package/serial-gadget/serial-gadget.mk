SERIAL_GADGET_VERSION = 0.0.1
SERIAL_GADGET_SITE = $(BR2_EXTERNAL_KBS_PATH)/package/serial-gadget
SERIAL_GADGET_SITE_METHOD = local
SERIAL_GADGET_INSTALL_TARGET = YES
SERIAL_GADGET_DEPENDENCIES = keystation loadmodules

define SERIAL_GADGET_INSTALL_TARGET_CMDS
    grep -qE '^ttyGS0::' ${TARGET_DIR}/etc/inittab || \
	sed -i '/GENERIC_SERIAL/a\
ttyGS0::respawn:/sbin/getty -L  ttyGS0 0 vt100 # USB gadget console' ${TARGET_DIR}/etc/inittab; \
	echo "g-serial" > $(TARGET_DIR)/etc/modules.d/05serial;
endef

$(eval $(generic-package))