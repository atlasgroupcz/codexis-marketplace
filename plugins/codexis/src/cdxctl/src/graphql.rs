// ==================== Automations ====================

pub const GET_AUTOMATIONS: &str = include_str!("queries/automation_get.graphql");

pub const SAVE_AUTOMATION: &str = include_str!("queries/automation_save.graphql");

// Toggle only the `enabled` flag. Unlike `saveAutomation` (USER-only, rejects
// read-only carried/plugin automations), the daemon's `setAutomationEnabled`
// works for both USER and carried automations - for carried ones it persists a
// per-user enabled override. This is how a shipped plugin automation is disabled.
pub const SET_AUTOMATION_ENABLED: &str = include_str!("queries/automation_set_enabled.graphql");

pub const DELETE_AUTOMATION: &str = include_str!("queries/automation_delete.graphql");

pub const TRIGGER_AUTOMATION: &str = include_str!("queries/automation_trigger.graphql");

pub const GET_AUTOMATION_RUNS: &str = include_str!("queries/automation_runs.graphql");

pub const GET_AUTOMATION_CONVERSATION: &str =
    include_str!("queries/automation_conversation.graphql");

// The files stored on an automation live in its own isolation: the daemon mounts the
// automation's files directory read-write at `~`, so listing `~` of `Automation:<uuid>`
// is the automation's file list. Uploading goes through the REST files endpoint with the
// same isolation id - see commands::automation::add_files.
pub const LIST_AUTOMATION_FILES: &str = include_str!("queries/automation_files_list.graphql");

pub const REMOVE_AUTOMATION_FILE: &str = include_str!("queries/automation_files_remove.graphql");

// ==================== Marketplaces ====================

pub const GET_MARKETPLACES: &str = include_str!("queries/marketplace_get.graphql");

pub const ADD_MARKETPLACE: &str = include_str!("queries/marketplace_add.graphql");

pub const REMOVE_MARKETPLACE: &str = include_str!("queries/marketplace_remove.graphql");

pub const UPDATE_MARKETPLACE: &str = include_str!("queries/marketplace_update.graphql");

// ==================== Plugins ====================

pub const GET_INSTALLED_PLUGINS: &str = include_str!("queries/plugin_get_installed.graphql");

pub const GET_AVAILABLE_PLUGINS: &str = include_str!("queries/plugin_get_available.graphql");

pub const INSTALL_PLUGIN: &str = include_str!("queries/plugin_install.graphql");

pub const UNINSTALL_PLUGIN: &str = include_str!("queries/plugin_uninstall.graphql");

// ==================== Agents ====================

pub const GET_AGENTS: &str = include_str!("queries/agent_get.graphql");

// The chat the shell runs in carries the model, jurisdictions, project and connections
// actually in use, which is per chat -- unlike a stored preference, which two concurrent
// chats would share.
pub const GET_CHAT_CONTEXT: &str = include_str!("queries/chat_context_get.graphql");

// ==================== Projects and connections ====================

pub const GET_PROJECTS: &str = include_str!("queries/project_get.graphql");

// saveProject replaces the whole project, so a partial update carries the stored fields
// forward - including the desktop folders, which only the desktop app can register.
pub const SAVE_PROJECT: &str = include_str!("queries/project_save.graphql");

// Every source a chat or an automation can mount: cloud drives plus own and shared drives.
pub const GET_CONNECTIONS: &str = include_str!("queries/connection_get.graphql");

pub const SAVE_AGENT: &str = include_str!("queries/agent_save.graphql");

pub const DELETE_AGENT: &str = include_str!("queries/agent_delete.graphql");

// ==================== Skills ====================

pub const GET_SKILLS: &str = include_str!("queries/skill_get.graphql");

pub const SAVE_SKILL: &str = include_str!("queries/skill_save.graphql");

pub const DELETE_SKILL: &str = include_str!("queries/skill_delete.graphql");

// ==================== Notifications ====================

pub const NOTIFICATIONS_QUERY: &str = include_str!("queries/notification_list.graphql");

pub const CREATE_NOTIFICATION_MUTATION: &str = include_str!("queries/notification_create.graphql");

pub const UPDATE_NOTIFICATION_STATE_MUTATION: &str =
    include_str!("queries/notification_update_state.graphql");

// ==================== Tabular Extraction ====================
//
// All tabular ops take a `TabularExtraction` Node id (the folder path nested
// under its isolation scope) - see commands::tabular::folder_to_node_id.

pub const GET_TABULAR_EXTRACTION: &str = include_str!("queries/tabular_get.graphql");

pub const ADD_TEXT_COLUMN: &str = include_str!("queries/tabular_add_text_column.graphql");

pub const ADD_BOOLEAN_COLUMN: &str = include_str!("queries/tabular_add_boolean_column.graphql");

pub const ADD_DATE_COLUMN: &str = include_str!("queries/tabular_add_date_column.graphql");

pub const ADD_NUMBER_COLUMN: &str = include_str!("queries/tabular_add_number_column.graphql");

pub const ADD_CURRENCY_COLUMN: &str = include_str!("queries/tabular_add_currency_column.graphql");

pub const ADD_LIST_COLUMN: &str = include_str!("queries/tabular_add_list_column.graphql");

pub const ADD_TAG_COLUMN: &str = include_str!("queries/tabular_add_tag_column.graphql");

pub const ADD_TAGS_COLUMN: &str = include_str!("queries/tabular_add_tags_column.graphql");

pub const REMOVE_TABULAR_COLUMN: &str = include_str!("queries/tabular_remove_column.graphql");

pub const START_TABULAR_EXTRACTION: &str = include_str!("queries/tabular_start.graphql");

#[cfg(test)]
mod tests {
    use super::{
        DELETE_AGENT, GET_AGENTS, GET_AUTOMATIONS, GET_CHAT_CONTEXT, GET_PROJECTS, GET_SKILLS,
        REMOVE_MARKETPLACE, SAVE_AGENT, SAVE_AUTOMATION, SAVE_PROJECT, SET_AUTOMATION_ENABLED,
    };

    #[test]
    fn agent_queries_target_canonical_agent_api() {
        assert!(GET_AGENTS.contains("query GetAgents"));
        assert!(GET_AGENTS.contains("agents {"));
        assert!(SAVE_AGENT.contains("saveAgent(id: $id, input: $input)"));
        assert!(DELETE_AGENT.contains("deleteNode(id: $id)"));
        assert!(GET_CHAT_CONTEXT.contains("... on Chat {"));
        assert!(GET_CHAT_CONTEXT.contains("modelConfiguration { key value }"));
    }

    #[test]
    fn agent_mutations_read_the_payload_wrapper() {
        // saveAgent returns a payload, so every agent field sits under
        // `agent` and the rejected input is reported in `errors`.
        assert!(SAVE_AGENT.contains("agent {"));
        assert!(SAVE_AGENT.contains("errors { field message allowedValues }"));
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
        // carried/plugin automations), NOT `saveAutomation` (USER-only guard).
        assert!(SET_AUTOMATION_ENABLED.contains("setAutomationEnabled(id: $id, enabled: $enabled)"));
        assert!(SET_AUTOMATION_ENABLED.contains("$enabled: Boolean!"));
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
        for query in [GET_AUTOMATIONS, SAVE_AUTOMATION] {
            assert!(query.contains("agent { id fullName }"));
            assert!(query.contains("skills { id fullName }"));
            assert!(query.contains("model"));
            assert!(query.contains("modelConfiguration { key value }"));
            assert!(query.contains("jurisdictions"));
        }
    }

    #[test]
    fn save_automation_reads_the_payload_wrapper() {
        // saveAutomation returns a payload, so every automation field sits under
        // `automation` and the rejected input is reported in `errors` - mirroring saveAgent.
        assert!(SAVE_AUTOMATION.contains("saveAutomation(id: $id, input: $input)"));
        assert!(SAVE_AUTOMATION.contains("$id: ID, $input: AutomationInput!"));
        assert!(SAVE_AUTOMATION.contains("automation {"));
        assert!(SAVE_AUTOMATION.contains("errors { field message allowedValues }"));
    }

    #[test]
    fn save_project_reads_the_payload_wrapper() {
        // ProjectInputError carries no allowedValues, unlike the agent and skill ones.
        assert!(SAVE_PROJECT.contains("saveProject(id: $id, input: $input)"));
        assert!(SAVE_PROJECT.contains("$id: ID, $input: ProjectInput!"));
        assert!(SAVE_PROJECT.contains("project {"));
        assert!(SAVE_PROJECT.contains("errors { field message }"));
    }

    #[test]
    fn project_queries_select_both_folder_lists_in_full() {
        // ProjectInput replaces the whole project, so `project update` rebuilds the input from
        // what these documents return. A field missing here is a field the update would clear.
        for query in [GET_PROJECTS, SAVE_PROJECT] {
            assert!(query.contains(
                "desktopFolders { deviceId deviceName shareName folderName relativePath }"
            ));
            assert!(query.contains("driveFolders { driveId name }"));
            assert!(query.contains("instructions"));
            assert!(query.contains("directoryPath { absolutePath displayPath }"));
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
