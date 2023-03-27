use colored::Colorize;

use crate::prompts::{
    confirm::Confirm,
    multi_select::MultiSelect,
    number::Number,
    password::Password,
    select::{Select, SelectOption},
    text::Text,
    toggle::Toggle,
};

use super::{
    num::Num,
    renderer::{DrawTime, Renderer},
};

pub fn fmt_text(prompt: &Text, renderer: &Renderer) -> (String, Option<(u16, u16)>) {
    (
        format!(
            "{} {}\n{} {}\n{}",
            message(prompt.message, &renderer.draw_time),
            text_default_value(&prompt.default_value),
            text_prefix(&prompt.validator_result),
            text_input(&prompt.input.value, &prompt.placeholder, false),
            text_error(&prompt.validator_result),
        ),
        Some((1, 2)),
    )
}

pub fn fmt_password(prompt: &Password, renderer: &Renderer) -> (String, Option<(u16, u16)>) {
    let text = match prompt.hidden {
        true => String::new(),
        false => "*".repeat(prompt.input.value.len()),
    };

    (
        format!(
            "{} {}\n{} {}\n{}",
            message(prompt.message, &renderer.draw_time),
            text_default_value(&prompt.default_value),
            text_prefix(&prompt.validator_result),
            text_input(&text, &prompt.placeholder, false),
            text_error(&prompt.validator_result),
        ),
        Some((1, 2)),
    )
}

pub fn fmt_number<T: Num>(prompt: &Number<T>, renderer: &Renderer) -> (String, Option<(u16, u16)>) {
    (
        format!(
            "{} {}\n{} {}\n{}",
            message(prompt.message, &renderer.draw_time),
            text_default_value(&prompt.default_value.as_deref()),
            text_prefix(&prompt.validator_result),
            text_input(&prompt.input.value, &prompt.placeholder, true),
            text_error(&prompt.validator_result),
        ),
        Some((1, 2)),
    )
}

pub fn fmt_toggle(prompt: &Toggle, renderer: &Renderer) -> String {
    format!(
        "{}\n{}  {}\n",
        message(prompt.message, &renderer.draw_time),
        toggle_option(prompt.options.0, !prompt.active),
        toggle_option(prompt.options.1, prompt.active),
    )
}

pub fn fmt_confirm(prompt: &Confirm, renderer: &Renderer) -> String {
    format!(
        "{}\n{}  {}\n",
        message(prompt.message, &renderer.draw_time),
        toggle_option("No", !prompt.active),
        toggle_option("Yes", prompt.active),
    )
}

pub fn fmt_select<T>(prompt: &Select<T>, renderer: &Renderer) -> String {
    let page_options: Vec<String> = select_format_options(
        &prompt.options,
        prompt.cursor.get_page(),
        prompt.cursor.items_per_page,
        prompt.cursor.focused,
        select_option,
    );

    format!(
        "{}\n{}{}{}\n",
        message(prompt.message, &renderer.draw_time),
        page_options.join("\n"),
        "\n".repeat(prompt.cursor.items_per_page - page_options.len()),
        select_pagination(prompt.cursor.get_page(), prompt.cursor.count_pages()),
    )
}

pub fn fmt_multi_select<T>(prompt: &MultiSelect<T>, renderer: &Renderer) -> String {
    let page_options: Vec<String> = select_format_options(
        &prompt.options,
        prompt.cursor.get_page(),
        prompt.cursor.items_per_page,
        prompt.cursor.focused,
        multi_select_option,
    );

    format!(
        "{} {}\n{}{}{}\n",
        message(prompt.message, &renderer.draw_time),
        min_max_message(prompt.min, prompt.max),
        page_options.join("\n"),
        "\n".repeat(prompt.cursor.items_per_page - page_options.len()),
        select_pagination(prompt.cursor.get_page(), prompt.cursor.count_pages()),
    )
}

// region: utils

#[inline]
fn text_prefix(validator_result: &Result<(), &str>) -> String {
    let error = match validator_result {
        Ok(_) => "›".blue(),
        Err(_) => "›".red(),
    };

    error.to_string()
}

#[inline]
fn text_default_value(default_value: &Option<&str>) -> String {
    let value = match default_value {
        Some(value) => format!("Default: {}", value).bright_black(),
        None => "".normal(),
    };

    value.to_string()
}

#[inline]
fn text_error(validator_result: &Result<(), &str>) -> String {
    match validator_result {
        Ok(_) => String::new(),
        Err(e) => format!("{}\n", e.red()),
    }
}

#[inline]
fn text_input(input: &str, placeholder: &Option<&str>, is_number: bool) -> String {
    let input = match (input.is_empty(), is_number) {
        (true, _) => placeholder.unwrap_or_default().bright_black(),
        (false, false) => input.normal(),
        (false, true) => input.yellow(),
    };

    input.to_string()
}

#[inline]
fn select_prefix(selected: bool) -> &'static str {
    match selected {
        true => "● ",
        false => "○ ",
    }
}

#[inline]
fn message(message: &str, draw_time: &DrawTime) -> String {
    let prefix = match draw_time {
        DrawTime::Last => "✓ ".green(),
        _ => "? ".blue(),
    };

    format!("{}{}", prefix, message)
}

#[inline]
fn min_max_message(min: Option<usize>, max: Option<usize>) -> String {
    match (min, max) {
        (None, None) => String::new(),
        (None, Some(max)) => format!("Max: {}", max),
        (Some(min), None) => format!("Min: {}", min),
        (Some(min), Some(max)) => format!("Min: {} · Max: {}", min, max),
    }
    .bright_black()
    .to_string()
}

#[inline]
fn toggle_option(option: &str, active: bool) -> String {
    let option = format!(" {} ", option);
    let option = match active {
        true => option.black().on_blue(),
        false => option.white().on_bright_black(),
    };

    option.to_string()
}

#[inline]
fn select_format_options<T, F>(
    options: &Vec<SelectOption<T>>,
    page: usize,
    items_per_page: usize,
    focused: usize,
    formatter: F,
) -> Vec<String>
where
    F: Fn(&SelectOption<T>, bool) -> String,
{
    let page_start_idx = page * items_per_page;
    let page_end_idx = (page_start_idx + items_per_page).min(options.len());
    let selected = focused % items_per_page;

    options[page_start_idx..page_end_idx]
        .iter()
        .enumerate()
        .map(|(i, option)| formatter(option, selected == i))
        .collect()
}

fn select_option_title<T>(option: &SelectOption<T>, focused: bool) -> String {
    let title = option.title;
    let title = match (option.disabled, focused) {
        (true, _) => title.bright_black().strikethrough(),
        (false, true) => title.blue(),
        (false, false) => title.normal(),
    };

    let make_description = |s: &str| format!(" · {}", s).bright_black();
    let description = match (focused, option.disabled, option.description) {
        (true, true, _) => make_description("(Disabled)"),
        (true, false, Some(description)) => make_description(description),
        _ => "".normal(),
    };

    format!("{} {}", title, description)
}

#[inline]
fn select_option<T>(option: &SelectOption<T>, focused: bool) -> String {
    let prefix = select_prefix(focused);
    let prefix = match (focused, option.disabled) {
        (false, _) => prefix.bright_black(),
        (true, true) => prefix.yellow(),
        (true, false) => prefix.blue(),
    };

    format!("{}{}", prefix, select_option_title(option, focused))
}

#[inline]
fn multi_select_option<T>(option: &SelectOption<T>, focused: bool) -> String {
    let prefix = select_prefix(option.active);
    let prefix = match (focused, option.disabled, option.active) {
        (true, true, _) => prefix.yellow(),
        (true, false, _) => prefix.blue(),
        (false, _, true) => prefix.normal(),
        (false, _, false) => prefix.bright_black(),
    };

    format!("{}{}", prefix, select_option_title(option, focused))
}

#[inline]
fn select_pagination(page: usize, pages: usize) -> String {
    if pages == 1 {
        return String::new();
    }

    let icon = "•";

    format!(
        "\n\n  {}{}{}",
        icon.repeat(page).bright_black(),
        icon,
        icon.repeat(pages.saturating_sub(page + 1)).bright_black(),
    )
}

// endregion: utils
