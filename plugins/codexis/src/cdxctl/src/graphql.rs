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
    deleteNode(id: $id)
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
        owner { name email }
        metadata { pluginRoot version description }
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
    removeMarketplace(id: $id) {
        id
        name
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

// ==================== Skills ====================

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
    deleteNode(id: $id)
}
"#;

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
    deleteNode(id: $id)
}
"#;

#[cfg(test)]
mod tests {
    use super::{CREATE_AGENT, DELETE_AGENT, GET_AGENTS, UPDATE_AGENT};

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

pub const MARK_NOTIFICATIONS_SEEN_MUTATION: &str = r#"
    mutation MarkNotificationsSeen($ids: [ID!]!) {
        markNotificationsSeen(ids: $ids)
    }
"#;

pub const MARK_NOTIFICATION_CONFIRMED_MUTATION: &str = r#"
    mutation MarkNotificationConfirmed($id: ID!) {
        markNotificationConfirmed(id: $id)
    }
"#;

// ==================== Tabular Extraction ====================

pub const GET_TABULAR_EXTRACTION: &str = r#"
query GetTabularExtraction($folder: String!) {
    tabularExtraction(folder: $folder) {
        folder
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
mutation AddTextTabularColumn($folder: String!, $name: String!, $description: String) {
    addTextTabularColumn(folder: $folder, name: $name, description: $description) {
        folder
        columns { id name description __typename }
        status
    }
}
"#;

pub const ADD_BOOLEAN_COLUMN: &str = r#"
mutation AddBooleanTabularColumn($folder: String!, $name: String!, $description: String) {
    addBooleanTabularColumn(folder: $folder, name: $name, description: $description) {
        folder
        columns { id name description __typename }
        status
    }
}
"#;

pub const ADD_DATE_COLUMN: &str = r#"
mutation AddDateTabularColumn($folder: String!, $name: String!, $description: String) {
    addDateTabularColumn(folder: $folder, name: $name, description: $description) {
        folder
        columns { id name description __typename }
        status
    }
}
"#;

pub const ADD_NUMBER_COLUMN: &str = r#"
mutation AddNumberTabularColumn($folder: String!, $name: String!, $description: String) {
    addNumberTabularColumn(folder: $folder, name: $name, description: $description) {
        folder
        columns { id name description __typename }
        status
    }
}
"#;

pub const ADD_CURRENCY_COLUMN: &str = r#"
mutation AddCurrencyTabularColumn($folder: String!, $name: String!, $description: String) {
    addCurrencyTabularColumn(folder: $folder, name: $name, description: $description) {
        folder
        columns { id name description __typename }
        status
    }
}
"#;

pub const ADD_LIST_COLUMN: &str = r#"
mutation AddListTabularColumn($folder: String!, $name: String!, $description: String) {
    addListTabularColumn(folder: $folder, name: $name, description: $description) {
        folder
        columns { id name description __typename }
        status
    }
}
"#;

pub const ADD_TAG_COLUMN: &str = r#"
mutation AddTagTabularColumn($folder: String!, $name: String!, $description: String, $options: [TagOptionInput!]!) {
    addTagTabularColumn(folder: $folder, name: $name, description: $description, options: $options) {
        folder
        columns { id name description __typename }
        status
    }
}
"#;

pub const ADD_TAGS_COLUMN: &str = r#"
mutation AddTagsTabularColumn($folder: String!, $name: String!, $description: String, $options: [TagOptionInput!]!) {
    addTagsTabularColumn(folder: $folder, name: $name, description: $description, options: $options) {
        folder
        columns { id name description __typename }
        status
    }
}
"#;

pub const REMOVE_TABULAR_COLUMN: &str = r#"
mutation RemoveTabularColumn($folder: String!, $columnId: ID!) {
    removeTabularColumn(folder: $folder, columnId: $columnId) {
        folder
        columns { id name description __typename }
        status
    }
}
"#;

pub const START_TABULAR_EXTRACTION: &str = r#"
mutation StartTabularExtraction($folder: String!) {
    startTabularExtraction(folder: $folder) {
        folder
        status
        progress { totalFiles extractedFiles startedAt elapsedSeconds }
        columns { id name description __typename }
    }
}
"#;
