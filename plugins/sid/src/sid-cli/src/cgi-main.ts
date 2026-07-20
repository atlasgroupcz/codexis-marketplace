#!/usr/bin/env node
/**
 * Entrypoint CGI komponenty. Veškerá logika je v `cgi.ts` — tenhle soubor
 * existuje jen proto, aby `cgi.ts` šel importovat v testech bez toho, aby
 * se při importu spustil požadavek (a zablokoval se na čtení stdin).
 */
import { serve } from "./cgi"

serve().then((out) => process.stdout.write(out))
