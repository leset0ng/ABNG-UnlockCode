use crate::astrobox::psys_host::{self, ui};
use std::sync::{Mutex, OnceLock};

pub const MAC_INPUT_EVENT: &str = "mac_input";
pub const SN_INPUT_EVENT: &str = "sn_input";
pub const TOGGLE_AGREE_EVENT: &str = "toggle_agree";
pub const CALCULATE_EVENT: &str = "calculate_unlock_code";

struct UiState {
    root_element_id: Option<String>,
    mac_input: String,
    sn_input: String,
    agreed: bool,
    unlock_code: Option<String>,
}

struct UiViewState {
    mac_input: String,
    sn_input: String,
    agreed: bool,
    unlock_code: Option<String>,
}

static UI_STATE: OnceLock<Mutex<UiState>> = OnceLock::new();

fn ui_state() -> &'static Mutex<UiState> {
    UI_STATE.get_or_init(|| {
        Mutex::new(UiState {
            root_element_id: None,
            mac_input: String::new(),
            sn_input: String::new(),
            agreed: false,
            unlock_code: None,
        })
    })
}

fn should_enable_calculate(state: &UiViewState) -> bool {
    state.agreed && !state.mac_input.trim().is_empty() && !state.sn_input.trim().is_empty()
}

pub fn ui_event_processor(evtype: ui::Event, event: &str, payload: &str) {
    let (root_element_id, view_state) = {
        let mut state = ui_state()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        match evtype {
            ui::Event::Input | ui::Event::Change => match event {
                MAC_INPUT_EVENT => {
                    state.mac_input = payload.to_string();
                }
                SN_INPUT_EVENT => {
                    state.sn_input = payload.to_string();
                }
                _ => {}
            },
            ui::Event::Click => match event {
                TOGGLE_AGREE_EVENT => {
                    state.agreed = !state.agreed;
                }
                CALCULATE_EVENT => {
                    let view_state = UiViewState {
                        mac_input: state.mac_input.clone(),
                        sn_input: state.sn_input.clone(),
                        agreed: state.agreed,
                        unlock_code: state.unlock_code.clone(),
                    };

                    if should_enable_calculate(&view_state) {
                        let mac = view_state.mac_input.to_uppercase();
                        let sn = view_state.sn_input.to_uppercase();
                        state.unlock_code = Some(crate::calc_unlock_code(mac, sn));
                    }
                }
                _ => {}
            },
            _ => {}
        }

        (
            state.root_element_id.clone(),
            UiViewState {
                mac_input: state.mac_input.clone(),
                sn_input: state.sn_input.clone(),
                agreed: state.agreed,
                unlock_code: state.unlock_code.clone(),
            },
        )
    };

    if let Some(root_element_id) = root_element_id {
        psys_host::ui::render(&root_element_id, build_main_ui(&view_state));
    }
}

fn build_result(code: &str) -> ui::Element {
    let mut digits_row = ui::Element::new(ui::ElementType::Div, None)
        .flex()
        .flex_direction(ui::FlexDirection::Row)
        .justify_center()
        .align_center()
        .margin_top(6);

    for digit in code.chars() {
        let digit_string = digit.to_string();
        let digit_box = ui::Element::new(ui::ElementType::Div, Some(digit_string.as_str()))
            .flex()
            .justify_center()
            .align_center()
            .width(36)
            .height(36)
            .border(1, "#d0d0d0")
            .radius(4)
            .size(22)
            .margin(2);
        digits_row = digits_row.child(digit_box);
    }

    ui::Element::new(ui::ElementType::Div, None)
        .flex()
        .flex_direction(ui::FlexDirection::Column)
        .align_center()
        .justify_center()
        .padding(8)
        .radius(8)
        .border(1, "#e0e0e0")
        .child(ui::Element::new(ui::ElementType::P, Some("解锁码结果")).size(16))
        .child(digits_row)
}

fn build_main_ui(view_state: &UiViewState) -> ui::Element {
    let header = ui::Element::new(ui::ElementType::Div, None)
        .flex()
        .flex_direction(ui::FlexDirection::Column)
        .align_center()
        .margin_bottom(12)
        .child(ui::Element::new(ui::ElementType::P, Some("解锁码计算")).size(24))
        .child(
            ui::Element::new(ui::ElementType::P, Some("请输入 MAC 与序列号"))
                .size(14)
                .opacity(0.7),
        );

    let mut root = ui::Element::new(ui::ElementType::Div, None)
        .flex()
        .flex_direction(ui::FlexDirection::Column)
        .width_full()
        .padding(16)
        .align_start()
        .child(header);

    if let Some(code) = &view_state.unlock_code {
        root = root.child(build_result(code));
    }

    let mac_label = ui::Element::new(ui::ElementType::P, Some("MAC"))
        .size(14)
        .margin_top(12);
    let mac_input = ui::Element::new(ui::ElementType::Input, Some(view_state.mac_input.as_str()))
        .width_full()
        .on(ui::Event::Input, MAC_INPUT_EVENT)
        .on(ui::Event::Change, MAC_INPUT_EVENT);

    let sn_label = ui::Element::new(ui::ElementType::P, Some("SN"))
        .size(14)
        .margin_top(10);
    let sn_input = ui::Element::new(ui::ElementType::Input, Some(view_state.sn_input.as_str()))
        .width_full()
        .on(ui::Event::Input, SN_INPUT_EVENT)
        .on(ui::Event::Change, SN_INPUT_EVENT);

    let warning_title = ui::Element::new(ui::ElementType::P, Some("! 注意"))
        .size(16)
        .text_color("#c50f1f")
        .margin_top(12);
    let warning_body = ui::Element::new(
        ui::ElementType::P,
        Some("仅用于合法设备，请勿用于非法用途。"),
    )
    .size(14)
    .opacity(0.8);

    let agree_label = if view_state.agreed {
        "[x] 我已知晓"
    } else {
        "[ ] 我已知晓"
    };
    let agree_button = ui::Element::new(ui::ElementType::Button, Some(agree_label))
        .without_default_styles()
        .border(1, "#c0c0c0")
        .radius(4)
        .padding(6)
        .margin_top(10)
        .on(ui::Event::Click, TOGGLE_AGREE_EVENT);

    let can_calculate = should_enable_calculate(view_state);
    let mut calculate_button = ui::Element::new(ui::ElementType::Button, Some("计算"))
        .width_full()
        .margin_top(10)
        .on(ui::Event::Click, CALCULATE_EVENT);
    if !can_calculate {
        calculate_button = calculate_button.disabled().opacity(0.6);
    }

    root.child(mac_label)
        .child(mac_input)
        .child(sn_label)
        .child(sn_input)
        .child(warning_title)
        .child(warning_body)
        .child(agree_button)
        .child(calculate_button)
}

pub fn render_main_ui(element_id: &str) {
    let view_state = {
        let mut state = ui_state()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.root_element_id = Some(element_id.to_string());
        UiViewState {
            mac_input: state.mac_input.clone(),
            sn_input: state.sn_input.clone(),
            agreed: state.agreed,
            unlock_code: state.unlock_code.clone(),
        }
    };

    psys_host::ui::render(element_id, build_main_ui(&view_state));
}
