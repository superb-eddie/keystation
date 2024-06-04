KEY_FIRMWARE_VERSION = 0.0.1
KEY_FIRMWARE_SITE = $(BR2_EXTERNAL_KBS_PATH)/package/key-firmware
KEY_FIRMWARE_SITE_METHOD = local
KEY_FIRMWARE_INSTALL_TARGET = YES

KEY_FIRMWARE_BIN = keystation-firmware.elf
KEY_FIRMWARE_ARTIFACT_PATH = ${KSB_BUILD_DIR}/cargo-target/avr-atmega328p/debug/$(KEY_FIRMWARE_BIN)

define KEY_FIRMWARE_BUILD_CMDS
	KSB_NO_LOCK="buildroot" ${KSB} firmware build;
endef

define KEY_FIRMWARE_INSTALL_TARGET_CMDS
	mkdir -p $(TARGET_DIR)/usr/share/;
	$(INSTALL) -D -m 655 $(KEY_FIRMWARE_ARTIFACT_PATH) $(TARGET_DIR)/usr/share/$(KEY_FIRMWARE_BIN)

endef

$(eval $(generic-package))