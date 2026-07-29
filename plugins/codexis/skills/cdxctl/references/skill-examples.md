# Skill examples

Worked `cdxctl skill create` calls. Copy the shape, swap the content. Write the body to a
temp file under `./.tmp/` and pass it with `--instructions-file`.

`--display-name`, `--summary` and `--example-prompt` take `lang:text` and repeat once per
language. `--example-prompt` repeats once per prompt as well, at most three per language,
in the order they should appear. It is optional on a skill - the AI reaches for a skill on
its own, where an agent is started by clicking a prompt. When you do give prompts, write
them concrete and directly runnable.

## Legal-research skill (jurisdiction-specific)

```bash
mkdir -p ./.tmp
cat > ./.tmp/vypoved-kontrola.md <<'EOF'
Posuď zákonnost výpovědi ze strany zaměstnavatele.
Na vstupu dostaneš text výpovědi, pracovní smlouvu a popis okolností.
Vycházej ze zákoníku práce (§ 48-62) a judikatury NS.
Ověř výpovědní důvod, výpovědní dobu, doručení a ochrannou dobu.
Výstupem je tabulka: zjištění, ustanovení, riziko, doporučená náprava.
EOF

cdxctl skill create --name vypoved-kontrola \
  --description "Use when the user needs a Czech employment dismissal reviewed for lawfulness under the zákoník práce; not for Slovak law." \
  --display-name cs:"Kontrola výpovědi" --display-name en:"Dismissal review" \
  --summary cs:"Posoudí zákonnost výpovědi a navrhne nápravu." \
  --summary en:"Judges a dismissal and proposes a fix." \
  --example-prompt cs:"Posuď zákonnost této výpovědi a navrhni nápravu." \
  --example-prompt cs:"Zkontroluj, zda byla u této výpovědi dodržena výpovědní doba." \
  --example-prompt en:"Judge whether this dismissal is lawful and propose a fix." \
  --example-prompt en:"Check whether the notice period was observed." \
  --jurisdiction CZ \
  --instructions-file ./.tmp/vypoved-kontrola.md
```

## Document skill

```bash
cat > ./.tmp/smlouva-shrnuti.md <<'EOF'
Shrň smlouvu do jedné strany.
Vypiš strany, předmět, cenu, dobu trvání a výpovědní podmínky.
Označ neobvyklá nebo riziková ujednání.
Výstup vytvoř ve Wordu.
EOF

cdxctl skill create --name smlouva-shrnuti \
  --description "Use when the user wants a contract summarised to one page with the key terms and risky clauses; document task, not legal research." \
  --display-name cs:"Shrnutí smlouvy" --display-name en:"Contract summary" \
  --summary cs:"Ze smlouvy udělá přehled klíčových podmínek na jednu stranu." \
  --summary en:"Turns a contract into a one-page overview of the key terms." \
  --instructions-file ./.tmp/smlouva-shrnuti.md
```

## Read-only skill (restricted tools)

```bash
cdxctl skill create --name kontrolor-jen-cteni \
  --description "Use when a document should be reviewed and commented on without changing any file." \
  --display-name cs:"Kontrolor (jen čtení)" --display-name en:"Reviewer (read-only)" \
  --summary cs:"Přečte dokument a vrátí připomínky, soubory nemění." \
  --summary en:"Reads a document and returns notes without changing files." \
  --allowed-tool readFile --allowed-tool grep \
  --instructions-file ./.tmp/kontrolor.md
```

## Updating a skill (keeps what you do not pass)

```bash
cdxctl skill update vypoved-kontrola \
  --summary cs:"Posoudí zákonnost výpovědi, doručení a navrhne nápravu."

# --example-prompt is per-language, and repeating it replaces that language's whole list -
# so pass every Czech prompt even when only one changes. English is not passed, so the
# stored English prompts stay.
cdxctl skill update vypoved-kontrola \
  --example-prompt cs:"Posuď zákonnost této výpovědi a navrhni nápravu." \
  --example-prompt cs:"Ověř, zda byla výpověď řádně doručena."
```
