use color_eyre::{Result, eyre::eyre};

use crate::{api::rest::Gateway, comments, config::Config, tasks::state::State};

use super::filter::TaskOrInteractive;

#[derive(clap::Parser, Debug)]
pub struct Params {
    #[clap(flatten)]
    task: TaskOrInteractive,
}

/// Displays full information about a task.
pub async fn view(params: Params, gw: &Gateway, cfg: &Config) -> Result<()> {
    let (id, state) = if let Some(id) = params.task.id() {
        // A direct task ID may not be present in the default today/overdue
        // filter, so fetch the full active task tree before looking it up.
        (id.clone(), State::fetch_tree(Some("all"), gw, cfg).await?)
    } else {
        params.task.task(gw, cfg).await?
    };
    let task = state.full_task(state.task(&id).ok_or_else(|| eyre!("no valid task"))?);
    println!("{task}");
    if task.0.note_count > 0 {
        let comments = gw.task_comments(&id).await?;
        comments::list(&comments)
    }
    Ok(())
}
