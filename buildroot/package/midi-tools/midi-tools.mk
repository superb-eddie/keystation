MIDI_TOOLS_VERSION = 0.0.1
MIDI_TOOLS_SITE = $(BR2_EXTERNAL_KBS_PATH)/package/midi-tools/source
MIDI_TOOLS_SITE_METHOD = local
MIDI_TOOLS_INSTALL_TARGET = YES
MIDI_TOOLS_DEPENDENCIES = host-rustc

define MIDI_TOOLS_CARGO_FETCH
# Hacky way to fetch deps
     cd $(MIDI_TOOLS_SRCDIR) && \
         PATH=$(HOST_DIR)/bin:$(PATH) $(PKG_CARGO_ENV) cargo fetch
endef

MIDI_TOOLS_POST_RSYNC_HOOKS += MIDI_TOOLS_CARGO_FETCH

$(eval $(cargo-package))