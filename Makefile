.PHONY: fmt test prepare _ensure_ollama_create _ensure_ollama_model

fmt:
	cargo fmt
	cargo clippy --fix --allow-dirty

test: prepare
	cargo test

prepare:
	@$(MAKE) _ensure_ollama_create NAME=mario MODELFILE=ollama-rs/tests/model/Modelfile.mario
	@$(MAKE) _ensure_ollama_create NAME=test_model MODELFILE=ollama-rs/tests/model/Modelfile.test_model
	@$(MAKE) _ensure_ollama_model MODEL=llama2:latest
	@$(MAKE) _ensure_ollama_model MODEL=granite-code:3b
	@$(MAKE) _ensure_ollama_model MODEL=llava:latest

_ensure_ollama_create:
	@ollama list 2>/dev/null | awk 'NR>1 {print $$1}' | grep -qxF '$(if $(strip $(MODEL)),$(MODEL),$(NAME):latest)' || ollama create '$(NAME)' -f '$(MODELFILE)'

_ensure_ollama_model:
	@ollama list 2>/dev/null | awk 'NR>1 {print $$1}' | grep -qxF '$(MODEL)' || ollama pull '$(MODEL)'
