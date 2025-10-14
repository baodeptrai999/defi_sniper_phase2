# Variables
CONFIG_PATH=/root/projects/yoshida/pump.fun-sniper-Rust/Config.toml
BLACKLIST_DIR=/root/projects/yoshida/pump.fun-sniper-Rust/src/assets/blacklist
IMAGE=pumpfunbot

# Run sniper_mode container
sniper_mode:
	docker run -d \
		--name sniper_mode_container \
		-v $(CONFIG_PATH):/app/Config.toml \
		-v $(BLACKLIST_DIR)/rugs.mint:/app/src/assets/blacklist/rugs.mint \
		-v $(BLACKLIST_DIR)/rugs.wallet:/app/src/assets/blacklist/rugs.wallet \
		-e CONFIG_PATH=/app/Config.toml \
		$(IMAGE) sniper_mode

# Run copy_mode container
copy_mode:
	docker run -d \
		--name copy_mode_container \
		-v $(CONFIG_PATH):/app/Config.toml \
		-v $(BLACKLIST_DIR)/rugs.mint:/app/src/assets/blacklist/rugs.mint \
		-v $(BLACKLIST_DIR)/rugs.wallet:/app/src/assets/blacklist/rugs.wallet \
		-e CONFIG_PATH=/app/Config.toml \
		$(IMAGE) copy_mode

# Run half_copy_mode container
half_copy_mode:
	docker run -d \
		--name half_copy_mode_container \
		-v $(CONFIG_PATH):/app/Config.toml \
		-v $(BLACKLIST_DIR)/rugs.mint:/app/src/assets/blacklist/rugs.mint \
		-v $(BLACKLIST_DIR)/rugs.wallet:/app/src/assets/blacklist/rugs.wallet \
		-e CONFIG_PATH=/app/Config.toml \
		$(IMAGE) half_copy_mode

# Stop sniper_mode container
stop_sniper:
	docker stop sniper_mode_container || true
	docker rm sniper_mode_container || true

# Stop copy_mode container
stop_copy:
	docker stop copy_mode_container || true
	docker rm copy_mode_container || true

# Stop half_copy_mode container
stop_half_copy:
	docker stop half_copy_mode_container || true
	docker rm half_copy_mode_container || true
