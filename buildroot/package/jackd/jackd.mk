JACKD_VERSION = 0.0.1
JACKD_SITE = $(BR2_EXTERNAL_KBS_PATH)/package/jackd
JACKD_SITE_METHOD = local
JACKD_INSTALL_TARGET = YES

define JACKD_INSTALL_TARGET_CMDS
	$(INSTALL) -D -m 755 $(JACKD_PKGDIR)/S10jackd $(TARGET_DIR)/etc/init.d/S10jackd;
endef

$(eval $(generic-package))