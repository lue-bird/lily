#![feature(deref_patterns)]
#![allow(incomplete_features, non_shorthand_field_patterns)]
mod lily;

fn main() -> iced::Result {
    iced::application(lily::initial_state, update_state_on_event, state_to_ui).run()
}
fn update_state_on_event(state: &mut lily::State, event: lily::Event) {
    *state = lily::update(event, std::mem::replace(state, lily::initial_state()));
}
fn state_to_ui(state: &lily::State) -> iced::Element<'_, lily::Event> {
    iced_element_from_lily(&lily::view(*state))
}
fn iced_element_from_lily<'a, Event: Clone + 'a>(
    lily_widget: &lily::Widget<Event>,
) -> iced::Element<'a, Event> {
    match lily_widget {
        lily::Widget::Text(content) => iced::widget::text(lily_str_to_cow(content)).into(),
        lily::Widget::Button(button) => {
            iced::widget::button(iced::widget::text(lily_str_to_cow(&button.label)))
                .on_press((button.on_press)(lily::Blank {}))
                .padding(iced_padding_from_lily(button.padding))
                .into()
        }
        lily::Widget::PickList(pick_list) => {
            let on_select = pick_list.on_select.clone();
            iced::widget::PickList::new(
                pick_list
                    .options
                    .iter()
                    .map(lily_str_to_cow)
                    .collect::<Vec<_>>(),
                pick_list
                    .selected
                    .as_ref()
                    .into_option()
                    .map(lily_str_to_cow),
                move |new_selected| on_select(cow_str_to_lily(new_selected)),
            )
            .into()
        }
        lily::Widget::Container(container) => {
            iced::widget::Container::new(iced_element_from_lily(&container.sub))
                .padding(iced_padding_from_lily(container.padding))
                .center_x(iced::Length::Shrink)
                .align_top(iced::Length::Shrink)
                .into()
        }
        lily::Widget::Column(column) => {
            iced::widget::Column::with_children(column.subs.iter().map(iced_element_from_lily))
                .spacing(column.spacing as f32)
                .into()
        }
        lily::Widget::Row(row) => {
            iced::widget::Row::with_children(row.subs.iter().map(iced_element_from_lily))
                .spacing(row.spacing as f32)
                .into()
        }
    }
}
fn iced_padding_from_lily(padding: lily::Padding) -> iced::Padding {
    iced::Padding {
        right: padding.right as f32,
        top: padding.top as f32,
        left: padding.left as f32,
        bottom: padding.bottom as f32,
    }
}
fn cow_str_to_lily(content: std::borrow::Cow<'static, str>) -> lily::Str {
    match content {
        std::borrow::Cow::Borrowed(str) => lily::Str::Slice(str),
        std::borrow::Cow::Owned(string) => lily::Str::from_string(string),
    }
}
fn lily_str_to_cow<'a>(content: &lily::Str) -> std::borrow::Cow<'a, str> {
    match content {
        lily::Str::Slice(str) => std::borrow::Cow::Borrowed(str),
        lily::Str::Rc(rc) => std::borrow::Cow::Owned(rc.to_string()),
    }
}
