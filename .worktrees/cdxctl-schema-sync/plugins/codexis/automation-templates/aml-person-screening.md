---
uuid: 3bb4e9bf-6e71-4d02-b01a-c814c0a85f9b
type: AGENT
title: AML screening of a person
description: Screen a given person from public sources — PEP status, sanctions and adverse media.
cron: 0 7 1 * *
max-turns: 25
enabled: false
notify-on-success: true
notify-on-failure: true
web-search: true
i18n:
  cs:
    title: "AML prověření osoby"
    description: "Prověří zadanou osobu z veřejných zdrojů — PEP, sankce a negativní zprávy."
  en:
    title: "AML screening of a person"
    description: "Screen a given person from public sources — PEP status, sanctions and adverse media."
  sk:
    title: "AML preverenie osoby"
    description: "Preverí zadanú osobu z verejných zdrojov — PEP, sankcie a negatívne správy."
---
Proveď AML prověření osoby: «UPRAVTE — jméno a příjmení; ideálně i datum narození, zemi a funkci/roli».

Na internetu z veřejných zdrojů zjisti a stručně shrň:

1. **Identifikace** — kdo osoba je (funkce, působení, vazby na firmy), aby bylo jasné, že jde o správnou osobu a ne o někoho se stejným jménem.
2. **Politicky exponovaná osoba (PEP)** — zda jde o politicky exponovanou osobu nebo osobu jí blízkou (rodina, spolupracovníci). Uveď konkrétní funkci a období.
3. **Sankce a seznamy** — zda je osoba na sankčních seznamech (EU, OSN, OFAC) nebo jiných watchlistech.
4. **Negativní zprávy** — zmínky v médiích o podvodu, korupci, trestním stíhání nebo jiné finanční trestné činnosti.
5. **Shrnutí rizika** — krátké celkové zhodnocení (nízké / střední / vysoké riziko) s odůvodněním.

Pravidla:
- U každého zjištění uveď zdroj (odkaz), aby šlo ověřit. Rozlišuj potvrzené informace a jen možné shody (např. shoda jmen).
- Pokud k některému bodu nic nenajdeš, napiš to jednou větou.
- Piš česky, jednoduše a stručně.

_Toto je informativní rešerše z veřejných zdrojů, ne závazné AML/KYC posouzení. Možné shody ověřte v oficiálních sankčních seznamech a PEP databázích._
