#!/usr/bin/env node
/**
 * Component entrypoint. All logic lives in `cgi.ts`; this file exists so that
 * importing it in tests doesn't execute a request and block reading stdin.
 */
import { serve } from "./cgi"

serve().then((out) => process.stdout.write(out))
