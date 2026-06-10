---
uuid: 214caaa4-3728-4d21-b379-ab4b376b7615
name: ares
description: "Český ARES pro právní ověřování firem a podnikatelů: vyhledání podle názvu nebo IČO, základní identifikace, statutární orgány, způsob jednání, živnosti a compliance údaje."
version: 0.1.0
jurisdictions: [CZ]
i18n:
  cs:
    displayName: "ARES — ověřování ekonomických subjektů"
    summary: "Vyhledávání a ověřování českých firem a podnikatelů v ARES pro právní a compliance použití."
  en:
    displayName: "ARES — Czech Entity Verification"
    summary: "Verify Czech companies and entrepreneurs using ARES."
---

# ARES — ověřování ekonomických subjektů

Použij tuto dovednost, když uživatel chce ověřit českou firmu, podnikatele nebo jiný ekonomický subjekt podle názvu, IČO, statutárních orgánů, živností nebo compliance údajů.

Pracuj česky a používej českou právní a rejstříkovou terminologii: IČO, obchodní firma, sídlo, právní forma, statutární orgán, způsob jednání, živnostenské oprávnění, skutečný majitel.

## Nástroj

Používej pouze CLI:

```bash
ares <command>
```

Nepoužívej přímo `curl` ani jiné HTTP nástroje. CLI vrací JSON na stdout. Pokud CLI vypíše `ERROR: ...`, chybu stručně sděl uživateli a nevymýšlej chybějící údaje.

## Kdy spustit který příkaz

- Uživatel zná název nebo jen část názvu: `ares search "<dotaz>"`.
- Uživatel zná IČO a chce ověřit subjekt: `ares company <ico>`.
- Ptá se na jednatele, představenstvo, členy orgánu nebo podpisové oprávnění: `ares officers <ico>`.
- Ptá se na živnosti nebo předmět podnikání ze živnostenského rejstříku: `ares trades <ico>`.
- Ptá se na skutečného majitele, AML/KYC nebo compliance kontrolu: `ares owners <ico>`.
- Potřebuješ původní odpověď zdroje nebo ladíš nejasnost: `ares raw <ico> --source <source>`.

Podrobnosti příkazů jsou v `references/cli.md`.

## Jak odpovídat

V odpovědi vždy uveď obchodní firmu nebo jméno, IČO a zdroj dat, pokud je dostupný. U adresy preferuj textovou adresu vrácenou ARES. U statutárních orgánů odděl osobu/funkci od způsobu jednání.

Když je výsledek neúplný, napiš to výslovně. Nezaměňuj základní identifikaci za výpis z obchodního rejstříku a neprezentuj nepřítomnost údaje jako právní závěr.

## Doporučený postup

1. Pokud uživatel nemá IČO, nejdřív vyhledej kandidáty přes `ares search`.
2. Vyber pravděpodobný subjekt podle názvu, sídla a IČO; při nejasnosti ukaž kandidáty.
3. Pro právní ověření spusť detailní příkaz podle otázky.
4. Shrň výsledek česky, věcně a s upozorněním na limity veřejného zdroje.
