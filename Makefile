.PHONY: build
build:
	git pull
	cargo build --release --target x86_64-unknown-linux-gnu

.PHONY: deploy
deploy: build
	@scp target/x86_64-unknown-linux-gnu/release/dashboard rbas@nabu:/srv/kathistiko/dashboard/
	@scp config.sample.toml rbas@nabu:/srv/kathistiko/dashboard/
	@scp public/css/main.css  rbas@nabu:/srv/kathistiko/dashboard/public/css/
	@echo "Restarting service on remote server..."
	ssh rbas@nabu 'sudo -S systemctl restart kathistikodashboard.service'
