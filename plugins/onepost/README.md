# ONEPOST — datové schránky

Propojení AI agenta s **ONEPOSTem** pro práci s datovými schránkami (ISDS). Agent umí pracovat se schránkami, složkami, zprávami, přílohami a koncepty, vyhledávat ve zprávách, stahovat nové zprávy z ISDS a odesílat připravené koncepty.

## Nastavení

Nastavte API klíč přes aplikaci **ONEPOST API klíč**:

1. Otevřete aplikaci ONEPOST API klíč.
2. Klikněte na **Otevřít ONEPOST** a v profilu vygenerujte klíč.
3. Zvolte typ přístupu — **Pouze čtení** pro prohlížení dat, nebo **Čtení a akce** pro operace jménem uživatele.
4. Vložte klíč do pole a uložte — agent ho ihned ověří.

Klíč se ukládá lokálně do konfigurace pluginu a kdykoli ho můžete přepsat nebo smazat.

## Co plugin umí

- Přehled datových schránek a složek
- Čtení zpráv a jejich příloh
- Vyhledávání ve zprávách
- Stažení a příjem nových zpráv z ISDS (vyžaduje přístup *Čtení a akce*)
- Nahrání přílohy pro koncept, vytvoření/editace konceptu a odeslání konceptu (vyžaduje přístup *Čtení a akce*)