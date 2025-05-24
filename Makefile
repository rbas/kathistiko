.PHONY: build
build:
	git pull
	cargo build --release

.PHONY: deploy
deploy: build
	@scp target/release/dashboard rbas@nabu:/srv/kathistiko/dashboard/
	@scp config.sample.toml rbas@nabu:/srv/kathistiko/dashboard/
	@scp public/css/main.css  rbas@nabu:/srv/kathistiko/dashboard/public/css/
	@echo "Restarting service on remote server..."
	ssh rbas@nabu 'sudo -S systemctl restart kathistikodashboard.service'
