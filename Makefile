.PHONY: test-e2e test-e2e-changed test-e2e-only

# Full run — every plugin with acceptance/e2e/
test-e2e:
	pytest tests/test-plugin-e2e.py \
	  --daemon   "$${DAEMON_URL:?set DAEMON_URL}" \
	  --token    "$${SERVICE_JWT:?set SERVICE_JWT}" \
	  --git-url  "$${TEST_GIT_URL:?set TEST_GIT_URL to a remote the daemon can clone}" \
	  --git-ref  "$${TEST_GIT_REF:?set TEST_GIT_REF to the branch you just pushed}" \
	  --base-ref "$${TEST_BASE_REF:-origin/main}" \
	  -v $(PYTEST_EXTRA)

# Only plugins changed vs. --base-ref
test-e2e-changed:
	$(MAKE) test-e2e PYTEST_EXTRA="--changed $(PYTEST_EXTRA)"

# Only a specific plugin: make test-e2e-only PLUGIN=codexis
test-e2e-only:
	@[ -n "$(PLUGIN)" ] || (echo "usage: make test-e2e-only PLUGIN=<name>"; exit 2)
	$(MAKE) test-e2e PYTEST_EXTRA="--only $(PLUGIN) $(PYTEST_EXTRA)"
