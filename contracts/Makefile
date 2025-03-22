ifneq (,$(wildcard ./.env))
	include .env
	export
endif

.PHONY: test
test: 
	forge test -vvvv

.PHONY: deploy
deploy:
	forge script script/DeployObsidianRouter.s.sol:DeployObsidianRouter --rpc-url $(ETH_RPC_URL) -vvvv  --broadcast

