//! Client Management Commands
//!
//! CRUD operations for client profiles stored in the local database.

use crate::db::{Client, ClientRepository, Database};
use crate::error::OptioError;
use serde::{Deserialize, Serialize};
use tauri::State;

/// Request to create a new client
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateClientRequest {
    pub name: String,
    pub target_subnet: Option<String>,
    pub contact_email: Option<String>,
    pub notes: Option<String>,
}

/// Client response for the frontend
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientResponse {
    pub id: String,
    pub name: String,
    pub target_subnet: Option<String>,
    pub contact_email: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Client> for ClientResponse {
    fn from(client: Client) -> Self {
        ClientResponse {
            id: client.id,
            name: client.name,
            target_subnet: client.target_subnet,
            contact_email: client.contact_email,
            notes: client.notes,
            created_at: client.created_at.to_rfc3339(),
            updated_at: client.updated_at.to_rfc3339(),
        }
    }
}

/// Create a new client
#[tauri::command]
pub async fn create_client(
    db: State<'_, Database>,
    request: CreateClientRequest,
) -> Result<ClientResponse, String> {
    tracing::info!("Creating client: {}", request.name);

    let client = Client::new(
        request.name,
        request.target_subnet,
        request.contact_email,
        request.notes,
    );

    let repo = ClientRepository::new(&db);
    repo.create(&client).map_err(|e| e.to_string())?;

    Ok(ClientResponse::from(client))
}

/// List all clients
#[tauri::command]
pub async fn list_clients(db: State<'_, Database>) -> Result<Vec<ClientResponse>, String> {
    tracing::debug!("Listing all clients");

    let repo = ClientRepository::new(&db);
    let clients = repo.list().map_err(|e| e.to_string())?;

    Ok(clients.into_iter().map(ClientResponse::from).collect())
}

/// Get a single client by ID
#[tauri::command]
pub async fn get_client(db: State<'_, Database>, id: String) -> Result<ClientResponse, String> {
    tracing::debug!("Getting client: {}", id);

    let repo = ClientRepository::new(&db);
    let client = repo
        .get(&id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Client not found: {}", id))?;

    Ok(ClientResponse::from(client))
}

/// Request to update a client
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateClientRequest {
    pub id: String,
    pub name: String,
    pub target_subnet: Option<String>,
    pub contact_email: Option<String>,
    pub notes: Option<String>,
}

/// Update an existing client
#[tauri::command]
pub async fn update_client(
    db: State<'_, Database>,
    request: UpdateClientRequest,
) -> Result<ClientResponse, String> {
    tracing::info!("Updating client: {}", request.id);

    let repo = ClientRepository::new(&db);

    // Get existing client to preserve created_at
    let existing = repo
        .get(&request.id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Client not found: {}", request.id))?;

    let updated = Client {
        id: request.id,
        name: request.name,
        target_subnet: request.target_subnet,
        contact_email: request.contact_email,
        notes: request.notes,
        created_at: existing.created_at,
        updated_at: chrono::Utc::now(),
    };

    repo.update(&updated).map_err(|e| e.to_string())?;

    Ok(ClientResponse::from(updated))
}

/// Delete a client
#[tauri::command]
pub async fn delete_client(db: State<'_, Database>, id: String) -> Result<bool, String> {
    tracing::info!("Deleting client: {}", id);

    let repo = ClientRepository::new(&db);
    repo.delete(&id).map_err(|e| e.to_string())
}
