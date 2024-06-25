KEY_FIRMWARE_VERSION = 0.0.1
KEY_FIRMWARE_SITE = $(BR2_EXTERNAL_KBS_PATH)/../key-firmware
KEY_FIRMWARE_SITE_METHOD = local
KEY_FIRMWARE_INSTALL_TARGET = YES

KEY_FIRMWARE_PROFILE = release
KEY_FIRMWARE_BIN = key-firmware.elf
KEY_FIRMWARE_ARTIFACT_PATH = ${KSB_BUILD_DIR}/cargo-target/avr-atmega328p/$(KEY_FIRMWARE_PROFILE)/

define KEY_FIRMWARE_BUILD_CMDS
	KSB_NO_LOCK="buildroot" ${KSB} firmware build --profile $(KEY_FIRMWARE_PROFILE);
endef

define KEY_FIRMWARE_INSTALL_TARGET_CMDS
	mkdir -p $(TARGET_DIR)/usr/share/;
	$(INSTALL) -D -m 655 $(KEY_FIRMWARE_ARTIFACT_PATH)/$(KEY_FIRMWARE_BIN) $(TARGET_DIR)/usr/share/$(KEY_FIRMWARE_BIN)
	$(INSTALL) -D -m 655 $(KEY_FIRMWARE_ARTIFACT_PATH)/key-firmware-version.txt $(TARGET_DIR)/usr/share/key-firmware-version.txt
endef

$(eval $(generic-package))