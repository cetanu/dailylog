DAILYLOG={
  concept: "rust_cli_journaling_tool_git_commit_style_parsing_markdown_output",
  arch: "modular_6files_anyhow_chrono_clap_termcolor",
  
  modules: {
    main: "cli_dispatch_only",
    config: "Config_struct_toml_loading_defaults", 
    entry: "editor_parsing_formatting_file_io",
    git: "repo_ops_sync_auto_commit",
    display: "terminal_colors_markdown_rendering",
    summary: "analytics_stats_weekday_filtering"
  },
  
  core_flow: "editor->parse_title_body->format_timestamp->append_md->optional_git_sync",
  entry_format: "first_line_title|blank_line|body -> ## HH:MM - title\\n\\nbody",
  
  config_fields: "log_dir git_repo git_auto_sync git_branch_name summary_days",
  config_defaults: "~/.dailylog master false [mon-fri]",
  
  commands: "default=new_entry previous=view_yesterday yesterday=append_yesterday summary=analyze_N_days sync=pull_push pull push",
  
  key_functions: {
    entry: "open_editor parse_entry format_entry append_to_log get_*_path",
    git: "is_git_repo run_git_command git_pull git_push git_sync auto_sync_if_enabled",
    display: "render_markdown_to_terminal view_previous_day_log add_to_previous_day_log", 
    summary: "summarize_logs extract_entry_titles parse_weekday",
    config: "load_config default_*"
  },
  
  data_patterns: {
    files: "YYYY-MM-DD.md ~/.dailylog.toml temp/dailylog.md",
    colors: "h1=blue h2=cyan h3=green lists=yellow dates=magenta code=gray",
    errors: "anyhow::Result<()> graceful_fallbacks user_friendly_messages",
    dates: "chrono::Local chrono::Duration chrono::NaiveDate",
    git: "pull_before_push auto_init_repo commit_with_timestamp"
  },
  
  dependencies: "anyhow chrono dirs toml serde clap termcolor",
  
  task_module_map: {
    config_changes: "config.rs",
    entry_parsing: "entry.rs", 
    git_features: "git.rs",
    display_colors: "display.rs",
    analytics: "summary.rs",
    cli_commands: "main.rs"
  }
}