"""Scope the global skill listing to a single marketplace plugin instance.

`skills` (the GraphQL query) is global: it lists every skill across every
installed plugin, regardless of which marketplace the plugin came from. When the
same plugin is installed twice - once by the test's marketplace and once by the
onboarding preset (a different marketplace) - the same skill `fullName` appears
twice, each carrying its own `plugin.id`. Both ids are `NodeIds.root(PLUGIN,
pluginKey)` and the pluginKey is the marketplace-scoped plugin id, so filtering
on `plugin.id` keeps only the copy from the marketplace we are testing.
"""

from __future__ import annotations

from collections.abc import Iterable


def skills_of_plugin(skills: Iterable[dict], plugin_id: str) -> set[str]:
    """Full names of the skills provided by the plugin with this exact id.

    Skills whose `plugin` is null (LOCAL skills, or ones whose owning plugin is
    gone) and skills from any other plugin instance are excluded.
    """
    result: set[str] = set()
    for skill in skills:
        plugin = skill.get("plugin") or {}
        if plugin.get("id") == plugin_id:
            result.add(skill["fullName"])
    return result
