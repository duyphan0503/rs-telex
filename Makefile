PREFIX ?= /usr
DESTDIR ?=
HOME_DIR ?= $(HOME)

all: build

build:
	cargo build --release

test:
	cargo test

install: build
	install -Dm755 target/release/rs-telex $(DESTDIR)$(PREFIX)/libexec/ibus-engine-rs-telex
	install -Dm644 data/rs-telex.xml $(DESTDIR)$(PREFIX)/share/ibus/component/rs-telex.xml
	install -Dm644 data/*.gschema.xml $(DESTDIR)$(PREFIX)/share/glib-2.0/schemas/
	glib-compile-schemas $(DESTDIR)$(PREFIX)/share/glib-2.0/schemas/ || true
	ibus write-cache || true

uninstall:
	rm -f $(DESTDIR)$(PREFIX)/libexec/ibus-engine-rs-telex
	rm -f $(DESTDIR)$(PREFIX)/share/ibus/component/rs-telex.xml
	rm -f $(DESTDIR)$(PREFIX)/share/glib-2.0/schemas/org.freedesktop.ibus.engine.rs-telex.gschema.xml
	glib-compile-schemas $(DESTDIR)$(PREFIX)/share/glib-2.0/schemas/ || true
	ibus write-cache || true

install-user: build
	install -Dm755 target/release/rs-telex $(HOME_DIR)/.local/bin/ibus-engine-rs-telex
	mkdir -p $(HOME_DIR)/.local/share/ibus/component
	sed 's|/usr/libexec/ibus-engine-rs-telex|$(HOME_DIR)/.local/bin/ibus-engine-rs-telex|g' data/rs-telex.xml > $(HOME_DIR)/.local/share/ibus/component/rs-telex.xml
	mkdir -p $(HOME_DIR)/.local/share/glib-2.0/schemas
	install -Dm644 data/*.gschema.xml $(HOME_DIR)/.local/share/glib-2.0/schemas/
	glib-compile-schemas $(HOME_DIR)/.local/share/glib-2.0/schemas/ || true
	ibus write-cache || true
	@echo "Successfully installed rs-telex in user environment (~/.local)."

uninstall-user:
	rm -f $(HOME_DIR)/.local/bin/ibus-engine-rs-telex
	rm -f $(HOME_DIR)/.local/share/ibus/component/rs-telex.xml
	rm -f $(HOME_DIR)/.local/share/glib-2.0/schemas/org.freedesktop.ibus.engine.rs-telex.gschema.xml
	glib-compile-schemas $(HOME_DIR)/.local/share/glib-2.0/schemas/ || true
	ibus write-cache || true
	@echo "Successfully uninstalled rs-telex from user environment."

deb: build
	rm -rf target/deb-staging
	mkdir -p target/deb-staging/DEBIAN
	mkdir -p target/deb-staging/usr/libexec
	mkdir -p target/deb-staging/usr/share/ibus/component
	mkdir -p target/deb-staging/usr/share/glib-2.0/schemas
	cp debian/control target/deb-staging/DEBIAN/
	cp target/release/rs-telex target/deb-staging/usr/libexec/ibus-engine-rs-telex
	cp data/rs-telex.xml target/deb-staging/usr/share/ibus/component/
	cp data/*.gschema.xml target/deb-staging/usr/share/glib-2.0/schemas/
	dpkg-deb --build --root-owner-group target/deb-staging target/rs-telex_0.1.0_amd64.deb
	@echo "Built package target/rs-telex_0.1.0_amd64.deb"

clean:
	cargo clean
	rm -rf target/deb-staging

.PHONY: all build test install uninstall install-user uninstall-user deb clean
