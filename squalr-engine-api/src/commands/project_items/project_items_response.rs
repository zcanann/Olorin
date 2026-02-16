use crate::commands::project_items::{
    activate::project_items_activate_response::ProjectItemsActivateResponse, add::project_items_add_response::ProjectItemsAddResponse,
    list::project_items_list_response::ProjectItemsListResponse,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ProjectItemsResponse {
    Add {
        project_items_add_response: ProjectItemsAddResponse,
    },
    Activate {
        project_items_activate_response: ProjectItemsActivateResponse,
    },
    List {
        project_items_list_response: ProjectItemsListResponse,
    },
}
