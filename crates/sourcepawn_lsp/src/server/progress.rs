use crate::{capabilities::ClientCapabilitiesExt, Server};

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Progress {
    Begin,
    Report,
    End,
}

impl Server {
    pub(crate) fn report_progress(
        &mut self,
        title: &str,
        state: Progress,
        message: Option<String>,
        fraction: Option<f64>,
        cancel_token: Option<String>,
    ) {
        if !self.client_capabilities.has_work_done_progress_support() {
            return;
        }
        let percentage = fraction.map(|f| {
            assert!((0.0..=1.0).contains(&f));
            (f * 100.0) as u32
        });
        let cancellable = Some(cancel_token.is_some());
        let token = lsp_types::ProgressToken::String(
            cancel_token.unwrap_or_else(|| format!("sourcepawnLsp/{title}")),
        );

        let work_done_progress = match state {
            Progress::Begin => {
                let _ = self
                    .client
                    .send_request_without_response::<lsp_types::request::WorkDoneProgressCreate>(
                        lsp_types::WorkDoneProgressCreateParams {
                            token: token.clone(),
                        },
                    );

                lsp_types::WorkDoneProgress::Begin(lsp_types::WorkDoneProgressBegin {
                    title: title.into(),
                    cancellable,
                    message,
                    percentage,
                })
            }
            Progress::Report => {
                lsp_types::WorkDoneProgress::Report(lsp_types::WorkDoneProgressReport {
                    cancellable,
                    message,
                    percentage,
                })
            }
            Progress::End => {
                lsp_types::WorkDoneProgress::End(lsp_types::WorkDoneProgressEnd { message })
            }
        };
        let _ = self
            .client
            .send_notification::<lsp_types::notification::Progress>(lsp_types::ProgressParams {
                token,
                value: lsp_types::ProgressParamsValue::WorkDone(work_done_progress),
            });
    }
}
