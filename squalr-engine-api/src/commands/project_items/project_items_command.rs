use crate::commands::project_items::{
    activate::project_items_activate_request::ProjectItemsActivateRequest, add::project_items_add_request::ProjectItemsAddRequest,
    list::project_items_list_request::ProjectItemsListRequest,
};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub enum ProjectItemsCommand {
    /// Adds project items from the provided scan results.
    Add {
        #[structopt(flatten)]
        project_items_add_request: ProjectItemsAddRequest,
    },
    /// Activates project items.
    Activate {
        #[structopt(flatten)]
        project_items_activate_request: ProjectItemsActivateRequest,
    },
    /// Lists opened project items.
    List {
        #[structopt(flatten)]
        project_items_list_request: ProjectItemsListRequest,
    },
}
