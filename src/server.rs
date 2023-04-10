use dashmap::DashMap;
use ropey::Rope;
use serde_json::Value;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use crate::ini;
use crate::utils;
use crate::vale;
use crate::yml;

#[derive(Debug, Clone)]
struct TextDocumentItem {
    uri: Url,
    text: String,
}

#[derive(Debug)]
pub struct Backend {
    pub client: Client,
    pub document_map: DashMap<String, Rope>,
    pub param_map: DashMap<String, Value>,
    pub cli: vale::ValeManager,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        // TODO: Workspace folders / settings
        let mut cwd = "".to_string();
        if params.root_uri.is_some() {
            cwd = params
                .root_uri
                .unwrap()
                .to_file_path()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();
        }

        self.param_map
            .insert("root".to_string(), Value::String(cwd.clone()));

        self.init(params.initialization_options, cwd).await;
        Ok(InitializeResult {
            server_info: None,
            offset_encoding: None,
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::FULL),
                        save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                            include_text: Some(true),
                        })),
                        will_save: None,
                        will_save_wait_until: None,
                    },
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec!["cli.sync".to_string(), "cli.compile".to_string()],
                    work_done_progress_options: Default::default(),
                }),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: None,
                    work_done_progress_options: Default::default(),
                    all_commit_characters: None,
                    completion_item: None,
                }),
                code_action_provider: Some(CodeActionProviderCapability::Options(
                    CodeActionOptions {
                        code_action_kinds: Some(vec![CodeActionKind::QUICKFIX]),
                        work_done_progress_options: WorkDoneProgressOptions {
                            work_done_progress: None,
                        },
                        resolve_provider: None,
                    },
                )),
                code_lens_provider: Some(CodeLensOptions {
                    resolve_provider: Some(true),
                }),
                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                        supported: Some(true),
                        change_notifications: Some(OneOf::Left(true)),
                    }),
                    file_operations: None,
                }),
                ..ServerCapabilities::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.on_change(TextDocumentItem {
            uri: params.text_document.uri,
            text: params.text_document.text,
        })
        .await
    }

    async fn did_change(&self, mut params: DidChangeTextDocumentParams) {
        self.update(TextDocumentItem {
            uri: params.text_document.uri,
            text: std::mem::take(&mut params.content_changes[0].text),
        });
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        if params.text.is_some() {
            self.on_change(TextDocumentItem {
                uri: params.text_document.uri,
                text: params.text.unwrap(),
            })
            .await
        }
    }

    async fn execute_command(&self, params: ExecuteCommandParams) -> Result<Option<Value>> {
        match params.command.as_str() {
            "cli.sync" => self.do_sync(params.arguments).await,
            "cli.compile" => self.do_compile(params.arguments).await,
            _ => {}
        };
        Ok(None)
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;

        let ext = self.get_ext(uri.clone());
        if self.document_map.get(uri.as_str()).is_none() {
            return Ok(None);
        }
        let pos = params.text_document_position_params.position;

        let rope = self.document_map.get(uri.as_str()).unwrap();
        let span = utils::position_to_range(pos, &rope);

        if span.is_none() {
            return Ok(None);
        }
        let range = span.unwrap();

        let token = utils::range_to_token(range, &rope);
        if ext == "ini" && ini::key_to_info(&token).is_some() {
            return Ok(Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: ini::key_to_info(&token).unwrap().to_string(),
                }),
                range: Some(range),
            }));
        } else if ext == "yml" {
            let rule = yml::Rule::new(uri.path());
            if rule.is_ok() {
                let info = rule.unwrap();
                let desc = info.token_info(&token);
                if desc.is_some() {
                    return Ok(Some(Hover {
                        contents: HoverContents::Markup(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: desc.unwrap().to_string(),
                        }),
                        range: Some(range),
                    }));
                }
            }
        }

        Ok(None)
    }

    async fn did_change_configuration(&self, _: DidChangeConfigurationParams) {
        self.client
            .log_message(MessageType::INFO, "configuration changed!")
            .await;
    }

    async fn did_change_workspace_folders(&self, _: DidChangeWorkspaceFoldersParams) {
        self.client
            .log_message(MessageType::INFO, "workspace folders changed!")
            .await;
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;

        let ext = self.get_ext(uri.clone());
        if self.document_map.get(uri.as_str()).is_none() {
            return Ok(None);
        }

        let position = params.text_document_position.position;
        let rope = self.document_map.get(uri.as_str()).unwrap();

        let context = rope.line(position.line as usize);
        let line = context.as_str().to_owned().unwrap_or("");

        let config = self.cli.config(self.config_path(), self.root_path());
        if config.is_err() {
            return Ok(None);
        }

        let styles = config.unwrap().styles_path;
        if ext == "ini" {
            match ini::complete(line, styles).await {
                Ok(computed) => {
                    return Ok(Some(CompletionResponse::Array(computed)));
                }
                Err(err) => {
                    self.client
                        .log_message(MessageType::ERROR, format!("Error: {}", err))
                        .await;
                }
            }
        }

        Ok(None)
    }

    async fn code_lens(&self, params: CodeLensParams) -> Result<Option<Vec<CodeLens>>> {
        let uri = params.text_document.uri;
        let path = uri.to_file_path().unwrap();

        let rule = yml::Rule::new(path.to_str().unwrap());
        if rule.is_ok() && rule.unwrap().can_compile() {
            let lens = vec![CodeLens {
                range: Range {
                    start: Position {
                        line: 0,
                        character: 0,
                    },
                    end: Position {
                        line: 0,
                        character: 0,
                    },
                },
                command: Some(Command {
                    title: "Compile Rule".to_string(),
                    command: "cli.compile".to_string(),
                    arguments: Some(vec![Value::String(uri.to_string())]),
                    ..Command::default()
                }),
                data: Some(Value::String("".to_string())),
            }];

            return Ok(lens.into());
        }

        Ok(None)
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        if params.context.diagnostics.is_empty() {
            return Ok(None);
        }

        let d = params.context.diagnostics[0].data.as_ref().unwrap();
        let s = serde_json::to_string(d).unwrap();

        match self.cli.fix(&s) {
            Ok(fixed) => {
                let alert: vale::ValeAlert = serde_json::from_str(&s).unwrap();
                let mut range = utils::alert_to_range(alert.clone());

                if !alert.action.name.is_some() {
                    return Ok(None);
                }

                let action_name = alert.action.name.unwrap();
                if action_name == "remove" {
                    // NOTE: we need to add a character when deleting to avoid
                    // leaving a double space.
                    range.end.character += 1;
                }

                let mut fixes = vec![];
                for fix in fixed.suggestions {
                    fixes.push(CodeActionOrCommand::CodeAction(CodeAction {
                        title: utils::make_title(
                            action_name.clone(),
                            alert.matched.clone(),
                            fix.clone(),
                        ),
                        kind: Some(CodeActionKind::QUICKFIX),
                        diagnostics: Some(params.context.diagnostics.clone()),
                        edit: Some(WorkspaceEdit {
                            changes: Some(
                                [(
                                    params.text_document.uri.clone(),
                                    vec![TextEdit {
                                        range: range,
                                        new_text: fix,
                                    }],
                                )]
                                .iter()
                                .cloned()
                                .collect(),
                            ),
                            ..WorkspaceEdit::default()
                        }),
                        ..CodeAction::default()
                    }));
                }
                Ok(Some(fixes))
            }
            Err(e) => {
                self.client
                    .log_message(MessageType::ERROR, format!("Error: {}", e))
                    .await;
                Ok(None)
            }
        }
    }
}

impl Backend {
    async fn on_change(&self, params: TextDocumentItem) {
        let uri = params.uri.clone();

        self.update(params.clone());
        if self.cli.is_installed() {
            match self.cli.run(uri.path(), self.config_filter()) {
                Ok(result) => {
                    let mut diagnostics = Vec::new();
                    for (_, v) in result.iter() {
                        for alert in v {
                            diagnostics.push(utils::alert_to_diagnostic(alert));
                        }
                    }
                    self.client
                        .publish_diagnostics(params.uri.clone(), diagnostics, None)
                        .await;
                }
                Err(err) => {
                    self.client
                        .log_message(MessageType::ERROR, format!("Parsing error: {:?}", err))
                        .await;
                    match serde_json::from_str::<vale::ValeError>(&err.to_string()) {
                        Ok(parsed) => {
                            self.client.show_message(MessageType::ERROR, parsed).await;
                        }
                        Err(e) => {
                            self.client.show_message(MessageType::ERROR, e).await;
                        }
                    };
                }
            }
        }
    }

    async fn init(&self, params: Option<Value>, cwd: String) {
        self.parse_params(params);

        if self.should_install() {
            match self.cli.install_or_update() {
                Ok(status) => {
                    self.client.log_message(MessageType::INFO, status).await;
                }
                Err(err) => {
                    self.client
                        .show_message(MessageType::INFO, err.to_string())
                        .await;
                    self.client
                        .log_message(MessageType::ERROR, err.to_string())
                        .await;
                }
            }
        }

        if self.should_sync() && cwd != "" {
            match self.cli.sync(self.config_path(), cwd) {
                Ok(_) => {
                    self.client
                        .log_message(MessageType::INFO, "Successfully synced Vale config.")
                        .await;
                }
                Err(err) => {
                    self.client
                        .log_message(MessageType::ERROR, err.to_string())
                        .await;
                }
            }
        }
    }

    fn should_install(&self) -> bool {
        self.get_setting("installVale") == Some(Value::Bool(true))
    }

    fn config_path(&self) -> String {
        self.get_string("configPath")
    }

    fn config_filter(&self) -> String {
        self.get_string("filter")
    }

    fn should_sync(&self) -> bool {
        self.get_setting("syncOnStartup") == Some(Value::Bool(true))
    }

    fn root_path(&self) -> String {
        self.get_string("root")
    }

    fn parse_params(&self, params: Option<Value>) {
        if let Some(Value::Object(map)) = params {
            for (k, v) in map {
                self.param_map.insert(k.to_string(), v.clone());
            }
        }
    }

    fn get_string(&self, key: &str) -> String {
        if self.get_setting(key).is_some() {
            let value = self.get_setting(key).unwrap();
            if value.is_string() {
                return value.as_str().unwrap().to_string();
            }
        }
        "".to_string()
    }

    fn get_setting(&self, key: &str) -> Option<Value> {
        if self.param_map.contains_key(key) {
            let value = self.param_map.get(key).unwrap();
            return Some(value.clone());
        }
        None
    }

    fn update(&self, params: TextDocumentItem) {
        let uri = params.uri.clone();
        if self.get_ext(uri) != "" {
            let rope = ropey::Rope::from_str(&params.text);
            self.document_map
                .insert(params.uri.to_string(), rope.clone());
        }
    }

    fn get_ext(&self, uri: Url) -> String {
        let ext = uri.path().split('.').last().unwrap_or("");
        if uri.path().contains(".vale.ini") {
            return "ini".to_string();
        } else if ext == "yml" {
            // TODO: ensure path is on `StylesPath`.
            return "yml".to_string();
        }
        "".to_string()
    }

    async fn do_sync(&self, _: Vec<Value>) {
        match self.cli.sync(self.config_path(), self.root_path()) {
            Ok(_) => {
                self.client
                    .show_message(MessageType::INFO, "Successfully synced Vale config.")
                    .await;
            }
            Err(e) => {
                self.client
                    .show_message(MessageType::ERROR, format!("Failed to sync CLI: {}", e))
                    .await;
            }
        }
    }

    async fn do_compile(&self, arguments: Vec<Value>) {
        if arguments.len() == 0 {
            self.client
                .show_message(MessageType::ERROR, "No URI provided. Please try again.")
                .await;
            return;
        }

        let arg = arguments[0].as_str().unwrap().to_string();
        let uri = Url::parse(&arg).unwrap().to_file_path().unwrap();

        let ext = uri.extension().unwrap().to_str().unwrap();
        if ext != "yml" {
            self.client
                .show_message(
                    MessageType::ERROR,
                    "Only YAML files are supported; skipping compilation.",
                )
                .await;
            return;
        }

        let resp = self.cli.upload_rule(
            self.config_path(),
            self.root_path(),
            uri.to_str().unwrap().to_string(),
        );

        match resp {
            Ok(r) => {
                let session = format!("https://regex101.com/r/{}", r.permalink_fragment);
                match open::that(session) {
                    Ok(_) => {
                        self.client
                            .show_message(
                                MessageType::INFO,
                                "Successfully compiled rule. Opening Regex101.",
                            )
                            .await;
                    }
                    Err(e) => {
                        self.client
                            .show_message(
                                MessageType::ERROR,
                                format!("Failed to open Regex101: {}", e),
                            )
                            .await;
                    }
                }
            }
            Err(e) => {
                self.client
                    .show_message(MessageType::ERROR, format!("Failed to compile rule: {}", e))
                    .await;
            }
        }
    }
}
