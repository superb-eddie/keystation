define POST_RSYNC_CARGO_VENDOR
     cd $(@D) && \
     mkdir -p .cargo/ &&\
     PATH=$(HOST_DIR)/bin:$(PATH) $(PKG_CARGO_ENV) cargo vendor --versioned-dirs --locked ${KSB_CARGO_VENDOR_DIR} > .cargo/config.toml
endef

include $(sort $(wildcard $(BR2_EXTERNAL_KBS_PATH)/package/*/*.mk))