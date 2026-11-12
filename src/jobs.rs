//! Job latar belakang dan toast notifikasi Dynamic Island.

use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::time::Instant;

use eframe::egui;

use crate::app::CetakinApp;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JobTag {
    Sheet,
    Print,
    Convert,
    Scan,
}

#[derive(Clone)]
pub struct JobCtx {
    tx: Sender<JobEvent>,
}

impl JobCtx {
    pub fn progress(&self, value: f32, message: &str) {
        let _ = self.tx.send(JobEvent::Progress(
            value.clamp(0.0, 1.0),
            message.to_string(),
        ));
    }
}

pub enum JobEvent {
    Progress(f32, String),
    Done(Result<JobResult, String>),
}

pub struct JobResult {
    pub title: String,
    pub note: String,
    pub files: Vec<PathBuf>,
}

pub struct Job {
    pub(crate) label: String,
    pub(crate) tag: JobTag,
    pub(crate) rx: Receiver<JobEvent>,
    pub(crate) progress: f32,
    pub(crate) message: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToastKind {
    Success,
    Error,
    Info,
}

pub struct Toast {
    pub(crate) title: String,
    pub(crate) detail: String,
    pub(crate) kind: ToastKind,
    pub(crate) created: Instant,
}

impl CetakinApp {
    // -------------------------------------------------------------- jobs

    pub fn start_job<F>(&mut self, label: impl Into<String>, tag: JobTag, work: F)
    where
        F: FnOnce(JobCtx) -> Result<JobResult, String> + Send + 'static,
    {
        if self.job.is_some() {
            self.toast_info(
                "Masih ada proses berjalan",
                "Tunggu proses selesai sebelum memulai yang baru.",
            );
            return;
        }
        let (tx, rx) = channel();
        let ctx = JobCtx { tx: tx.clone() };
        let label = label.into();
        std::thread::spawn(move || {
            let result = work(ctx);
            let _ = tx.send(JobEvent::Done(result));
        });
        self.job = Some(Job {
            label,
            tag,
            rx,
            progress: 0.0,
            message: "Memulai…".to_string(),
        });
        self.island_pinned = false;
        self.island_hovered = false;
    }

    pub(crate) fn poll_job(&mut self, ctx: &egui::Context) {
        let mut finished: Option<Result<JobResult, String>> = None;
        if let Some(job) = &mut self.job {
            loop {
                match job.rx.try_recv() {
                    Ok(JobEvent::Progress(p, message)) => {
                        job.progress = p;
                        job.message = message;
                    }
                    Ok(JobEvent::Done(result)) => {
                        finished = Some(result);
                        break;
                    }
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        finished = Some(Err("Proses terhenti tanpa hasil.".to_string()));
                        break;
                    }
                }
            }
        }
        if let Some(result) = finished {
            let tag = self.job.as_ref().map(|j| j.tag);
            self.job = None;
            match result {
                Ok(res) => {
                    if tag == Some(JobTag::Convert) {
                        self.convert.results = res.files.clone();
                        self.convert.note = res.note.clone();
                    }
                    if tag == Some(JobTag::Scan) {
                        self.scan.last_saved = res.files.clone();
                        if self.scan.add_after_save {
                            self.scan.add_after_save = false;
                            let images: Vec<_> = res.files.clone();
                            if !images.is_empty() {
                                self.add_photos(images);
                                self.tab = crate::app::Tab::Sheet;
                            }
                        }
                    }
                    if !res.files.is_empty() {
                        self.last_output = res.files.clone();
                    }
                    if tag == Some(JobTag::Print) {
                        self.toast_success("Siap dicetak", &res.note);
                    } else {
                        self.toast_success(&res.title, &res.note);
                    }
                }
                Err(e) => {
                    self.scan.add_after_save = false;
                    self.toast_error("Proses gagal", &e);
                }
            }
            ctx.request_repaint();
        }
    }

    pub fn toast_success(&mut self, title: &str, detail: &str) {
        self.toast = Some(Toast {
            title: title.to_string(),
            detail: detail.to_string(),
            kind: ToastKind::Success,
            created: Instant::now(),
        });
        self.island_pinned = false;
    }

    pub fn toast_error(&mut self, title: &str, detail: &str) {
        self.toast = Some(Toast {
            title: title.to_string(),
            detail: detail.to_string(),
            kind: ToastKind::Error,
            created: Instant::now(),
        });
        self.island_pinned = false;
    }

    pub fn toast_info(&mut self, title: &str, detail: &str) {
        self.toast = Some(Toast {
            title: title.to_string(),
            detail: detail.to_string(),
            kind: ToastKind::Info,
            created: Instant::now(),
        });
        self.island_pinned = false;
    }
}
