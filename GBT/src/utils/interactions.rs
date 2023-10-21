use anyhow::Error;
use dialoguer::{theme::ColorfulTheme, Input, InputValidator, MultiSelect, Confirm};

pub fn get_input_string_with_validator<V>(
    prompt: &str,
    default: Option<&str>,
    validator: V,
) -> String
where
    V: InputValidator<String>,
    V::Err: ToString,
{
    let theme = ColorfulTheme::default();
    let mut input = Input::with_theme(&theme)
        .with_prompt(prompt)
        .allow_empty(true);
    if default.is_some() {
        input = input.default(default.unwrap().to_string());
    }
    let res = input.validate_with(validator).interact_text().unwrap();
    return res;
}

pub fn get_input_string(prompt: &str, default: Option<&str>) -> String {
    let theme = ColorfulTheme::default();
    let mut input = Input::with_theme(&theme).with_prompt(prompt);
    if default.is_some() {
        input = input.default(default.unwrap().to_string());
    }
    let res = input.interact_text().unwrap();
    return res;
}

pub fn get_multi_input(prompt: &str, items: Vec<&str>, default: Option<Vec<bool>>) -> Vec<String> {
    let sel;
    loop {
        let theme = ColorfulTheme::default();
        let mut input = MultiSelect::with_theme(&theme)
            .with_prompt(prompt)
            .items(&items).report(false);
        if default.as_ref().is_some() {
            input = input.defaults(default.as_ref().unwrap());
        }
        let selection = input.interact().unwrap();
        if !selection.is_empty() {
            sel = selection;
            break;
        }
    }
    let mut res = vec![];
    for selected in sel {
        res.push(items[selected].to_string());
    }
    return res;
}

pub fn get_confirm(prompt: &str, default: bool) -> bool {
    let input = Confirm::with_theme(&ColorfulTheme::default())
    .with_prompt(prompt)
    .default(default)
    .interact()
    .unwrap();
    return input;
}

#[macro_export]
macro_rules! get_confirm {
    ($a: expr) => {
        crate::utils::interactions::get_confirm($a, false)
    };
    ($a: expr, $b: expr) => {
        crate::utils::interactions::get_confirm($a, $b)
    };
}
