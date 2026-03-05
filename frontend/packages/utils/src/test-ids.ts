export const TEST_ID = {
  USER_AVATAR_TRIGGER: 'user-avatar-trigger',
  LANGUAGE_SWITCHER_TRIGGER: 'language-switcher-trigger',
  LANGUAGE_OPTION: (code: string) => `language-option-${code}`,
  THEME_TOGGLE: 'theme-toggle',

  // Marketplace page
  MARKETPLACE_PAGE: 'marketplace-page',
  MARKETPLACE_ADD_BUTTON: 'marketplace-add-button',
  MARKETPLACE_EMPTY_STATE: 'marketplace-empty-state',

  // Add marketplace dialog
  MARKETPLACE_ADD_DIALOG: 'marketplace-add-dialog',
  MARKETPLACE_ADD_GIT_URL: 'marketplace-add-git-url',
  MARKETPLACE_ADD_SUBMIT: 'marketplace-add-submit',

  // Marketplace card
  MARKETPLACE_CARD: (name: string) => `marketplace-card-${name}`,
  MARKETPLACE_CARD_TRIGGER: (name: string) => `marketplace-card-trigger-${name}`,
  MARKETPLACE_CARD_CONTENT: (name: string) => `marketplace-card-content-${name}`,
  MARKETPLACE_REMOVE_BUTTON: (name: string) => `marketplace-remove-${name}`,
  MARKETPLACE_REMOVE_CONFIRM: 'marketplace-remove-confirm',

  // Plugin items
  PLUGIN_ITEM: (name: string) => `plugin-item-${name}`,
  PLUGIN_INSTALL_BUTTON: (name: string) => `plugin-install-${name}`,
  PLUGIN_UNINSTALL_BUTTON: (name: string) => `plugin-uninstall-${name}`,
  PLUGIN_INSTALLED_BADGE: (name: string) => `plugin-installed-badge-${name}`,

  // Plugin agent definitions
  PLUGIN_AGENTS_TRIGGER: (name: string) => `plugin-agents-trigger-${name}`,
  PLUGIN_AGENTS_CONTENT: (name: string) => `plugin-agents-content-${name}`,
  PLUGIN_AGENT_ITEM: (pluginName: string, agentName: string) => `plugin-agent-${pluginName}-${agentName}`,

  // Plugin skill definitions
  PLUGIN_SKILLS_CONTENT: (name: string) => `plugin-skills-content-${name}`,
  PLUGIN_SKILL_ITEM: (pluginName: string, skillName: string) => `plugin-skill-${pluginName}-${skillName}`,

  // Sidebar — plugin apps
  SIDEBAR_APP_ITEM: (pluginId: string, componentId: string) =>
    `sidebar-app-${pluginId}-${componentId}`,

  // Plugin apps page
  PLUGIN_APPS_IFRAME: 'plugin-apps-iframe',

  // File explorer
  FILE_EXPLORER_PAGE: 'file-explorer-page',
  FILE_EXPLORER_BREADCRUMB: 'file-explorer-breadcrumb',
  FILE_EXPLORER_CONTENT: 'file-explorer-content',

  // Breadcrumb
  BREADCRUMB_HOME: 'breadcrumb-home',
  BREADCRUMB_SEGMENT: (name: string) => `breadcrumb-segment-${name}`,

  // Columns view
  COLUMNS_PANEL_LEFT: 'columns-panel-left',
  COLUMNS_PANEL_RIGHT: 'columns-panel-right',

  // Context menu
  CONTEXT_MENU_NEW_FOLDER: 'context-menu-new-folder',
  CONTEXT_MENU_UPLOAD_FILES: 'context-menu-upload-files',
  CONTEXT_MENU_COPY_PATH: 'context-menu-copy-path',
  CONTEXT_MENU_DOWNLOAD: 'context-menu-download',

  // File upload
  FILE_UPLOAD_INPUT: 'file-upload-input',

  // Create directory dialog
  CREATE_DIR_DIALOG: 'create-dir-dialog',
  CREATE_DIR_INPUT: 'create-dir-input',
  CREATE_DIR_SUBMIT: 'create-dir-submit',

  // File items
  FILE_ITEM: (name: string) => `file-item-${name}`,

  // Context menu - additional items
  CONTEXT_MENU_RENAME: 'context-menu-rename',
  CONTEXT_MENU_DELETE: 'context-menu-delete',

  // Rename dialog
  RENAME_DIALOG: 'rename-dialog',
  RENAME_INPUT: 'rename-input',
  RENAME_SUBMIT: 'rename-submit',

  // Delete dialog
  DELETE_DIALOG: 'delete-dialog',
  DELETE_CONFIRM: 'delete-confirm',

  // Toolbar - view mode
  TOOLBAR_VIEW_LIST: 'toolbar-view-list',
  TOOLBAR_VIEW_GRID: 'toolbar-view-grid',
  TOOLBAR_VIEW_COLUMNS: 'toolbar-view-columns',
  TOOLBAR_VIEW_TABULAR: 'toolbar-view-tabular',

  // Tabular view
  TABULAR_ADD_COLUMN: 'tabular-add-column',
  TABULAR_START_EXTRACTION: 'tabular-start-extraction',
  TABULAR_TABLE: 'tabular-table',
  TABULAR_COLUMN_CHIP: (id: string) => `tabular-column-chip-${id}`,
  TABULAR_COLUMN_EDIT: (id: string) => `tabular-column-edit-${id}`,
  TABULAR_COLUMN_REMOVE: (id: string) => `tabular-column-remove-${id}`,
  TABULAR_ADD_COLUMN_NAME: 'tabular-add-column-name',
  TABULAR_ADD_COLUMN_DESCRIPTION: 'tabular-add-column-description',
  TABULAR_ADD_COLUMN_TYPE: 'tabular-add-column-type',
  TABULAR_ADD_COLUMN_SUBMIT: 'tabular-add-column-submit',
  TABULAR_COLUMN_HEADER: (name: string) => `tabular-column-header-${name}`,
  TABULAR_COLUMN_MENU: (name: string) => `tabular-column-menu-${name}`,
  TABULAR_COLUMN_REMOVE_CONFIRM: 'tabular-column-remove-confirm',
  TABULAR_EDIT_COLUMN_NAME: 'tabular-edit-column-name',
  TABULAR_EDIT_COLUMN_SAVE: 'tabular-edit-column-save',
  TABULAR_ADD_TAG_OPTION_VALUE: 'tabular-add-tag-option-value',
  TABULAR_ADD_TAG_OPTION_SUBMIT: 'tabular-add-tag-option-submit',

  // Toolbar - sort
  TOOLBAR_SORT_TRIGGER: 'toolbar-sort-trigger',
  SORT_OPTION: (id: string) => `sort-option-${id}`,

  // Toolbar - settings
  TOOLBAR_SETTINGS_TRIGGER: 'toolbar-settings-trigger',
  SETTINGS_SHOW_HIDDEN: 'settings-show-hidden',

  // Upload conflict dialog
  UPLOAD_CONFLICT_DIALOG: 'upload-conflict-dialog',
  UPLOAD_CONFLICT_REPLACE: 'upload-conflict-replace',
  UPLOAD_CONFLICT_SKIP: 'upload-conflict-skip',
  UPLOAD_CONFLICT_KEEP_BOTH: 'upload-conflict-keep-both',
  UPLOAD_CONFLICT_CONTINUE: 'upload-conflict-continue',

  // File preview
  FILE_PREVIEW_PANEL: 'file-preview-panel',
  FILE_PREVIEW_CLOSE: 'file-preview-close',
  FILE_PREVIEW_EDIT: 'file-preview-edit',
  FILE_PREVIEW_DOWNLOAD: 'file-preview-download',

  // File edit
  FILE_EDIT_PAGE: 'file-edit-page',
  FILE_EDIT_SAVE: 'file-edit-save',
  FILE_EDIT_SAVE_AS: 'file-edit-save-as',
  FILE_EDIT_BACK: 'file-edit-back',
  FILE_EDIT_VIEW_PREVIEW: 'file-edit-view-preview',
  FILE_EDIT_UNSAVED_BADGE: 'file-edit-unsaved-badge',
  FILE_EDIT_EDITOR: 'file-edit-editor',

  // Save As Dialog
  SAVE_AS_DIALOG: 'save-as-dialog',
  SAVE_AS_FILENAME: 'save-as-filename',
  SAVE_AS_SAVE: 'save-as-save',
  SAVE_AS_CANCEL: 'save-as-cancel',

  // JSON Editor
  JSON_EDITOR_ERROR: 'json-editor-error',
  JSON_EDITOR_VALID: 'json-editor-valid',
  JSON_EDITOR_FORMAT: 'json-editor-format',

  // Markdown Editor
  MARKDOWN_EDITOR_TEXTAREA: 'markdown-editor-textarea',
  MARKDOWN_EDITOR_PREVIEW: 'markdown-editor-preview',

  // Env Editor
  ENV_EDITOR_FORM_TAB: 'env-editor-form-tab',
  ENV_EDITOR_RAW_TAB: 'env-editor-raw-tab',
  ENV_EDITOR_ADD_VARIABLE: 'env-editor-add-variable',
  ENV_EDITOR_KEY_INPUT: 'env-editor-key-input',
  ENV_EDITOR_VALUE_INPUT: 'env-editor-value-input',
  ENV_EDITOR_DELETE_ENTRY: 'env-editor-delete-entry',

  // Unsaved Changes Dialog
  UNSAVED_DIALOG: 'unsaved-dialog',
  UNSAVED_KEEP_EDITING: 'unsaved-keep-editing',
  UNSAVED_DISCARD: 'unsaved-discard',

  // --- Chat ---

  // Chat page
  CHAT_PAGE: 'chat-page',
  CHAT_MESSAGES: 'chat-messages',
  CHAT_MESSAGE: (role: string, index: number) => `chat-message-${role}-${index}`,

  // Chat input
  CHAT_INPUT_TEXTAREA: 'chat-input-textarea',
  CHAT_INPUT_SUBMIT: 'chat-input-submit',
  CHAT_ATTACHMENT_TRIGGER: 'chat-attachment-trigger',
  CHAT_ATTACHMENT_FILES: 'chat-attachment-files',

  // Chat metadata
  CHAT_METADATA: 'chat-metadata',
  CHAT_METADATA_COUNT: 'chat-metadata-count',

  // Skill selector
  SKILL_SELECTOR_TRIGGER: 'skill-selector-trigger',
  SKILL_SELECTOR_ITEM: (name: string) => `skill-item-${name}`,
  SKILL_SELECTED_CHIP: 'skill-selected-chip',

  // Thinking / tool chain
  CHAT_THINKING_TRIGGER: 'chat-thinking-trigger',

  // File reference in AI message
  CHAT_FILE_REFERENCE: (name: string) => `chat-file-ref-${name}`,

  // Sidebar
  SIDEBAR_TOGGLE: 'sidebar-toggle',
  SIDEBAR_SCOPE_GLOBAL: 'sidebar-scope-global',
  SIDEBAR_SCOPE_LOCAL: 'sidebar-scope-local',
  SIDEBAR_CHAT_ITEM: 'sidebar-chat-item',
  SIDEBAR_CHAT_MENU: 'sidebar-chat-menu',
  SIDEBAR_CHAT_DELETE: 'sidebar-chat-delete',

  // Automation pickers
  AUTOMATION_AGENT_PICKER: 'automation-agent-picker',
  AUTOMATION_AGENT_PICKER_DROPDOWN: 'automation-agent-picker-dropdown',
  AUTOMATION_SKILLS_PICKER: 'automation-skills-picker',
  AUTOMATION_SKILLS_PICKER_DROPDOWN: 'automation-skills-picker-dropdown',

  // Automation runs
  AUTOMATION_RUN_TRIGGER: 'automation-run-trigger',
  AUTOMATION_RUN_STATUS_BADGE: 'automation-run-status-badge',
  AUTOMATION_RUN_DETAILS_DIALOG: 'automation-run-details-dialog',
  AUTOMATION_RUN_HISTORY: 'automation-run-history',
  AUTOMATION_RUN_HISTORY_TOGGLE: 'automation-run-history-toggle',

  // Automations
  AUTOMATION_PAGE: 'automation-page',
  AUTOMATION_ADD_BUTTON: 'automation-add-button',
  AUTOMATION_LIST: 'automation-list',
  AUTOMATION_EMPTY_STATE: 'automation-empty-state',
  AUTOMATION_DIALOG: 'automation-dialog',
  AUTOMATION_DIALOG_SUBMIT: 'automation-dialog-submit',
  AUTOMATION_CARD: 'automation-card',
  AUTOMATION_CARD_TITLE: 'automation-card-title',
  AUTOMATION_TOGGLE: 'automation-toggle',
  AUTOMATION_EDIT: 'automation-edit',
  AUTOMATION_DELETE: 'automation-delete',
  AUTOMATION_DELETE_CONFIRM: 'automation-delete-confirm',
  AUTOMATION_SETTINGS_MENU: 'automation-settings-menu',

  // Automation editor
  AUTOMATION_EDITOR_PAGE: 'automation-editor-page',
  AUTOMATION_EDITOR_MODE_SWITCH: 'automation-editor-mode-switch',
  AUTOMATION_EDITOR_SAVE: 'automation-editor-save',
  AUTOMATION_EDITOR_CANCEL: 'automation-editor-cancel',

  // Automation wizard
  AUTOMATION_WIZARD_STEPPER: 'automation-wizard-stepper',
  AUTOMATION_WIZARD_STEP_1: 'automation-wizard-step-1',
  AUTOMATION_WIZARD_STEP_2: 'automation-wizard-step-2',
  AUTOMATION_WIZARD_STEP_3: 'automation-wizard-step-3',
  AUTOMATION_WIZARD_RENDERER: 'automation-wizard-renderer',
  AUTOMATION_WIZARD_NEXT: 'automation-wizard-next',
  AUTOMATION_WIZARD_BACK: 'automation-wizard-back',
  AUTOMATION_ADVANCED_RENDERER: 'automation-advanced-renderer',

  // Automation editor sections
  AUTOMATION_SECTION_AGENT_SKILLS: 'automation-section-agent-skills',
  AUTOMATION_SECTION_DETAILS: 'automation-section-details',
  AUTOMATION_SECTION_SCHEDULE: 'automation-section-schedule',

  // Automation conflict
  AUTOMATION_CONFLICT_BANNER: 'automation-conflict-banner',

  // Automation unsaved changes
  AUTOMATION_UNSAVED_DIALOG: 'automation-unsaved-dialog',

  // Cron Builder
  CRON_BUILDER: 'cron-builder',
  CRON_PRESET_SELECT: 'cron-preset-select',
  CRON_MINUTE_SELECT: 'cron-minute-select',
  CRON_RAW_TOGGLE: 'cron-raw-toggle',
  CRON_RAW_INPUT: 'cron-raw-input',

  // Folder Picker
  FOLDER_PICKER_INPUT: 'folder-picker-input',
  FOLDER_PICKER_BROWSE: 'folder-picker-browse',
  FOLDER_PICKER_CLEAR: 'folder-picker-clear',
  FOLDER_PICKER_DIALOG: 'folder-picker-dialog',
  FOLDER_PICKER_SELECT_CURRENT: 'folder-picker-select-current',
  FOLDER_PICKER_HOME: 'folder-picker-home',
  FOLDER_PICKER_ENTRY: 'folder-picker-entry',
  FOLDER_PICKER_ENTRY_NAME: 'folder-picker-entry-name',
  FOLDER_PICKER_ENTRY_SELECT: 'folder-picker-entry-select',
  FOLDER_PICKER_PARENT: 'folder-picker-parent',
  FOLDER_PICKER_EMPTY: 'folder-picker-empty',
} as const
