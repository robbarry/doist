use color_eyre::{Result, eyre::WrapErr};
use owo_colors::{OwoColorize, Stream};

use crate::{
    api::rest::{CreateSection, Gateway},
    config::Config,
};

use super::filter::TaskOrInteractive;

#[derive(clap::Parser, Debug)]
pub struct Params {
    #[clap(flatten)]
    pub task: TaskOrInteractive,
    /// Name of the section to move the task into, within the task's current project.
    /// Created automatically if no section with this name exists yet.
    #[arg(short = 'S', long = "section")]
    pub section: String,
}

/// Moves a task into a section of its current project, creating the section if it doesn't
/// already exist.
pub async fn mv(params: Params, gw: &Gateway, cfg: &Config) -> Result<()> {
    let id = params
        .task
        .task_id(gw, cfg)
        .await
        .wrap_err("no task selected for moving")?;
    let task = gw.task(&id).await?;

    let sections = gw.sections().await?;
    let section = match sections
        .into_iter()
        .find(|s| s.project_id == task.project_id && s.name.eq_ignore_ascii_case(&params.section))
    {
        Some(section) => section,
        None => gw
            .create_section(&CreateSection {
                name: params.section.clone(),
                project_id: task.project_id.clone(),
                ..Default::default()
            })
            .await
            .wrap_err("unable to create section")?,
    };

    gw.move_task(&id, &section.id).await?;
    println!(
        "moved task {} to section {}",
        id.if_supports_color(Stream::Stdout, |text| text.bright_red()),
        section.name
    );
    Ok(())
}

#[cfg(test)]
mod test {
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{body_json, method, path},
    };

    use crate::api::rest::{Section, Task};

    use super::*;

    async fn mock_empty_state(mock_server: &MockServer) {
        for path_str in ["/api/v1/tasks/filter", "/api/v1/projects", "/api/v1/labels"] {
            Mock::given(method("GET"))
                .and(path(path_str))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "results": [],
                    "next_cursor": null
                })))
                .mount(mock_server)
                .await;
        }
    }

    #[tokio::test]
    async fn moves_to_existing_section() {
        let mock_server = MockServer::start().await;
        mock_empty_state(&mock_server).await;
        Mock::given(method("GET"))
            .and(path("/api/v1/tasks/123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(Task::new("123", "task")))
            .mount(&mock_server)
            .await;
        Mock::given(method("GET"))
            .and(path("/api/v1/sections"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [Section::new("sec1", "", "Done")],
                "next_cursor": null
            })))
            .mount(&mock_server)
            .await;
        Mock::given(method("POST"))
            .and(path("/api/v1/tasks/123/move"))
            .and(body_json(serde_json::json!({"section_id": "sec1"})))
            .respond_with(ResponseTemplate::new(200).set_body_json(Task::new("123", "task")))
            .mount(&mock_server)
            .await;

        let gw = Gateway::new("", &mock_server.uri().parse().unwrap());
        let result = mv(
            Params {
                task: "123".to_string().into(),
                section: "done".to_string(),
            },
            &gw,
            &Config::default(),
        )
        .await;
        mock_server.verify().await;
        assert!(result.is_ok(), "{:?}", result.unwrap_err());
    }

    #[tokio::test]
    async fn creates_missing_section() {
        let mock_server = MockServer::start().await;
        mock_empty_state(&mock_server).await;
        Mock::given(method("GET"))
            .and(path("/api/v1/tasks/123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(Task::new("123", "task")))
            .mount(&mock_server)
            .await;
        Mock::given(method("GET"))
            .and(path("/api/v1/sections"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [],
                "next_cursor": null
            })))
            .mount(&mock_server)
            .await;
        Mock::given(method("POST"))
            .and(path("/api/v1/sections"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(Section::new("sec1", "", "Done")),
            )
            .mount(&mock_server)
            .await;
        Mock::given(method("POST"))
            .and(path("/api/v1/tasks/123/move"))
            .and(body_json(serde_json::json!({"section_id": "sec1"})))
            .respond_with(ResponseTemplate::new(200).set_body_json(Task::new("123", "task")))
            .mount(&mock_server)
            .await;

        let gw = Gateway::new("", &mock_server.uri().parse().unwrap());
        let result = mv(
            Params {
                task: "123".to_string().into(),
                section: "Done".to_string(),
            },
            &gw,
            &Config::default(),
        )
        .await;
        mock_server.verify().await;
        assert!(result.is_ok(), "{:?}", result.unwrap_err());
    }
}
