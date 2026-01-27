use super::{AppModel, Message, icons};
use crate::fl;
use cosmic::iced::{Alignment, Length, window::Id};
use cosmic::prelude::*;
use cosmic::widget;

pub fn view(app: &AppModel) -> Element<'_, Message> {
    app.core
        .applet
        .icon_button("applications-games-symbolic")
        .on_press(Message::TogglePopup)
        .into()
}

pub fn view_window(app: &AppModel, _id: Id) -> Element<'_, Message> {
    let mut connected_list = widget::list_column().padding(5).spacing(0);
    if app.connected.is_empty() {
        connected_list = connected_list.add(widget::text(fl!("no-connected")));
    } else {
        for dev in &app.connected {
            let battery_text = dev
                .battery
                .map(|value| format!("{value}%"))
                .unwrap_or_else(|| fl!("battery-unknown").to_string());
            let label = format!("{} ({})", dev.name, dev.addr);
            let disconnect_button = widget::button::icon(icons::disconnect_icon())
                .tooltip(fl!("disconnect"))
                .on_press(Message::DisconnectDevice(dev.addr.clone()))
                .extra_small();

            let row = widget::row()
                .align_y(Alignment::Center)
                .spacing(8)
                .push(widget::text(label).width(Length::FillPortion(8)))
                .push(widget::text(battery_text).width(Length::FillPortion(2)))
                .push(
                    widget::container(disconnect_button)
                        .width(Length::FillPortion(2))
                        .align_x(Alignment::End),
                );

            connected_list = connected_list.add(row);
        }
    }

    let mut paired_list = widget::list_column().padding(5).spacing(0);
    if app.paired.is_empty() {
        paired_list = paired_list.add(widget::text(fl!("no-paired")));
    } else {
        for (addr, name) in &app.paired {
            if app.renaming_addr.as_ref() == Some(addr) {
                let addr_clone = addr.clone();
                let input = widget::text_input(fl!("rename-placeholder"), &app.renaming_value)
                    .on_input(Message::RenameInput)
                    .on_submit(move |_| Message::RenameSubmit(addr_clone.clone()));

                let actions = widget::row()
                    .spacing(8)
                    .push(
                        widget::button::text(fl!("save"))
                            .on_press(Message::RenameSubmit(addr.clone())),
                    )
                    .push(widget::button::text(fl!("cancel")).on_press(Message::RenameCancel));

                let row = widget::row()
                    .align_y(Alignment::Center)
                    .spacing(8)
                    .push(input.width(Length::FillPortion(3)))
                    .push(actions.width(Length::FillPortion(2)));

                paired_list = paired_list.add(row);
                continue;
            }

            let label = format!("{} ({})", name, addr);
            let buttons: Element<'_, Message> = if let Some(remaining) = app.reconnecting.get(addr)
            {
                widget::container(widget::text(fl!("reconnecting", seconds = remaining)))
                    .width(Length::FillPortion(4))
                    .align_x(Alignment::End)
                    .into()
            } else {
                let rename_button = widget::button::icon(icons::rename_icon())
                    .tooltip(fl!("rename"))
                    .on_press(Message::RenameStart(addr.clone(), name.clone()))
                    .extra_small();

                let refresh_button = widget::button::icon(icons::reconnect_icon())
                    .tooltip(fl!("reconnect"))
                    .on_press(Message::ReconnectDevice(addr.clone()))
                    .extra_small();

                let remove_button = widget::button::icon(icons::remove_icon())
                    .tooltip(fl!("remove"))
                    .on_press(Message::RemoveDevice(addr.clone()))
                    .extra_small();

                let button_row = widget::row()
                    .spacing(8)
                    .push(rename_button)
                    .push(refresh_button)
                    .push(remove_button)
                    .width(Length::Shrink);

                widget::container(button_row)
                    .width(Length::FillPortion(4))
                    .align_x(Alignment::End)
                    .into()
            };

            let row = widget::row()
                .align_y(Alignment::Center)
                .spacing(8)
                .push(widget::text(label).width(Length::FillPortion(6)))
                .push(buttons);

            paired_list = paired_list.add(row);
        }
    }

    let mut content = widget::column()
        .padding(8)
        .spacing(8)
        .push(widget::text(fl!("connected-devices")))
        .push(connected_list)
        .push(widget::text(fl!("paired-devices")))
        .push(paired_list);

    if let Some(error) = &app.last_error {
        content = content.push(widget::text(fl!("error-loading", error = error)));
    }

    app.core.applet.popup_container(content).into()
}
