# Agent examples

Worked `cdxctl agent create` calls. Copy the shape, swap the content.

## Employment-law reviewer

```bash
mkdir -p ./.tmp
cat > ./.tmp/koncipient-pp.md <<'EOF'
Posuď zákonnost výpovědi ze strany zaměstnavatele.
Na vstupu dostaneš text výpovědi, pracovní smlouvu a popis okolností.
Vycházej ze zákoníku práce (§ 48–62) a judikatury NS.
Ověř výpovědní důvod, výpovědní dobu, doručení a ochrannou dobu.
Výstupem je tabulka: zjištění, ustanovení, riziko, doporučená náprava.
EOF

cdxctl agent create --name koncipient-pracovni-pravo \
  --description "Posuzuje zákonnost výpovědí podle zákoníku práce" \
  --display-name "Koncipient pro pracovní právo" \
  --summary "Posoudí zákonnost výpovědi a navrhne nápravu." \
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
  --display-name "Personalista pro pracovní smlouvy" \
  --summary "Připraví a zkontroluje pracovní smlouvu nového zaměstnance." \
  --instructions-file ./.tmp/personalista.md \
  --skill codexis --skill docx
```

## Tax memo writer

```bash
cdxctl agent create --name danovy-poradce-dph \
  --description "Odpovídá na dotazy k DPH a píše stručné daňové memo" \
  --display-name "Daňový poradce pro DPH" \
  --summary "Odpoví na dotaz k DPH a shrne ho do krátkého memo." \
  --instructions-file ./.tmp/dph.md \
  --skill codexis-dane
```

## Document summariser

```bash
cdxctl agent create --name shrnovac-dokumentu \
  --description "Shrne dlouhý dokument do jedné strany" \
  --display-name "Shrnovač dokumentů" \
  --summary "Z dlouhého dokumentu udělá přehledné shrnutí na jednu stranu." \
  --instructions-file ./.tmp/shrnovac.md \
  --skill pdf --skill docx
```

## Read-only reviewer (restricted tools)

```bash
cdxctl agent create --name kontrolor-jen-cteni \
  --description "Zkontroluje dokument a jen komentuje, nic nemění" \
  --display-name "Kontrolor (jen čtení)" \
  --summary "Přečte dokument a vrátí připomínky, soubory nemění." \
  --instructions-file ./.tmp/kontrolor.md \
  --skill codexis --skill docx \
  --tool readFile --tool grep
```

## Updating an agent

```bash
cdxctl agent update personalista-smlouvy \
  --summary "Připraví, zkontroluje a archivuje pracovní smlouvu." \
  --skill codexis --skill docx --skill pdf
```
