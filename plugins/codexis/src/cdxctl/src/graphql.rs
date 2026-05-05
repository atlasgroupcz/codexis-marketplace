// ==================== Automations ====================

pub const GET_AUTOMATIONS: &str = r#"
query GetAutomations {
    automations {
        id
        type
        command
        title
        description
        agentFullName
        skillFullNames
        cron
        prompt
        enabled
        maxTurns
        workDirPathInfo { absolutePath displayPath }
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
        agentFullName
        skillFullNames
        cron
        prompt
        enabled
        maxTurns
        workDirPathInfo { absolutePath displayPath }
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
        agentFullName
        skillFullNames
        cron
        prompt
        enabled
        maxTurns
        workDirPathInfo { absolutePath displayPath }
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
        status
        startedAt
        trigger
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
        updatedPlugins { id name version marketplace }
    }
}
"#;

pub const UPDATE_ALL_MARKETPLACES: &str = r#"
mutation UpdateAllMarketplaces {
    updateAllMarketplaces {
        marketplace
        previousCommit
        newCommit
        status
        error
        updatedPlugins { id name version marketplace }
    }
}
"#;

// ==================== Plugins ====================

pub const GET_INSTALLED_PLUGINS: &str = r#"
query GetInstalledPlugins($marketplace: ID!) {
    installedPlugins(marketplace: $marketplace) {
        id
        name
        version
        description
        homepage
        marketplace
        installLocation { absolutePath displayPath }
        installedAt
    }
}
"#;

pub const GET_AVAILABLE_PLUGINS: &str = r#"
query GetAvailablePlugins($marketplace: ID) {
    availablePlugins(marketplace: $marketplace) {
        id
        name
        version
        description
        homepage
        marketplace
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
        marketplace
        installedAt
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
        marketplace
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
        marketplace
        plugin
        tools
        skills
        model
        maxTurns
        disallowedTools
        sourceKind
        editable
        deletable
        pathInfo { absolutePath displayPath }
        sourcePath { absolutePath displayPath }
    }
}
"#;

pub const CREATE_AGENT: &str = r#"
mutation CreateAgent($markdown: String!) {
    createAgent(markdown: $markdown) {
        id
        name
        fullName
        description
        marketplace
        plugin
        tools
        skills
        model
        maxTurns
        disallowedTools
        sourceKind
        editable
        deletable
        pathInfo { absolutePath displayPath }
        sourcePath { absolutePath displayPath }
    }
}
"#;

pub const UPDATE_AGENT: &str = r#"
mutation UpdateAgent($id: ID!, $markdown: String!) {
    updateAgent(id: $id, markdown: $markdown) {
        id
        name
        fullName
        description
        marketplace
        plugin
        tools
        skills
        model
        maxTurns
        disallowedTools
        sourceKind
        editable
        deletable
        pathInfo { absolutePath displayPath }
        sourcePath { absolutePath displayPath }
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
        marketplace
        plugin
        allowedTools
        sourceKind
        editable
        deletable
        pathInfo { absolutePath displayPath }
        sourcePath { absolutePath displayPath }
    }
}
"#;

pub const CREATE_SKILL: &str = r#"
mutation CreateSkill($markdown: String!) {
    createSkill(markdown: $markdown) {
        id
        name
        fullName
        description
        marketplace
        plugin
        allowedTools
        sourceKind
        editable
        deletable
        pathInfo { absolutePath displayPath }
        sourcePath { absolutePath displayPath }
    }
}
"#;

pub const UPDATE_SKILL: &str = r#"
mutation UpdateSkill($id: ID!, $markdown: String!) {
    updateSkill(id: $id, markdown: $markdown) {
        id
        name
        fullName
        description
        marketplace
        plugin
        allowedTools
        sourceKind
        editable
        deletable
        pathInfo { absolutePath displayPath }
        sourcePath { absolutePath displayPath }
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

#[cfg(test)]
mod tests {
    use super::{
        CREATE_AGENT, DELETE_AGENT, GET_AGENTS, REMOVE_MARKETPLACE, UPDATE_AGENT,
    };

    #[test]
    fn agent_queries_target_canonical_agent_api() {
        assert!(GET_AGENTS.contains("query GetAgents"));
        assert!(GET_AGENTS.contains("agents {"));
        assert!(CREATE_AGENT.contains("createAgent(markdown: $markdown)"));
        assert!(UPDATE_AGENT.contains("updateAgent(id: $id, markdown: $markdown)"));
        assert!(DELETE_AGENT.contains("deleteNode(id: $id)"));
    }

    #[test]
    fn agent_queries_request_local_crud_metadata() {
        for query in [GET_AGENTS, CREATE_AGENT, UPDATE_AGENT] {
            assert!(query.contains("sourceKind"));
            assert!(query.contains("editable"));
            assert!(query.contains("deletable"));
            assert!(query.contains("pathInfo { absolutePath displayPath }"));
            assert!(query.contains("sourcePath { absolutePath displayPath }"));
            assert!(query.contains("maxTurns"));
            assert!(query.contains("disallowedTools"));
        }
    }

    #[test]
    fn marketplace_remove_uses_generic_delete_node() {
        assert!(REMOVE_MARKETPLACE.contains("deleteNode(id: $id)"));
        assert!(REMOVE_MARKETPLACE.contains("on Marketplace"));
    }
}

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
// path) — see commands::tabular::folder_to_node_id.

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
