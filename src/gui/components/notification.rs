use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use egui::{mutex::Mutex, Align2, Context, Frame, Id, Order, Ui, Vec2};
use once_cell::sync::Lazy;

type NotificationsState = Arc<Mutex<Vec<Notification>>>;

pub(crate) struct Notification {
    pub started_at: Instant,
    pub duration: Duration,
    pub add_contents: Box<dyn Fn(&mut Ui) -> Option<bool> + Sync + Send + 'static>,
}

impl Notification {
    #[must_use = "You need to send the notification for it to be useful"]
    pub fn new(add_contents: impl Fn(&mut Ui) -> Option<bool> + Sync + Send + 'static) -> Self {
        Notification {
            started_at: Instant::now(),
            duration: Duration::from_secs(2),
            add_contents: Box::new(add_contents),
        }
    }

    #[allow(unused)]
    #[must_use = "You need to send the notification for it to be useful"]
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    pub fn send(self, ctx: &Context) {
        let mut memory = ctx.memory();
        let state = memory
            .data
            .get_temp_mut_or_default::<NotificationsState>(*ID);

        state.lock().push(self);
    }
}

static ID: Lazy<Id> = Lazy::new(|| Id::new("__NOTIFICATIONS__"));

/// Render notifications, if any
///
/// This will automatically render notifications created with [`send_notification()`].
pub(crate) fn show_notifications(ctx: &Context) {
    let notifcations = {
        let mut memory = ctx.memory();
        let state = memory
            .data
            .get_temp_mut_or_default::<NotificationsState>(*ID);

        let n = state.lock().drain(..).collect::<Vec<_>>();
        n
    };

    let mut new_notifications = Vec::new();

    let now = Instant::now();
    egui::Area::new(ID.with("popup"))
        .order(Order::Foreground)
        .anchor(Align2::CENTER_TOP, Vec2::ZERO)
        .show(ctx, |ui| {
            for n in notifcations {
                let mut should_close_notification = None;
                Frame::popup(ui.style())
                    .outer_margin(ui.style().spacing.window_margin)
                    .show(ui, |ui| {
                        should_close_notification = (n.add_contents)(ui);
                    });

                // If the function returned whether or not the notification should close
                if let Some(should_close) = should_close_notification {
                    if !should_close {
                        new_notifications.push(n);
                    }

                // If the function did not specify whether or not the notification should close,
                // just check the notification duration and keep it alive it it isn't expired yet.
                } else if now.duration_since(n.started_at) < n.duration {
                    ui.ctx().request_repaint();
                    new_notifications.push(n);
                }
            }
        });

    let mut memory = ctx.memory();
    let state = memory
        .data
        .get_temp_mut_or_default::<NotificationsState>(*ID);

    *state.lock() = new_notifications;
}

pub fn send_error_notification(ctx: &egui::Context, message: String) {
    Notification::new(move |ui| {
        Some(
            ui.horizontal(|ui| {
                ui.colored_label(egui::Color32::RED, format!("Error: {}", message));
                ui.button("x").clicked()
            })
            .inner,
        )
    })
    .send(ctx);
}
