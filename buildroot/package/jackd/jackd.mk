JACKD_VERSION = 0.0.1
JACKD_SITE = $(BR2_EXTERNAL_KBS_PATH)/package/loadmodules
JACKD_SITE_METHOD = local
JACKD_INSTALL_TARGET = YES

define JACKD_INSTALL_TARGET_CMDS
	$(INSTALL) -D -m 755 $(JACKD_PKGDIR)/S05jackd $(TARGET_DIR)/etc/init.d/S05jackd;
endef

$(eval $(generic-package))