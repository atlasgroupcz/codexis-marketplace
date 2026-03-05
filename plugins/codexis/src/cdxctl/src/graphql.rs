// ==================== Automations ====================

pub const GET_AUTOMATIONS: &str = r#"
query GetAutomations {
    automations {
        id
        uuid
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
            uuid
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
        uuid
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
        uuid
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
        uuid
        automationUuid
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
        name
        description
        source { source url path ref }
        pluginCount
        error
    }
}
"#;

pub const REMOVE_MARKETPLACE: &str = r#"
mutation RemoveMarketplace($name: String!) {
    removeMarketplace(name: $name) {
        name
    }
}
"#;

pub const UPDATE_MARKETPLACE: &str = r#"
mutation UpdateMarketplace($marketplace: String!) {
    updateMarketplace(marketplace: $marketplace) {
        marketplace
        previousCommit
        newCommit
        status
        error
        updatedPlugins { name version marketplace }
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
        updatedPlugins { name version marketplace }
    }
}
"#;

// ==================== Plugins ====================

pub const GET_INSTALLED_PLUGINS: &str = r#"
query GetInstalledPlugins($marketplace: String!) {
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
query GetAvailablePlugins($marketplace: String) {
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
