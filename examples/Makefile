# Gating a build on secret availability.
#
# `kivro status` exits 3 when a required secret is missing, so it composes
# with make's own failure handling without any parsing.

RUN := kivro run --

.PHONY: check dev test release migrate

check:
	@kivro status --quiet

dev: check
	$(RUN) cargo watch -x run

test: check
	$(RUN) cargo test

release: check
	$(RUN) cargo build --release

# A different environment for one target.
migrate:
	kivro run --env production -- ./scripts/migrate.sh
