set dotenv-load := false

# configurable paths
bin_dir := env_var_or_default("BIN_DIR", "~/.local/bin")
app_dir := env_var_or_default("APP_DIR", "~/.local/share/applications")
icon_dir := env_var_or_default("ICON_DIR", "~/.local/share/icons/hicolor/scalable/apps")

# default recipe
_default:
	@just --list

# Build release binary
build:
	cargo build --release

# Install for current user
install: build
	install -Dm755 target/release/cosmic-bluetooth-gamepad {{bin_dir}}/cosmic-bluetooth-gamepad
	install -Dm644 resources/com.keewee.CosmicBluetoothGamepad.desktop {{app_dir}}/com.keewee.CosmicBluetoothGamepad.desktop
	install -Dm644 resources/icon.svg {{icon_dir}}/cosmic-bluetooth-gamepad.svg
	update-desktop-database {{app_dir}} || true
	gtk-update-icon-cache -f ~/.local/share/icons/hicolor || true

# Uninstall for current user
uninstall:
	rm -f {{bin_dir}}/cosmic-bluetooth-gamepad
	rm -f {{app_dir}}/com.keewee.CosmicBluetoothGamepad.desktop
	rm -f {{icon_dir}}/cosmic-bluetooth-gamepad.svg
	update-desktop-database {{app_dir}} || true
	gtk-update-icon-cache -f ~/.local/share/icons/hicolor || true

# Clean build artifacts
clean:
	cargo clean
