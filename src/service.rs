use crate::manager::Manager;
use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolResult, Content, Implementation, InitializeRequestParam, InitializeResult,
    ProtocolVersion, ServerCapabilities, ServerInfo,
};
use rmcp::service::RequestContext;
use rmcp::{tool, tool_handler, tool_router, ErrorData, RoleServer, ServerHandler};
use schemars::JsonSchema;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize, JsonSchema, Clone)]
struct StartRequest {
    command: String,
}

#[derive(Deserialize, JsonSchema, Clone)]
struct StopRequest {
    session_key: String,
}

#[derive(Deserialize, JsonSchema, Clone)]
struct StatusRequest {
    session_key: Option<String>,
}

#[derive(Deserialize, JsonSchema, Clone)]
struct TailRequest {
    session_key: String,
}

#[derive(Clone)]
pub struct DevManagerService {
    manager: Arc<Manager>,
    tool_router: ToolRouter<Self>,
}

impl DevManagerService {
    pub fn new(manager: Arc<Manager>) -> Self {
        Self {
            manager,
            tool_router: Self::tool_router(),
        }
    }
}

#[tool_router]
impl DevManagerService {
    #[tool(description = "Start a development server. Returns auto-generated session key, port number and status.")]
    async fn start(&self, Parameters(req): Parameters<StartRequest>) -> Result<CallToolResult, ErrorData> {
        let result = self.manager.start(req.command).await;
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "Stop a running development server session.")]
    async fn stop(&self, Parameters(req): Parameters<StopRequest>) -> Result<CallToolResult, ErrorData> {
        let result = self.manager.stop(req.session_key).await;
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "Get status of one or all development server sessions.")]
    async fn status(&self, Parameters(req): Parameters<StatusRequest>) -> Result<CallToolResult, ErrorData> {
        let result = self.manager.status(req.session_key);
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "Get stdout/stderr logs for a development server session.")]
    async fn tail(&self, Parameters(req): Parameters<TailRequest>) -> Result<CallToolResult, ErrorData> {
        let result = self.manager.tail(req.session_key);
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }
}

#[tool_handler]
impl ServerHandler for DevManagerService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "MCP Dev Server Manager - manages multiple development server sessions with automatic port allocation and log capture.".to_string()
            ),
        }
    }

    async fn initialize(
        &self,
        _params: InitializeRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, ErrorData> {
        Ok(self.get_info())
    }
}
