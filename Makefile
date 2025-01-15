
##@ Build

.PHONY: build
build: ## Build cim binaries.
	@cargo build --release

.PHONY: dev
dev: ## Build cim binaries.
	@cargo build --release

.PHONY: fmt
fmt: ## fmt projects
	# @cargo fmt -- --check
	@cargo fmt

##@ Generate

.PHONY: cli
cli: ## install database cli
	@cargo install sqlx-cli --no-default-features --features rustls,mysql

##@ Test and Lint

.PHONY: test
test: ## Test go code.
	@cargo test

.PHONY: check
check: ## check rust code
	@cargo check --all

.PHONY: clippy
clippy: ## run rust linter
	@cargo clippy

.PHONY: rs
rs: build ## run server with docker
	docker build -f Dockerfile.server -t server:latest . && \
	docker run -itd -p 30050:30050 --restart=always --name server server:latest

##@ Clean
clean: ## Delete all builds
	@cargo clean

.PHONY: ds
ds: ## delete server from docker
	docker rm server -f && \
	docker rmi server

FORMATTING_BEGIN_YELLOW = \033[0;33m
FORMATTING_BEGIN_BLUE = \033[36m
FORMATTING_END = \033[0m

.PHONY: help
help:
	@awk 'BEGIN {\
	    FS = ":.*##"; \
	    printf                "Usage: ${FORMATTING_BEGIN_BLUE}OPTION${FORMATTING_END}=<value> make ${FORMATTING_BEGIN_YELLOW}<target>${FORMATTING_END}\n"\
	  } \
	  /^[a-zA-Z0-9_-]+:.*?##/ { printf "  ${FORMATTING_BEGIN_BLUE}%-46s${FORMATTING_END} %s\n", $$1, $$2 } \
	  /^.?.?##~/              { printf "   %-46s${FORMATTING_BEGIN_YELLOW}%-46s${FORMATTING_END}\n", "", substr($$1, 6) } \
	  /^##@/                  { printf "\n\033[1m%s\033[0m\n", substr($$0, 5) } ' $(MAKEFILE_LIST)
