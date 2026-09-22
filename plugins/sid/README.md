## Co to je?

Doplněk propojuje asistenta se systémem SID, kde má vaše organizace řízené interní
dokumenty. Asistent v nich hledá, čte je, stahuje soubory a hlídá, co máte potvrdit
jako přečtené.

## Nastavení

Doplněk potřebuje váš osobní API klíč:

1. Otevřete aplikaci **SID - API klíč**.
2. V SID přejděte do **Profil → API klíče** a klíč vygenerujte.
3. Vložte ho do pole a uložte - asistent klíč ihned ověří.

Klíč je osobní, takže asistent vidí přesně ty dokumenty a složky, na které máte právo vy.
Ukládá se přes platformu (`cdxctl`), doplněk sám do souboru s klíčem nezapisuje.

## Co plugin umí

- Fulltextové vyhledávání ve směrnicích, vnitřních předpisech a pokynech
- Čtení dokumentu, jeho metadat a stažení souboru
- Přehled složek, do kterých máte přístup
- Legislativa, o kterou se dokument opírá
- Přehled dokumentů, které máte potvrdit jako přečtené (samotné potvrzení se dělá v SID)
- Upozornění na novinky a změny v SID

## K čemu se to hodí v praxi?

- **„Co říká naše směrnice o..."** - odpověď z vašeho vnitřního předpisu, ne z internetu.
- **Compliance** - vidíte, co máte nepotvrzené, a vyřídíte to na jednom místě.
- **Onboarding** - nový kolega se v pravidlech zorientuje rychleji.
- **Návaznost na zákon** - dohledáte, o kterou legislativu se směrnice opírá.

## Součásti pluginu

### SID - API klíč (aplikace)
Vložení, ověření a správa osobního API klíče.

### Hlídač novinek
Doplněk s sebou přináší automatizaci, která pravidelně kontroluje novinky
a nepotvrzené dokumenty. Upozornění v aplikaci je zapnuté, e-mail vypnutý - obojí
i případné další adresáty přepnete v nastavení doplňku.

## Zdroj dat

SID - Systém interních dokumentů vaší organizace. Přístup je omezen vaším osobním
API klíčem.
