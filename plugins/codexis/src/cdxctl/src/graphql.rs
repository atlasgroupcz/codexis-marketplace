// ==================== Automations ====================

pub const GET_AUTOMATIONS: &str = r#"
query GetAutomations {
    automations {
        id
        type
        command
        title
        description
        agent { id fullName }
        skills { id fullName }
        schedule { repeat dayPart daysOfWeek dayOfMonth }
        prompt
        enabled
        maxTurns
        workDirPath { absolutePath displayPath }
        lastRun {
            id
            status
            startedAt
            finishedAt
            trigger
            errorMessage
        }
    }
}
"#;

pub const CREATE_AUTOMATION: &str = r#"
mutation CreateAutomation($input: AutomationInput!) {
    createAutomation(input: $input) {
        id
        type
        command
        title
        description
        agent { id fullName }
        skills { id fullName }
        schedule { repeat dayPart daysOfWeek dayOfMonth }
        prompt
        enabled
        maxTurns
        workDirPath { absolutePath displayPath }
    }
}
"#;

pub const UPDATE_AUTOMATION: &str = r#"
mutation UpdateAutomation($id: ID!, $input: AutomationInput!) {
    updateAutomation(id: $id, input: $input) {
        id
        type
        command
        title
        description
        agent { id fullName }
        skills { id fullName }
        schedule { repeat dayPart daysOfWeek dayOfMonth }
        prompt
        enabled
        maxTurns
        workDirPath { absolutePath displayPath }
    }
}
"#;

// Toggle only the `enabled` flag. Unlike `updateAutomation` (USER-only, rejects
// read-only carried/plugin automations), the daemon's `setAutomationEnabled`
// works for both USER and carried automations - for carried ones it persists a
// per-user enabled override. This is how a shipped plugin automation is disabled.
pub const SET_AUTOMATION_ENABLED: &str = r#"
mutation SetAutomationEnabled($id: ID!, $enabled: Boolean!) {
    setAutomationEnabled(id: $id, enabled: $enabled) {
        id
        title
        enabled
    }
}
"#;

pub const DELETE_AUTOMATION: &str = r#"
mutation DeleteAutomation($id: ID!) {
    deleteNode(id: $id) {
        id
        ... on Automation { title }
    }
}
"#;

pub const TRIGGER_AUTOMATION: &str = r#"
mutation TriggerAutomation($id: ID!) {
    triggerAutomation(id: $id) {
        id
        lastRun { id status startedAt trigger }
    }
}
"#;

// ==================== Codexis 1.0 sync ====================

pub const START_CODEXIS_IMPORT: &str = r#"
mutation StartCodexisImport {
    startCodexisImport {
        runId
        status
        snapshotId
        chatsImported
        chatsSkipped
        messagesImported
        agentsImported
        agentsSkipped
        filesImported
        error
    }
}
"#;

pub const CODEXIS_IMPORT_STATUS: &str = r#"
query CodexisImportStatus {
    codexisImportStatus {
        runId
        status
        snapshotId
        chatsImported
        chatsSkipped
        messagesImported
        agentsImported
        agentsSkipped
        filesImported
        error
    }
}
"#;

// ==================== Marketplaces ====================

pub const GET_MARKETPLACES: &str = r#"
query GetMarketplaces {
    marketplaces {
        id
        name
        description
        source { source url path ref }
        installLocation { absolutePath displayPath }
        lastUpdated
        pluginCount
        error
        plugins {
            id
            name
            description
            version
            category
            tags
        }
    }
}
"#;

pub const ADD_MARKETPLACE: &str = r#"
mutation AddMarketplace($input: MarketplaceSourceInput!) {
    addMarketplace(input: $input) {
        id
        name
        description
        source { source url path ref }
        pluginCount
        error
    }
}
"#;

pub const REMOVE_MARKETPLACE: &str = r#"
mutation RemoveMarketplace($id: ID!) {
    deleteNode(id: $id) {
        id
        ... on Marketplace { name }
    }
}
"#;

pub const UPDATE_MARKETPLACE: &str = r#"
mutation UpdateMarketplace($id: ID!) {
    updateMarketplace(id: $id) {
        marketplace
        previousCommit
        newCommit
        status
        error
        updatedPlugins { id name version marketplace { id name } }
    }
}
"#;

// ==================== Plugins ====================

pub const GET_INSTALLED_PLUGINS: &str = r#"
query GetInstalledPlugins {
    marketplaces(filter: { installState: INSTALLED }) {
        id
        name
        plugins {
            id
            name
            version
            description
            marketplace { id name }
            installation { installedAt installedVersion }
        }
    }
}
"#;

pub const GET_AVAILABLE_PLUGINS: &str = r#"
query GetAvailablePlugins {
    marketplaces {
        id
        name
        plugins {
            id
            name
            version
            description
            category
            tags
        }
    }
}
"#;

pub const INSTALL_PLUGIN: &str = r#"
mutation InstallPlugin($input: PluginInstallInput!) {
    installPlugin(input: $input) {
        id
        name
        version
        description
        marketplace { id name }
        installation { installedAt installedVersion }
    }
}
"#;

pub const UNINSTALL_PLUGIN: &str = r#"
mutation UninstallPlugin($input: PluginInstallInput!) {
    uninstallPlugin(input: $input) {
        id
        name
        version
        description
        marketplace { id name }
    }
}
"#;

// ==================== Agents ====================

pub const GET_AGENTS: &str = r#"
query GetAgents {
    agents {
        id
        name
        fullName
        description
        plugin { id name marketplace { id name } }
        tools { toolName displayName kind source }
        skills
        jurisdictions
        model
        maxTurns
        instructions
        disallowedTools { toolName displayName kind source }
        sourceKind
        editable
        deletable
        path { absolutePath displayPath }
        sourcePath { absolutePath displayPath }
    }
}
"#;

// The chat the shell runs in carries the model and jurisdictions actually in use, which
// is per chat -- unlike a stored preference, which two concurrent chats would share.
pub const GET_CHAT_CONTEXT: &str = r#"
query GetChatContext($id: ID!) {
    node(id: $id) {
        ... on Chat {
            id
            model
            jurisdictions
        }
    }
}
"#;

pub const CREATE_AGENT_MARKDOWN: &str = r#"
mutation CreateAgent($markdown: Markdown!) {
    createAgent(markdown: $markdown) {
        id
        name
        fullName
        description
        sourceKind
        editable
        deletable
        maxTurns
        tools { toolName }
        disallowedTools { toolName }
        path { absolutePath displayPath }
        sourcePath { absolutePath displayPath }
    }
}
"#;

pub const UPDATE_AGENT_MARKDOWN: &str = r#"
mutation UpdateAgent($id: ID!, $markdown: Markdown!) {
    updateAgent(id: $id, markdown: $markdown) {
        id
        name
        fullName
        description
        sourceKind
        editable
        deletable
        maxTurns
        tools { toolName }
        disallowedTools { toolName }
        path { absolutePath displayPath }
        sourcePath { absolutePath displayPath }
    }
}
"#;

pub const SAVE_AGENT: &str = r#"
mutation SaveAgent($id: ID, $input: AgentInput!) {
    saveAgent(id: $id, input: $input) {
        agent {
            id
            name
            fullName
            description
            plugin { id name }
            tools { toolName displayName kind source }
            skills
            jurisdictions
            model
            maxTurns
            instructions
            disallowedTools { toolName displayName kind source }
            sourceKind
            editable
            deletable
            path { absolutePath displayPath }
            sourcePath { absolutePath displayPath }
        }
        errors { field message allowedValues }
    }
}
"#;

pub const DELETE_AGENT: &str = r#"
mutation DeleteAgent($id: ID!) {
    deleteNode(id: $id) {
        id
        ... on Agent { name }
    }
}
"#;

// ==================== Skills ====================

pub const GET_SKILLS: &str = r#"
query GetSkills {
    skills {
        id
        name
        fullName
        description
        plugin { id name marketplace { id name } }
        allowedTools
        sourceKind
        editable
        deletable
        path { absolutePath displayPath }
        sourcePath { absolutePath displayPath }
    }
}
"#;

pub const CREATE_SKILL: &str = r#"
mutation CreateSkill($markdown: Markdown!) {
    createSkill(markdown: $markdown) {
        id
        name
        fullName
        description
        plugin { id name }
        allowedTools
        sourceKind
        editable
        deletable
        path { absolutePath displayPath }
        sourcePath { absolutePath displayPath }
    }
}
"#;

pub const UPDATE_SKILL: &str = r#"
mutation UpdateSkill($id: ID!, $markdown: Markdown!) {
    updateSkill(id: $id, markdown: $markdown) {
        id
        name
        fullName
        description
        plugin { id name }
        allowedTools
        sourceKind
        editable
        deletable
        path { absolutePath displayPath }
        sourcePath { absolutePath displayPath }
    }
}
"#;

pub const SAVE_SKILL: &str = r#"
mutation SaveSkill($id: ID, $input: SkillInput!) {
    saveSkill(id: $id, input: $input) {
        skill {
            id
            name
            fullName
            description
            allowedTools
            sourceKind
            editable
            deletable
            displayName
            summary
            path { absolutePath displayPath }
            sourcePath { absolutePath displayPath }
        }
        errors { field message allowedValues }
    }
}
"#;

pub const DELETE_SKILL: &str = r#"
mutation DeleteSkill($id: ID!) {
    deleteNode(id: $id) {
        id
        ... on Skill { name }
    }
}
"#;

// ==================== Notifications ====================

pub const NOTIFICATIONS_QUERY: &str = r#"
query Notifications {
    notifications {
        items {
            id
            message
            action
            link
            seen
            confirmed
            createdAt
        }
        totalItems
        unseenCount
    }
}
"#;

pub const CREATE_NOTIFICATION_MUTATION: &str = r#"
mutation CreateNotification($input: CreateNotificationInput!) {
    createNotification(input: $input) {
        id
        message
        action
        link
        seen
        confirmed
        createdAt
    }
}
"#;

pub const UPDATE_NOTIFICATION_STATE_MUTATION: &str = r#"
mutation UpdateNotificationState($input: UpdateNotificationStateInput!) {
    updateNotificationState(input: $input) {
        id
        message
        seen
        confirmed
    }
}
"#;

// ==================== Tabular Extraction ====================
//
// All tabular ops take a `TabularExtraction` Node id (encoded from the folder
// path) - see commands::tabular::folder_to_node_id.

pub const GET_TABULAR_EXTRACTION: &str = r#"
query GetTabularExtraction($id: ID!) {
    tabularExtraction(id: $id) {
        id
        status
        progress { totalFiles extractedFiles startedAt elapsedSeconds }
        columns {
            id
            name
            description
            __typename
        }
        rows {
            fileName
            status
            error
            cells {
                column { id name }
                ... on TextCellValue { text }
                ... on DateCellValue { date }
                ... on BooleanCellValue { checked }
                ... on NumberCellValue { number }
                ... on CurrencyCellValue { amount currencyCode }
                ... on ListCellValue { items }
                ... on TagCellValue { tag }
                ... on TagsCellValue { tags }
            }
        }
    }
}
"#;

pub const ADD_TEXT_COLUMN: &str = r#"
mutation AddTextTabularColumn($id: ID!, $name: String!, $description: String) {
    addTextTabularColumn(id: $id, name: $name, description: $description) {
        id
        columns { id name description __typename }
        status
    }
}
"#;

pub const ADD_BOOLEAN_COLUMN: &str = r#"
mutation AddBooleanTabularColumn($id: ID!, $name: String!, $description: String) {
    addBooleanTabularColumn(id: $id, name: $name, description: $description) {
        id
        columns { id name description __typename }
        status
    }
}
"#;

pub const ADD_DATE_COLUMN: &str = r#"
mutation AddDateTabularColumn($id: ID!, $name: String!, $description: String) {
    addDateTabularColumn(id: $id, name: $name, description: $description) {
        id
        columns { id name description __typename }
        status
    }
}
"#;

pub const ADD_NUMBER_COLUMN: &str = r#"
mutation AddNumberTabularColumn($id: ID!, $name: String!, $description: String) {
    addNumberTabularColumn(id: $id, name: $name, description: $description) {
        id
        columns { id name description __typename }
        status
    }
}
"#;

pub const ADD_CURRENCY_COLUMN: &str = r#"
mutation AddCurrencyTabularColumn($id: ID!, $name: String!, $description: String) {
    addCurrencyTabularColumn(id: $id, name: $name, description: $description) {
        id
        columns { id name description __typename }
        status
    }
}
"#;

pub const ADD_LIST_COLUMN: &str = r#"
mutation AddListTabularColumn($id: ID!, $name: String!, $description: String) {
    addListTabularColumn(id: $id, name: $name, description: $description) {
        id
        columns { id name description __typename }
        status
    }
}
"#;

pub const ADD_TAG_COLUMN: &str = r#"
mutation AddTagTabularColumn($id: ID!, $name: String!, $description: String, $options: [TagOptionInput!]!) {
    addTagTabularColumn(id: $id, name: $name, description: $description, options: $options) {
        id
        columns { id name description __typename }
        status
    }
}
"#;

pub const ADD_TAGS_COLUMN: &str = r#"
mutation AddTagsTabularColumn($id: ID!, $name: String!, $description: String, $options: [TagOptionInput!]!) {
    addTagsTabularColumn(id: $id, name: $name, description: $description, options: $options) {
        id
        columns { id name description __typename }
        status
    }
}
"#;

pub const REMOVE_TABULAR_COLUMN: &str = r#"
mutation RemoveTabularColumn($id: ID!, $columnId: ID!) {
    removeTabularColumn(id: $id, columnId: $columnId) {
        id
        columns { id name description __typename }
        status
    }
}
"#;

pub const START_TABULAR_EXTRACTION: &str = r#"
mutation StartTabularExtraction($id: ID!) {
    startTabularExtraction(id: $id) {
        id
        status
        progress { totalFiles extractedFiles startedAt elapsedSeconds }
        columns { id name description __typename }
    }
}
"#;

#[cfg(test)]
mod tests {
    use super::{
        CREATE_AGENT_MARKDOWN, CREATE_AUTOMATION, DELETE_AGENT, GET_AGENTS, GET_AUTOMATIONS,
        GET_CHAT_CONTEXT, GET_SKILLS, REMOVE_MARKETPLACE, SAVE_AGENT, SET_AUTOMATION_ENABLED,
        UPDATE_AGENT_MARKDOWN, UPDATE_AUTOMATION,
    };

    #[test]
    fn agent_queries_target_canonical_agent_api() {
        assert!(GET_AGENTS.contains("query GetAgents"));
        assert!(GET_AGENTS.contains("agents {"));
        assert!(SAVE_AGENT.contains("saveAgent(id: $id, input: $input)"));
        // The markdown path stays on the deprecated mutations so binaries already in a VM
        // keep working after the daemon ships saveAgent.
        assert!(CREATE_AGENT_MARKDOWN.contains("createAgent(markdown: $markdown)"));
        assert!(UPDATE_AGENT_MARKDOWN.contains("updateAgent(id: $id, markdown: $markdown)"));
        assert!(DELETE_AGENT.contains("deleteNode(id: $id)"));
        assert!(GET_CHAT_CONTEXT.contains("... on Chat {"));
    }

    #[test]
    fn agent_mutations_read_the_payload_wrapper() {
        // createAgent/updateAgent return a payload, so every agent field sits under
        // `agent` and the rejected input is reported in `errors`.
        for mutation in [SAVE_AGENT] {
            assert!(mutation.contains("agent {"));
            assert!(mutation.contains("errors { field message allowedValues }"));
        }
    }

    #[test]
    fn agent_queries_request_local_crud_metadata() {
        for query in [GET_AGENTS, SAVE_AGENT] {
            assert!(query.contains("sourceKind"));
            assert!(query.contains("editable"));
            assert!(query.contains("deletable"));
            assert!(query.contains("path { absolutePath displayPath }"));
            assert!(query.contains("sourcePath { absolutePath displayPath }"));
            assert!(query.contains("maxTurns"));
            assert!(query.contains("tools { toolName"));
            assert!(query.contains("disallowedTools { toolName"));
        }
    }

    #[test]
    fn set_automation_enabled_targets_dedicated_toggle_mutation() {
        // The enabled toggle must go through `setAutomationEnabled` (works for
        // carried/plugin automations), NOT `updateAutomation` (USER-only guard).
        assert!(SET_AUTOMATION_ENABLED.contains("setAutomationEnabled(id: $id, enabled: $enabled)"));
        assert!(SET_AUTOMATION_ENABLED.contains("$enabled: Boolean!"));
        assert!(!SET_AUTOMATION_ENABLED.contains("updateAutomation"));
    }

    #[test]
    fn marketplace_remove_uses_generic_delete_node() {
        assert!(REMOVE_MARKETPLACE.contains("deleteNode(id: $id)"));
        assert!(REMOVE_MARKETPLACE.contains("on Marketplace"));
    }

    #[test]
    fn automation_queries_select_agent_and_skills_as_nodes() {
        // Daemon schema exposes `agent: Agent` / `skills: [Skill!]!` (with id +
        // fullName), not the legacy flat `agentFullName` / `skillFullNames` fields.
        for query in [GET_AUTOMATIONS, CREATE_AUTOMATION, UPDATE_AUTOMATION] {
            assert!(query.contains("agent { id fullName }"));
            assert!(query.contains("skills { id fullName }"));
            assert!(!query.contains("agentFullName"));
            assert!(!query.contains("skillFullNames"));
        }
    }

    #[test]
    fn agent_and_skill_lists_surface_plugin_and_marketplace() {
        // Marketplace is reachable only through the owning plugin in the daemon
        // schema - Agent/Skill have no direct `marketplace` field.
        for query in [GET_AGENTS, GET_SKILLS] {
            assert!(query.contains("plugin { id name marketplace { id name } }"));
        }
    }
}
