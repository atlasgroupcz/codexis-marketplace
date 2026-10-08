# Assistant examples

Worked `cdxctl agent create` calls. Copy the shape, swap the content.

`--display-name`, `--summary` and `--example-prompt` take `lang:text` and repeat once
per language - the prompt's language and English.

Every assistant gets **three `--example-prompt` per language**, in the order they should
appear. They are what the user clicks to start a run, so each is a concrete, directly
runnable sentence asking the assistant for one of its jobs - never a category label, never
a `«fill this in»` placeholder. Cover three different jobs rather than one question
reworded.

## Employment-law reviewer

```bash
mkdir -p ./.tmp
cat > ./.tmp/koncipient-pp.md <<'EOF'
Posuď zákonnost výpovědi ze strany zaměstnavatele.
Na vstupu dostaneš text výpovědi, pracovní smlouvu a popis okolností.
Vycházej ze zákoníku práce (§ 48-62) a judikatury NS.
Ověř výpovědní důvod, výpovědní dobu, doručení a ochrannou dobu.
Výstupem je tabulka: zjištění, ustanovení, riziko, doporučená náprava.
EOF

cdxctl agent create --name koncipient-pracovni-pravo \
  --description "Posuzuje zákonnost výpovědí podle zákoníku práce" \
  --display-name cs:"Koncipient pro pracovní právo" \
  --display-name en:"Employment-law associate" \
  --summary cs:"Posoudí zákonnost výpovědi a navrhne nápravu." \
  --summary en:"Judges a dismissal and proposes a fix." \
  --example-prompt cs:"Posuď zákonnost této výpovědi a navrhni nápravu." \
  --example-prompt cs:"Zkontroluj, zda byla u této výpovědi dodržena výpovědní doba." \
  --example-prompt cs:"Shrň rizika této výpovědi do tabulky." \
  --example-prompt en:"Judge whether this dismissal is lawful and propose a fix." \
  --example-prompt en:"Check whether the notice period was observed." \
  --example-prompt en:"Summarize the risks of this dismissal in a table." \
  --instructions-file ./.tmp/koncipient-pp.md \
  --skill codexis
```

## HR contract preparer

```bash
cat > ./.tmp/personalista.md <<'EOF'
Připrav a zkontroluj pracovní smlouvu nového zaměstnance.
Na vstupu dostaneš údaje o pozici, mzdě a nástupu.
Vycházej ze zákoníku práce a ověř povinné náležitosti smlouvy.
Označ rizikové nebo neplatné ujednání a navrhni opravu.
Hotovou smlouvu vytvoř ve Wordu.
EOF

cdxctl agent create --name personalista-smlouvy \
  --description "Připravuje a kontroluje pracovní smlouvy podle zákoníku práce" \
  --display-name cs:"Personalista pro pracovní smlouvy" \
  --display-name en:"HR contract preparer" \
  --summary cs:"Připraví a zkontroluje pracovní smlouvu nového zaměstnance." \
  --summary en:"Prepares and checks a new hire's employment contract." \
  --example-prompt cs:"Připrav pracovní smlouvu pro nového zaměstnance na pozici účetní." \
  --example-prompt cs:"Zkontroluj tuto pracovní smlouvu a označ riziková ujednání." \
  --example-prompt cs:"Doplň do smlouvy konkurenční doložku podle zákoníku práce." \
  --example-prompt en:"Draft an employment contract for a new accountant." \
  --example-prompt en:"Review this employment contract and flag risky clauses." \
  --example-prompt en:"Add a non-compete clause under the Czech Labour Code." \
  --instructions-file ./.tmp/personalista.md \
  --skill codexis --skill docx
```

## Tax memo writer

```bash
cdxctl agent create --name danovy-poradce-dph \
  --description "Odpovídá na dotazy k DPH a píše stručné daňové memo" \
  --display-name cs:"Daňový poradce pro DPH" \
  --display-name en:"VAT advisor" \
  --summary cs:"Odpoví na dotaz k DPH a shrne ho do krátkého memo." \
  --summary en:"Answers a VAT question and sums it up in a short memo." \
  --example-prompt cs:"Napiš memo k DPH u přeúčtování nákladů mezi spřízněnými firmami." \
  --example-prompt cs:"Kdy vzniká povinnost registrace k DPH při prodeji do EU?" \
  --example-prompt cs:"Shrň, jak se uplatní odpočet DPH u firemního automobilu." \
  --example-prompt en:"Write a VAT memo on recharging costs between related companies." \
  --example-prompt en:"When does selling into the EU trigger VAT registration?" \
  --example-prompt en:"Summarize how VAT deduction works for a company car." \
  --instructions-file ./.tmp/dph.md \
  --skill codexis-dane
```

## Document summariser

```bash
cdxctl agent create --name shrnovac-dokumentu \
  --description "Shrne dlouhý dokument do jedné strany" \
  --display-name cs:"Shrnovač dokumentů" \
  --display-name en:"Document summariser" \
  --summary cs:"Z dlouhého dokumentu udělá přehledné shrnutí na jednu stranu." \
  --summary en:"Turns a long document into a one-page summary." \
  --example-prompt cs:"Shrň tento dokument na jednu stranu." \
  --example-prompt cs:"Vypiš z tohoto dokumentu klíčové termíny a lhůty." \
  --example-prompt cs:"Porovnej tyto dvě verze dokumentu a popiš rozdíly." \
  --example-prompt en:"Summarize this document in one page." \
  --example-prompt en:"List the key dates and deadlines in this document." \
  --example-prompt en:"Compare these two versions and describe the differences." \
  --instructions-file ./.tmp/shrnovac.md \
  --skill pdf --skill docx
```

## Read-only reviewer (restricted tools)

```bash
cdxctl agent create --name kontrolor-jen-cteni \
  --description "Zkontroluje dokument a jen komentuje, nic nemění" \
  --display-name cs:"Kontrolor (jen čtení)" \
  --display-name en:"Reviewer (read-only)" \
  --summary cs:"Přečte dokument a vrátí připomínky, soubory nemění." \
  --summary en:"Reads a document and returns notes without changing files." \
  --example-prompt cs:"Zkontroluj tento dokument a vrať připomínky." \
  --example-prompt cs:"Najdi v tomto dokumentu nesrovnalosti a rozpory." \
  --example-prompt cs:"Posuď, zda dokument obsahuje všechny povinné náležitosti." \
  --example-prompt en:"Review this document and return your notes." \
  --example-prompt en:"Find inconsistencies and contradictions in this document." \
  --example-prompt en:"Check whether the document has every mandatory element." \
  --instructions-file ./.tmp/kontrolor.md \
  --skill codexis --skill docx \
  --tool readFile --tool grep
```

## Updating an assistant

```bash
# Adds the pdf skill: --skill replaces the WHOLE list, so repeat the skills the
# assistant already has. The Czech summary is replaced; other languages and fields stay put.
cdxctl agent update personalista-smlouvy \
  --summary cs:"Připraví, zkontroluje a archivuje pracovní smlouvu." \
  --skill codexis --skill docx --skill pdf

# --example-prompt is per-language too, and repeating it replaces that language's whole
# list - so pass all three Czech prompts even when only one of them changes. English is
# not passed here, so the stored English prompts stay.
cdxctl agent update personalista-smlouvy \
  --example-prompt cs:"Připrav pracovní smlouvu pro nového zaměstnance na pozici účetní." \
  --example-prompt cs:"Zkontroluj tuto pracovní smlouvu a označ riziková ujednání." \
  --example-prompt cs:"Ulož hotovou smlouvu do PDF."
```
