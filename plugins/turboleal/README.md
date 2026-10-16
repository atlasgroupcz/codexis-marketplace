## Co to je?

Doplněk pro Turboleal, hlídač české legislativy. Popíšete, čím se zabýváte, nebo
nahrajete smlouvy a interní předpisy, a Turboleal pomocí AI najde zákony a
paragrafy, které se vás týkají. Každé dvě hodiny je kontroluje v CODEXIS a při
novele, zrušení nebo chystané změně vytvoří upozornění.

## Nastavení

Doplněk potřebuje váš osobní API klíč z Turbolealu:

1. V Turbolealu otevřete v postranním panelu **API klíče** a vytvořte nový klíč.
2. Otevřete aplikaci **Turboleal - API klíč**, vložte klíč a uložte ho - klíč se
   hned ověří.

Klíč můžete vložit i přímo do konverzace, asistent ho ověří a uloží za vás. Klíč
je osobní, asistent tak vidí jen vaše sledované sady. Ukládá se přes platformu
(`cdxctl`), doplněk sám do souboru s klíčem nezapisuje. V Turbolealu ho můžete
kdykoli zneplatnit.

## Součásti pluginu

### Turboleal - API klíč (aplikace)
Vložení, ověření a odebrání osobního API klíče.

## Co v něm najdete?

### Sledované sady
Založení sady ze slovního zadání, jeho úprava a smazání. Stav ukazuje, jestli
už AI dohledala předpisy.

### Soubory
Nahrání souborů i celých složek. Citace v nich najde Hypertexter a AI je ověří.

### Legislativa a reference
Které předpisy a paragrafy sada sleduje a proč, s citací ze zadání nebo ze
souboru včetně strany.

### Upozornění
Změny sledovaných paragrafů s rozdílem znění a novelou, která je způsobila.
Vyřízené upozornění se dá potvrdit.

## Na co si dát pozor

**Smazání sady je nevratné.** Spolu s ní zmizí její soubory, reference a
upozornění. Agent má pokyn si mazání nechat potvrdit.

**Hledání předpisů chvíli trvá.** Zadání zpracuje AI zhruba do dvou minut,
soubory podle délky i několik minut. Do té doby je sada ve stavu zpracování.
