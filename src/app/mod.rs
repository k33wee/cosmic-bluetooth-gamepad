mod handlers;
mod icons;
mod messages;
mod model;
mod view;

pub const APP_ID: &str = "com.keewee.CosmicBluetoothGamepad";

pub use messages::Message;
pub use model::{AppModel, ConnectedDevice, DeviceData};

use cosmic::iced::window::Id;
use cosmic::prelude::*;

impl cosmic::Application for AppModel {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = APP_ID;

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    fn init(core: cosmic::Core, flags: Self::Flags) -> (Self, Task<cosmic::Action<Self::Message>>) {
        handlers::init(core, flags)
    }

    fn on_close_requested(&self, id: Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn view(&self) -> Element<'_, Self::Message> {
        view::view(self)
    }

    fn view_window(&self, id: Id) -> Element<'_, Self::Message> {
        view::view_window(self, id)
    }

    fn subscription(&self) -> cosmic::iced::Subscription<Self::Message> {
        handlers::subscription(self)
    }

    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        handlers::update(self, message)
    }

    fn style(&self) -> Option<cosmic::iced_runtime::Appearance> {
        Some(cosmic::applet::style())
    }
}
