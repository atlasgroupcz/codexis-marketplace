"""Normalize raw ARES responses into Claude-friendly JSON shapes."""

from __future__ import annotations

from typing import Any


def search_summary(raw: dict[str, Any], *, echo: dict[str, Any]) -> dict[str, Any]:
    """Return candidate rows used by `ares search`."""
    candidates = raw.get("ekonomickeSubjekty") or []
    return _compact(
        {
            "echo": echo,
            "typVystupu": "kandidati",
            "pocetCelkem": raw.get("pocetCelkem"),
            "pocetVraceno": len(candidates),
            "kandidati": [_candidate(entity) for entity in candidates],
        }
    )


def company_summary(raw: dict[str, Any], *, echo: dict[str, Any]) -> dict[str, Any]:
    """Return the compact identification shape used by `ares company`."""
    return _compact(
        {
            "echo": echo,
            "typVystupu": "kartaSubjektu",
            "kartaSubjektu": {
                "nazev": raw.get("obchodniJmeno"),
                "ico": raw.get("ico"),
                "icoId": raw.get("icoId"),
                "dic": raw.get("dic"),
                "dicSkDph": raw.get("dicSkDph"),
                "sidlo": _address_text(raw.get("sidlo")),
                "sidloDetail": raw.get("sidlo"),
                "pravniForma": raw.get("pravniForma"),
                "pravniFormaRos": raw.get("pravniFormaRos"),
                "datumVzniku": raw.get("datumVzniku"),
                "datumZaniku": raw.get("datumZaniku"),
                "datumAktualizace": raw.get("datumAktualizace"),
                "primarniZdroj": raw.get("primarniZdroj"),
                "registrace": raw.get("seznamRegistraci"),
                "czNace": raw.get("czNace"),
                "czNace2008": raw.get("czNace2008"),
            },
        }
    )


def officers_summary(raw: dict[str, Any], *, echo: dict[str, Any]) -> dict[str, Any]:
    """Return public-register governance data for `ares officers`."""
    records = raw.get("zaznamy") or []
    return _compact(
        {
            "echo": echo,
            "typVystupu": "verejnyRejstrik",
            "icoId": raw.get("icoId"),
            "zaznamy": [_vr_record(record) for record in records],
        }
    )


def trades_summary(raw: dict[str, Any], *, echo: dict[str, Any]) -> dict[str, Any]:
    """Return trade-licence data for `ares trades`."""
    records = raw.get("zaznamy") or []
    return _compact(
        {
            "echo": echo,
            "typVystupu": "zivnostenskyRejstrik",
            "icoId": raw.get("icoId"),
            "zaznamy": [_rzp_record(record) for record in records],
        }
    )


def _candidate(entity: dict[str, Any]) -> dict[str, Any]:
    return _compact(
        {
            "nazev": entity.get("obchodniJmeno"),
            "ico": entity.get("ico"),
            "sidlo": _address_text(entity.get("sidlo")),
            "pravniForma": entity.get("pravniForma"),
            "pravniFormaRos": entity.get("pravniFormaRos"),
            "primarniZdroj": entity.get("primarniZdroj"),
            "stavRegistraci": entity.get("seznamRegistraci"),
        }
    )


def _vr_record(record: dict[str, Any]) -> dict[str, Any]:
    return _compact(
        {
            "obchodniJmeno": _values(record.get("obchodniJmeno")),
            "ico": _values(record.get("ico")),
            "rejstrik": record.get("rejstrik"),
            "primarniZaznam": record.get("primarniZaznam"),
            "stavSubjektu": record.get("stavSubjektu"),
            "datumVzniku": _values(record.get("datumVzniku")),
            "datumZapisu": record.get("datumZapisu"),
            "datumVymazu": record.get("datumVymazu"),
            "datumAktualizace": record.get("datumAktualizace"),
            "spisovaZnacka": [_spisova_znacka(item) for item in _as_list(record.get("spisovaZnacka"))],
            "zakladniKapital": [_money_record(item) for item in _as_list(record.get("zakladniKapital"))],
            "statutarniOrgany": [_organ(item) for item in _as_list(record.get("statutarniOrgany"))],
            "zpusobJednani": _texts_from_organs(record.get("statutarniOrgany")),
            "spolecnici": [_spolecnici(item) for item in _as_list(record.get("spolecnici"))],
            "akcionari": [_organ(item) for item in _as_list(record.get("akcionari"))],
            "exekuce": _texts(record.get("exekuce")),
            "insolvence": record.get("insolvence"),
            "konkursy": record.get("konkursy"),
        }
    )


def _rzp_record(record: dict[str, Any]) -> dict[str, Any]:
    return _compact(
        {
            "nazev": record.get("obchodniJmeno"),
            "ico": record.get("ico"),
            "sidlo": _address_text(record.get("sidlo")),
            "pravniForma": record.get("pravniForma"),
            "datumVzniku": record.get("datumVzniku"),
            "datumZaniku": record.get("datumZaniku"),
            "datumAktualizace": record.get("datumAktualizace"),
            "typSubjektu": record.get("typSubjektu"),
            "zivnostenskyUrad": record.get("zivnostenskyUrad"),
            "zivnostiStav": record.get("zivnostiStav"),
            "provozovnyStav": record.get("provozovnyStav"),
            "insolvencniRizeni": record.get("insolvencniRizeni"),
            "zivnosti": [_zivnost(item) for item in _as_list(record.get("zivnosti"))],
        }
    )


def _organ(organ: dict[str, Any]) -> dict[str, Any]:
    return _compact(
        {
            "nazevOrganu": organ.get("nazevOrganu"),
            "typOrganu": organ.get("typOrganu"),
            "typAkcionare": organ.get("typAkcionare"),
            "datumZapisu": organ.get("datumZapisu"),
            "datumVymazu": organ.get("datumVymazu"),
            "pocetClenu": organ.get("pocetClenu"),
            "clenove": [_engagement(item) for item in _as_list(organ.get("clenoveOrganu"))],
            "zpusobJednani": _texts(organ.get("zpusobJednani")),
        }
    )


def _engagement(engagement: dict[str, Any]) -> dict[str, Any]:
    membership = engagement.get("clenstvi") or {}
    clenstvi = membership.get("clenstvi") or {}
    function = membership.get("funkce") or {}
    return _compact(
        {
            "osoba": _person_or_entity(engagement),
            "typAngazma": engagement.get("typAngazma"),
            "nazevAngazma": engagement.get("nazevAngazma"),
            "funkce": function.get("nazev"),
            "vznikFunkce": function.get("vznikFunkce"),
            "zanikFunkce": function.get("zanikFunkce"),
            "vznikClenstvi": clenstvi.get("vznikClenstvi"),
            "zanikClenstvi": clenstvi.get("zanikClenstvi"),
            "datumZapisu": engagement.get("datumZapisu"),
            "datumVymazu": engagement.get("datumVymazu"),
            "textZaOsobu": membership.get("textZaOsobu"),
            "textZruseni": membership.get("textZruseni"),
        }
    )


def _spolecnici(group: dict[str, Any]) -> dict[str, Any]:
    return _compact(
        {
            "nazevOrganu": group.get("nazevOrganu"),
            "typOrganu": group.get("typOrganu"),
            "datumZapisu": group.get("datumZapisu"),
            "datumVymazu": group.get("datumVymazu"),
            "spolecnici": [_spolecnik(item) for item in _as_list(group.get("spolecnik"))],
            "spolecnyPodil": group.get("spolecnyPodil"),
            "uvolnenyPodil": group.get("uvolnenyPodil"),
        }
    )


def _spolecnik(spolecnik: dict[str, Any]) -> dict[str, Any]:
    return _compact(
        {
            "osoba": _engagement(spolecnik.get("osoba") or {}),
            "podil": spolecnik.get("podil"),
            "datumZapisu": spolecnik.get("datumZapisu"),
            "datumVymazu": spolecnik.get("datumVymazu"),
        }
    )


def _zivnost(zivnost: dict[str, Any]) -> dict[str, Any]:
    return _compact(
        {
            "predmetPodnikani": zivnost.get("predmetPodnikani"),
            "druhZivnosti": zivnost.get("druhZivnosti"),
            "datumVzniku": zivnost.get("datumVzniku"),
            "datumZaniku": zivnost.get("datumZaniku"),
            "platnostDo": zivnost.get("platnostDo"),
            "datumAktualizace": zivnost.get("datumAktualizace"),
            "pozastaveni": zivnost.get("pozastaveniZivnosti"),
            "preruseni": zivnost.get("preruseniZivnosti"),
            "podminky": zivnost.get("podminkyProvozovaniZivnosti"),
            "obory": [_compact({"nazev": item.get("oborNazev"), "platnostOd": item.get("platnostOd"), "platnostDo": item.get("platnostDo")}) for item in _as_list(zivnost.get("oboryCinnosti"))],
            "odpovedniZastupci": [_rzp_person(item) for item in _as_list(zivnost.get("odpovedniZastupci"))],
            "provozovny": [_provozovna(item) for item in _as_list(zivnost.get("provozovny"))],
        }
    )


def _provozovna(provozovna: dict[str, Any]) -> dict[str, Any]:
    return _compact(
        {
            "icp": provozovna.get("icp"),
            "nazev": provozovna.get("nazev"),
            "sidlo": _address_text(provozovna.get("sidloProvozovny")),
            "umisteni": provozovna.get("umisteniProvozovny"),
            "typProvozovny": provozovna.get("typProvozovny"),
            "platnostOd": provozovna.get("platnostOd"),
            "platnostDo": provozovna.get("platnostDo"),
            "pozastaveni": provozovna.get("pozastaveniProvozovny"),
            "obory": [_compact({"nazev": item.get("oborNazev"), "platnostOd": item.get("platnostOd"), "platnostDo": item.get("platnostDo")}) for item in _as_list(provozovna.get("oboryCinnosti"))],
        }
    )


def _person_or_entity(payload: dict[str, Any]) -> dict[str, Any] | str | None:
    if payload.get("fyzickaOsoba"):
        return _vr_person(payload["fyzickaOsoba"])
    if payload.get("pravnickaOsoba"):
        entity = payload["pravnickaOsoba"]
        return _compact(
            {
                "typ": "pravnickaOsoba",
                "nazev": entity.get("obchodniJmeno"),
                "ico": entity.get("ico"),
                "pravniForma": entity.get("pravniForma"),
                "adresa": _address_text(entity.get("adresa")),
            }
        )
    if payload.get("skrytyUdaj"):
        return _text(payload["skrytyUdaj"])
    return None


def _vr_person(person: dict[str, Any]) -> dict[str, Any]:
    return _compact(
        {
            "typ": "fyzickaOsoba",
            "jmeno": _join(
                person.get("titulPredJmenem"),
                person.get("jmeno"),
                person.get("prijmeni"),
                person.get("titulZaJmenem"),
            ),
            "datumNarozeni": person.get("datumNarozeni"),
            "statniObcanstvi": person.get("statniObcanstvi"),
            "adresa": _address_text(person.get("adresa") or person.get("bydliste")),
            "textOsoba": person.get("textOsoba"),
        }
    )


def _rzp_person(person: dict[str, Any]) -> dict[str, Any]:
    return _compact(
        {
            "jmeno": _join(
                person.get("titulPredJmenem"),
                person.get("jmeno"),
                person.get("prijmeni"),
                person.get("titulZaJmenem"),
            ),
            "datumNarozeni": person.get("datumNarozeni"),
            "typAngazma": person.get("typAngazma"),
            "statniObcanstvi": person.get("statniObcanstvi"),
            "platnostOd": person.get("platnostOd"),
            "platnostDo": person.get("platnostDo"),
        }
    )


def _money_record(record: dict[str, Any]) -> dict[str, Any]:
    return _compact(
        {
            "typJmeni": record.get("typJmeni"),
            "text": record.get("text"),
            "vklad": _money(record.get("vklad")),
            "splaceni": _money(record.get("splaceni")),
            "datumZapisu": record.get("datumZapisu"),
            "datumVymazu": record.get("datumVymazu"),
        }
    )


def _money(value: dict[str, Any] | None) -> str | dict[str, Any] | None:
    if not value:
        return None
    if value.get("hodnota") and value.get("typObnos"):
        return f"{value['hodnota']} {value['typObnos']}"
    return _compact(value)


def _spisova_znacka(item: dict[str, Any]) -> dict[str, Any]:
    label = _join(item.get("oddil"), item.get("vlozka"), item.get("soud"))
    return _compact(
        {
            "znacka": label,
            "soud": item.get("soud"),
            "oddil": item.get("oddil"),
            "vlozka": item.get("vlozka"),
            "datumZapisu": item.get("datumZapisu"),
            "datumVymazu": item.get("datumVymazu"),
        }
    )


def _texts_from_organs(organs: Any) -> list[str]:
    result: list[str] = []
    for organ in _as_list(organs):
        result.extend(_texts(organ.get("zpusobJednani")))
    return result


def _texts(items: Any) -> list[str]:
    return [text for text in (_text(item) for item in _as_list(items)) if text]


def _text(item: Any) -> str | None:
    if isinstance(item, dict):
        return item.get("hodnota") or item.get("popis") or item.get("text")
    if isinstance(item, str):
        return item
    return None


def _values(items: Any) -> list[Any]:
    return [item.get("hodnota") for item in _as_list(items) if isinstance(item, dict) and item.get("hodnota")]


def _address_text(address: Any) -> str | None:
    if not isinstance(address, dict):
        return None
    if address.get("textovaAdresa"):
        return address["textovaAdresa"]
    return _join(
        address.get("nazevUlice"),
        address.get("cisloDoAdresy") or address.get("cisloDomovni"),
        address.get("nazevObce"),
        address.get("psc") or address.get("pscTxt"),
    )


def _join(*parts: Any) -> str | None:
    result = " ".join(str(part).strip() for part in parts if part not in (None, ""))
    return result or None


def _as_list(value: Any) -> list[Any]:
    if value is None:
        return []
    if isinstance(value, list):
        return value
    return [value]


def _compact(value: Any) -> Any:
    if isinstance(value, dict):
        compacted = {key: _compact(item) for key, item in value.items()}
        return {key: item for key, item in compacted.items() if item not in (None, "", [], {})}
    if isinstance(value, list):
        return [item for item in (_compact(item) for item in value) if item not in (None, "", [], {})]
    return value
