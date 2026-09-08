import { FakeCdxctl } from "./test-support"

/**
 * Every test runs with a working fake `cdxctl` on hand.
 *
 * Storing a setting spawns `cdxctl appenv`, and plenty of tests store one as a
 * side effect (caching the user's role, say) without caring how. Without this
 * they would depend on whatever `cdxctl` the machine happens to have.
 */
new FakeCdxctl()
