# Agent examples

Worked `cdxctl agent create` calls. Copy the shape, swap the content.

`--display-name` and `--summary` take `lang:text` and repeat once per language - the
prompt's language and English.

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
  --instructions-file ./.tmp/kontrolor.md \
  --skill codexis --skill docx \
  --tool readFile --tool grep
```

## Updating an agent

```bash
# Adds a skill and reword the Czech summary; other languages and fields stay put.
cdxctl agent update personalista-smlouvy \
  --summary cs:"Připraví, zkontroluje a archivuje pracovní smlouvu." \
  --skill codexis --skill docx --skill pdf
```
