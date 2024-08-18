use glm::Vec3;
use imgui::internal;

use crate::ImguiId;

const MAX_NAME_WIDTH: f32 = 100.0;
const PADDING: f32 = 15.0;
const ITEM_WIDTH: f32 = 50.0;

const SCALAR_ITEM_NAME: [&str; 3] = ["X", "Y", "Z"];

pub fn display_scalar<T>(ui: &mut imgui::Ui, label: &str, curr_id: &mut ImguiId, scalar: &mut T)
where
    T: internal::DataTypeKind,
{
    let id = ui.push_id(curr_id.get_next_id());
    ui.text(label);
    ui.same_line_with_pos(MAX_NAME_WIDTH);
    ui.set_next_item_width(ITEM_WIDTH);
    ui.input_scalar("##hidden", scalar).build();
    id.end();
}

pub fn display_scalar_2<T: internal::DataTypeKind>(ui: &mut imgui::Ui, field_name: &str, curr_id: &mut ImguiId, scalar: &mut [T; 2]) {
    let cursor_y = ui.cursor_pos()[1];
    ui.set_cursor_pos([5.0, cursor_y]);
    ui.text(field_name);

    let mut current_position = MAX_NAME_WIDTH;
    for i in 0..2 {
        let id = ui.push_id(curr_id.get_next_id());

        ui.same_line_with_pos(current_position);
        ui.text(SCALAR_ITEM_NAME[i]);
        current_position += PADDING;

        ui.same_line_with_pos(current_position);
        ui.set_next_item_width(ITEM_WIDTH);
        ui.input_scalar("##hidden", &mut scalar[i]).build();

        current_position += ITEM_WIDTH + PADDING;

        id.end();
    }
}

pub fn display_scalar_3(ui: &mut imgui::Ui, field_name: &str, mut curr_id: &mut ImguiId, scalar: &mut Vec3) {
    let cursor_y = ui.cursor_pos()[1];
    ui.set_cursor_pos([5.0, cursor_y]);
    ui.text(field_name);

    let mut current_position = MAX_NAME_WIDTH;

    for i in 0..3 {
        let id = ui.push_id(curr_id.get_next_id());

        ui.same_line_with_pos(current_position);
        ui.text(SCALAR_ITEM_NAME[i]);
        current_position += PADDING;

        ui.same_line_with_pos(current_position);
        ui.set_next_item_width(ITEM_WIDTH);
        ui.input_scalar("##hidden", &mut scalar[i]).build();

        current_position += ITEM_WIDTH + PADDING;

        id.end();
    }
}

pub fn display_boolean(ui: &mut imgui::Ui, label: &str, curr_id: &mut ImguiId, scalar: &mut bool) {
    let id = ui.push_id(curr_id.get_next_id());
    ui.text(label);
    ui.same_line_with_pos(MAX_NAME_WIDTH);
    ui.checkbox("##hidden", scalar);
    id.end();
}

pub fn display_enum(ui: &mut imgui::Ui, label: &str, curr_id: &mut ImguiId, enum_types: &[&str], current_index: &mut i32, space: f32) {
    let id = ui.push_id(curr_id.get_next_id());
    ui.text(label);
    ui.same_line_with_pos(space);
    ui.list_box(label, current_index, enum_types, enum_types.len() as i32);
    id.end();
}

pub fn display_text(ui: &mut imgui::Ui, label: &str, curr_id: &mut ImguiId, input: &mut String) {
    let id = ui.push_id(curr_id.get_next_id());
    ui.text(label);
    ui.same_line_with_pos(MAX_NAME_WIDTH);
    ui.input_text("##",input);
    id.end();
}

pub fn display_slider<T>(ui: &mut imgui::Ui, label: &str, curr_id: &mut ImguiId, min: T, max: T, scalar: &mut T)
where
    T: internal::DataTypeKind,
{
    let id = ui.push_id(curr_id.get_next_id());
    ui.text(label);
    ui.same_line_with_pos(MAX_NAME_WIDTH);
    ui.slider(label, min, max, scalar);
    id.end();
}
