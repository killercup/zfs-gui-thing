use core::fmt::Display;
use vgtk::lib::gtk::*;

pub trait ResultNotificationExt<T> {
    fn or_popup(self) -> Self;
}

impl<T, E: Display> ResultNotificationExt<T> for Result<T, E> {
    fn or_popup(self) -> Self {
        if let Err(e) = &self {
            log::error!("{}", e);

            vgtk::message_dialog(
                vgtk::current_window().as_ref(),
                DialogFlags::MODAL,
                MessageType::Error,
                ButtonsType::Ok,
                true,
                format!("<b>ERROR:</b> {}", e),
            )
        }
        self
    }
}
