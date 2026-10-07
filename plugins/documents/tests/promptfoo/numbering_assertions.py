import re
from html import unescape
from xml.etree import ElementTree

NAMESPACE = "{http://schemas.openxmlformats.org/wordprocessingml/2006/main}"


def assert_numbering(output: str, context: dict) -> bool:
    answer = output.split("<<CDX_EVAL_TOOL_CALLS>>")[0].strip()
    if re.match(r"<(?:p|pre|code)(?:\s|>)", answer):
        answer = unescape(re.sub(r"</?(?:p|pre|code)(?:\s[^>]*)?>", "", answer)).strip()
    answer = re.sub(r"^```(?:xml)?\s*|\s*```$", "", answer).strip()
    try:
        root = ElementTree.fromstring(answer)
    except ElementTree.ParseError:
        return False
    if root.tag != f"{NAMESPACE}numbering":
        return False
    instances = root.findall(f"{NAMESPACE}num")
    definitions = root.findall(f"{NAMESPACE}abstractNum")
    if len(instances) != 1 or len(definitions) != 1:
        return False
    instance, definition = instances[0], definitions[0]
    if (
        not instance.get(f"{NAMESPACE}numId")
        or instance.get(f"{NAMESPACE}numId") == "0"
    ):
        return False
    expected_num_id = context["vars"].get("numbering_num_id")
    if expected_num_id is not None and instance.get(f"{NAMESPACE}numId") != str(
        expected_num_id
    ):
        return False
    reference = instance.find(f"{NAMESPACE}abstractNumId")
    definition_id = definition.get(f"{NAMESPACE}abstractNumId")
    if (
        not definition_id
        or reference is None
        or reference.get(f"{NAMESPACE}val") != definition_id
    ):
        return False
    if instance.findall(f"{NAMESPACE}lvlOverride"):
        return False
    expected_levels = context["vars"]["numbering_levels"]
    levels = definition.findall(f"{NAMESPACE}lvl")
    if len(levels) != len(expected_levels):
        return False
    for index, expected in enumerate(expected_levels):
        matching = [
            level for level in levels if level.get(f"{NAMESPACE}ilvl") == str(index)
        ]
        if len(matching) != 1:
            return False
        for name, expected_value in expected.items():
            properties = matching[0].findall(f"{NAMESPACE}{name}")
            if expected_value is None:
                if properties:
                    return False
            elif len(properties) != 1 or properties[0].get(f"{NAMESPACE}val") != str(
                expected_value
            ):
                return False
    return True
