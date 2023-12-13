use std::borrow::Cow;
use crate::ColoredStrings;
use colored::{ColoredString, Colorize};

use crate::prompts::{
    confirm::Confirm,
    message::Message,
    multi_select::MultiSelect,
    number::Number,
    password::Password,
    select::{Select, SelectInput, SelectOption},
    text::Text,
    toggle::Toggle,
};

use super::{num_like::NumLike, renderer::DrawTime};

pub fn fmt_confirm(prompt: &Confirm, draw_time: DrawTime, out: &mut ColoredStrings) {
    let options = ["No", "Yes"];

    if draw_time == DrawTime::Last {
        fmt_last_message2(&prompt.message, options[prompt.active as usize], out);
        return;
    }

    fmt_msg2(&prompt.message, out);
    out.0.push("\n".into());
    fmt_toggle_options3(options, prompt.active, out);
}

pub fn fmt_confirm2(prompt: &Confirm, draw_time: DrawTime, out: &mut ColoredStrings) {
    let options = ["No", "Yes"];

    if draw_time == DrawTime::Last {
        fmt_last_message2(&prompt.message, options[prompt.active as usize], out);
        return;
    }

    fmt_msg2(&prompt.message, out);
    out.0.push("\n".into());
    fmt_toggle_options3(options, prompt.active, out);
}

pub fn fmt_message2(prompt: &Message, _draw_time: DrawTime, out: &mut ColoredStrings) {
    out.push(prompt.message.as_ref().into());
}

// pub fn fmt_toggle(prompt: &Toggle, draw_time: DrawTime) -> String {
//     if draw_time == DrawTime::Last {
//         return fmt_last_message(&prompt.message, &prompt.options[prompt.active as usize]);
//     }

//     [
//         fmt_message(&prompt.message),
//         fmt_toggle_options(prompt.options, prompt.active),
//     ]
//     .join("\n")
// }

pub fn fmt_toggle2(prompt: &Toggle, draw_time: DrawTime, out: &mut ColoredStrings) {
    if draw_time == DrawTime::Last {
        fmt_last_message2(&prompt.message, &prompt.options[prompt.active as usize], out);
        return;
    }

    fmt_msg2(&prompt.message, out);
    out.push("\n".into());
    fmt_toggle_options2(&prompt.options, prompt.active, out);
}

// pub fn fmt_select<T>(prompt: &Select<T>, draw_time: DrawTime) -> String {
//     if draw_time == DrawTime::Last {
//         return fmt_last_message(&prompt.message, &prompt.options[prompt.input.focused].title);
//     }

//     [
//         fmt_message(&prompt.message),
//         fmt_select_page_options(&prompt.options, &prompt.input, false),
//         fmt_select_pagination(prompt.input.get_page(), prompt.input.count_pages()),
//     ]
//     .join("\n")
// }

pub fn fmt_select2<T>(prompt: &Select<T>, draw_time: DrawTime, out: &mut ColoredStrings) {
    if draw_time == DrawTime::Last {
        fmt_last_message2(
            &prompt.message,
            &prompt.options[prompt.input.focused].title,
            out,
        );
        return;
    }

    fmt_msg2(&prompt.message, out);
    out.push("\n".into());
    fmt_select_page_options2(&prompt.options, &prompt.input, false, out);
    out.push("\n".into());
    fmt_select_pagination2(prompt.input.get_page(), prompt.input.count_pages(), out);
}

pub fn fmt_multi_select<T>(prompt: &MultiSelect<T>, draw_time: DrawTime) -> String {
    if draw_time == DrawTime::Last {
        return fmt_last_message(
            prompt.message,
            &format!(
                "[{}]",
                prompt
                    .options
                    .iter()
                    .filter(|opt| opt.active)
                    .map(|opt| opt.title.as_str())
                    .collect::<Vec<_>>()
                    .join(", "),
            ),
        );
    }

    [
        fmt_multi_select_message(prompt.message, prompt.min, prompt.max),
        fmt_select_page_options(&prompt.options, &prompt.input, true),
        fmt_select_pagination(prompt.input.get_page(), prompt.input.count_pages()),
    ]
    .join("\n")
}

pub fn fmt_multi_select2<T>(
    prompt: &MultiSelect<T>,
    draw_time: DrawTime,
    out: &mut ColoredStrings,
) {
    if draw_time == DrawTime::Last {
        fmt_last_message2(
            prompt.message,
            &format!(
                "[{}]",
                prompt
                    .options
                    .iter()
                    .filter(|opt| opt.active)
                    .map(|opt| opt.title.as_str())
                    .collect::<Vec<_>>()
                    .join(", "),
            ),
            out,
        );
        return;
    }

    fmt_multi_select_message2(prompt.message, prompt.min, prompt.max, out);
    out.push("\n".into());
    fmt_select_page_options2(&prompt.options, &prompt.input, true, out);
    out.push("\n".into());
    fmt_select_pagination2(prompt.input.get_page(), prompt.input.count_pages(), out);
}

pub fn fmt_text(prompt: &Text, draw_time: DrawTime) -> (String, [usize; 2]) {
    if draw_time == DrawTime::Last {
        return (
            fmt_last_message(prompt.message, &prompt.input.value),
            [0, 0],
        );
    }

    (
        [
            fmt_line_message(prompt.message, &prompt.default_value),
            fmt_line_input(
                &prompt.input.value,
                &prompt.placeholder,
                &prompt.validator_result,
                false,
            ),
            fmt_line_validator(&prompt.validator_result),
        ]
        .join("\n"),
        get_cursor_position(prompt.input.col),
    )
}

pub fn fmt_text2(prompt: &Text, draw_time: DrawTime, out: &mut ColoredStrings) -> [usize; 2] {
    if draw_time == DrawTime::Last {
        fmt_last_message2(prompt.message, &prompt.input.value, out);
        return [0, 0];
    }

    fmt_line_message2(prompt.message, &prompt.default_value, out);
    out.push("\n".into());
    fmt_line_input2(
        &prompt.input.value,
        &prompt.placeholder,
        &prompt.validator_result,
        false,
        out,
    );
    out.push("\n".into());
    fmt_line_validator2(&prompt.validator_result, out);
    get_cursor_position(prompt.input.col)
}

pub fn fmt_password(prompt: &Password, draw_time: DrawTime) -> (String, [usize; 2]) {
    if draw_time == DrawTime::Last {
        return (fmt_last_message(prompt.message, "…"), [0, 0]);
    }

    let text = match prompt.hidden {
        true => String::new(),
        false => "*".repeat(prompt.input.value.len()),
    };

    let cursor_col = if prompt.hidden { 0 } else { prompt.input.col };

    (
        [
            fmt_line_message(prompt.message, &prompt.default_value),
            fmt_line_input(&text, &prompt.placeholder, &prompt.validator_result, false),
            fmt_line_validator(&prompt.validator_result),
        ]
        .join("\n"),
        get_cursor_position(cursor_col),
    )
}

pub fn fmt_password2(
    prompt: &Password,
    draw_time: DrawTime,
    out: &mut ColoredStrings,
) -> [usize; 2] {
    if draw_time == DrawTime::Last {
        fmt_last_message2(prompt.message, "…", out);
        return [0, 0];
    }

    let text = match prompt.hidden {
        true => String::new(),
        false => "*".repeat(prompt.input.value.len()),
    };

    let cursor_col = if prompt.hidden { 0 } else { prompt.input.col };

    fmt_line_message2(prompt.message, &prompt.default_value, out);
    out.push("\n".into());
    fmt_line_input2(
        &text,
        &prompt.placeholder,
        &prompt.validator_result,
        false,
        out,
    );
    out.push("\n".into());
    fmt_line_validator2(&prompt.validator_result, out);
    get_cursor_position(cursor_col)
}

pub fn fmt_number<T: NumLike>(prompt: &Number<T>, draw_time: DrawTime) -> (String, [usize; 2]) {
    if draw_time == DrawTime::Last {
        return (
            fmt_last_message(prompt.message, &prompt.input.value),
            [0, 0],
        );
    }

    (
        [
            fmt_line_message(
                prompt.message,
                &prompt.default_value.map(|x| x.to_string()).as_deref(),
            ),
            fmt_line_input(
                &prompt.input.value,
                &prompt.placeholder,
                &prompt.validator_result,
                true,
            ),
            fmt_line_validator(&prompt.validator_result),
        ]
        .join("\n"),
        get_cursor_position(prompt.input.col),
    )
}

pub fn fmt_number2<T: NumLike>(
    prompt: &Number<T>,
    draw_time: DrawTime,
    out: &mut ColoredStrings,
) -> [usize; 2] {
    if draw_time == DrawTime::Last {
        fmt_last_message2(prompt.message, &prompt.input.value, out);
        return [0, 0];
    }
    fmt_line_message2(
        prompt.message,
        &prompt.default_value.map(|x| x.to_string()).as_deref(),
        out,
    );
    out.push("\n".into());
    fmt_line_input2(
        &prompt.input.value,
        &prompt.placeholder,
        &prompt.validator_result,
        false,
        out,
    );
    out.push("\n".into());
    fmt_line_validator2(&prompt.validator_result, out);
    get_cursor_position(prompt.input.col)
}

// region: general

fn fmt_message(message: &str) -> String {
    format!("{} {}", "▣".blue(), message)
}

pub fn fmt_msg2(message: &str, out: &mut ColoredStrings) {
    out.extend(["▣".blue(), " ".into(), message.into()]);
}

pub fn fmt_echo(message: &str, out: &mut ColoredStrings) {
    out.push(message.into());
}

fn fmt_last_message(message: &str, answer: &str) -> String {
    format!("{} {} {}", "■".green(), message, answer.purple())
}

pub fn fmt_last_message2(message: &str, answer: &str, out: &mut ColoredStrings) {
    out.extend([
        "■".green(),
        " ".into(),
        message.into(),
        " ".into(),
        answer.to_owned().purple(),
    ]);
}

// endregion: general

// region: toggle

// fn fmt_toggle_options<'a>(options: [Cow<'a, str>; 2], active: bool) -> String {
//     let mut out = ColoredStrings::default();
//     fmt_toggle_options2(options, active, &mut out);
//     format!("{}", out)
// }
fn fmt_toggle_options2<'a>(options: &[Cow<'a, str>; 2], active: bool, out: &mut ColoredStrings) {
    let fmt_option = |opt, active| {
        let opt = format!(" {} ", opt);
        match active {
            true => opt.black().on_blue(),
            false => opt.white().on_bright_black(),
        }
    };

    out.extend([
        fmt_option(options[0].as_ref(), !active),
        " ".into(),
        fmt_option(options[1].as_ref(), active),
    ]);
}

pub fn fmt_toggle_options3(options: [&str; 2], active: bool, out: &mut ColoredStrings) {
    let fmt_option = |opt, active| {
        let opt = format!(" {} ", opt);
        match active {
            true => opt.black().on_blue(),
            false => opt.white().on_bright_black(),
        }
    };

    out.extend([
        fmt_option(options[0], !active),
        " ".into(),
        fmt_option(options[1], active),
    ]);
}

pub fn fmt_toggle_options4(options: &[&str], active: usize, out: &mut ColoredStrings) {
    let fmt_option = |opt, active| {
        let opt = format!(" {} ", opt);
        match active {
            true => opt.black().on_blue(),
            false => opt.white().on_bright_black(),
        }
    };
    for i in 0..options.len() {
        out.push(fmt_option(options[i], i == active));
        if i < options.len() - 1 {
            out.push(" ".into());
        }
    }
}

// endregion: toggle

// region: line

fn fmt_line_message2(msg: &str, default_value: &Option<&str>, out: &mut ColoredStrings) {
    let value = match default_value {
        Some(value) => format!("Default: {}", value).bright_black(),
        None => "".normal(),
    };
    fmt_msg2(msg, out);
    out.push(" ".into());
    out.push(value);
}

fn fmt_line_message(msg: &str, default_value: &Option<&str>) -> String {
    let value = match default_value {
        Some(value) => format!("Default: {}", value).bright_black(),
        None => "".normal(),
    };

    format!("{} {}", fmt_message(msg), value)
}

fn fmt_line_input(
    input: &str,
    placeholder: &Option<&str>,
    validator_result: &Result<(), &str>,
    is_number: bool,
) -> String {
    let prefix = match validator_result {
        Ok(_) => "›".blue(),
        Err(_) => "›".red(),
    };

    let input = match (input.is_empty(), is_number) {
        (true, _) => placeholder.unwrap_or_default().bright_black(),
        (false, true) => input.yellow(),
        (false, false) => input.normal(),
    };

    format!("{} {}", prefix, input)
}

fn fmt_line_input2(
    input: &str,
    placeholder: &Option<&str>,
    validator_result: &Result<(), &str>,
    is_number: bool,
    out: &mut ColoredStrings,
) {
    let prefix = match validator_result {
        Ok(_) => "›".blue(),
        Err(_) => "›".red(),
    };

    let input = input.to_owned();
    let input = match (input.is_empty(), is_number) {
        (true, _) => placeholder
            .map(|s| s.to_owned())
            .unwrap_or_default()
            .bright_black(),
        (false, true) => input.yellow(),
        (false, false) => input.normal(),
    };
    out.push(prefix);
    out.push(" ".into());
    out.push(input);
}

fn fmt_line_validator(validator_result: &Result<(), &str>) -> String {
    match validator_result {
        Ok(_) => String::new(),
        Err(e) => format!("{}", e.red()),
    }
}

fn fmt_line_validator2(validator_result: &Result<(), &str>, out: &mut ColoredStrings) {
    if let Err(e) = validator_result {
        out.push(e.to_string().red());
    }
}

fn get_cursor_position(cursor_col: usize) -> [usize; 2] {
    let x = 2 + cursor_col;
    let y = 1;

    [x, y]
}

// endregion: line

// region: select

fn fmt_multi_select_message(msg: &str, min: Option<usize>, max: Option<usize>) -> String {
    let min_max = match (min, max) {
        (None, None) => String::new(),
        (None, Some(max)) => format!("Max: {}", max),
        (Some(min), None) => format!("Min: {}", min),
        (Some(min), Some(max)) => format!("Min: {} · Max: {}", min, max),
    }
    .bright_black();

    format!("{} {}", fmt_message(msg), min_max)
}

fn fmt_multi_select_message2(
    msg: &str,
    min: Option<usize>,
    max: Option<usize>,
    out: &mut ColoredStrings,
) {
    let min_max = match (min, max) {
        (None, None) => String::new(),
        (None, Some(max)) => format!("Max: {}", max),
        (Some(min), None) => format!("Min: {}", min),
        (Some(min), Some(max)) => format!("Min: {} · Max: {}", min, max),
    }
    .bright_black();

    fmt_msg2(msg, out);
    out.push(" ".into());
    out.push(min_max);
}

fn fmt_select_page_options<T>(
    options: &[SelectOption<T>],
    input: &SelectInput,
    is_multiple: bool,
) -> String {
    let items_per_page = input.items_per_page;
    let total = input.total_items;

    let page_len = items_per_page.min(total);
    let page_start = input.get_page() * items_per_page;
    let page_end = (page_start + page_len).min(total);
    let page_focused = input.focused % items_per_page;

    let mut page_options: Vec<String> = options[page_start..page_end]
        .iter()
        .enumerate()
        .map(|(i, option)| fmt_select_option(option, page_focused == i, is_multiple))
        .collect();

    page_options.resize(page_len, String::new());
    page_options.join("\n")
}

fn fmt_select_page_options2<T>(
    options: &[SelectOption<T>],
    input: &SelectInput,
    is_multiple: bool,
    out: &mut ColoredStrings,
) {
    let items_per_page = input.items_per_page;
    let total = input.total_items;

    let page_len = items_per_page.min(total);
    let page_start = input.get_page() * items_per_page;
    let page_end = (page_start + page_len).min(total);
    let page_focused = input.focused % items_per_page;

    let mut page_options: Vec<Vec<ColoredString>> = options[page_start..page_end]
        .iter()
        .enumerate()
        .map(|(i, option)| {
            let mut o = ColoredStrings::default();
            fmt_select_option2(option, page_focused == i, is_multiple, &mut o);
            o.0
        })
        .collect();

    page_options.resize(page_len, vec![]);
    for page in page_options {
        out.push("\n".into());
        out.extend(page);
    }
    // let joined: Vec<ColoredString> = page_options.join::<ColoredString>("\n".into());
    // out.extend(joined);
}

fn fmt_select_pagination(page: usize, pages: usize) -> String {
    if pages == 1 {
        return String::new();
    }

    let icon = "•";

    format!(
        "\n  {}{}{}",
        icon.repeat(page).bright_black(),
        icon,
        icon.repeat(pages.saturating_sub(page + 1)).bright_black(),
    )
}

fn fmt_select_pagination2(page: usize, pages: usize, out: &mut ColoredStrings) {
    if pages == 1 {
        return;
    }

    let icon = "•";

    out.extend([
        "\n  ".into(),
        icon.repeat(page).bright_black(),
        icon.into(),
        icon.repeat(pages.saturating_sub(page + 1)).bright_black(),
    ]);
}

fn fmt_select_option<T>(option: &SelectOption<T>, focused: bool, multiple: bool) -> String {
    let prefix = if multiple {
        let prefix = match (option.active, focused) {
            (true, true) => "◉",
            (true, false) => "●",
            _ => "○",
        };

        match (focused, option.active, option.disabled) {
            (true, _, true) => prefix.red(),
            (true, _, false) => prefix.blue(),
            (false, true, _) => prefix.normal(),
            (false, false, _) => prefix.bright_black(),
        }
    } else {
        match (focused, option.disabled) {
            (false, _) => "○".bright_black(),
            (true, true) => "○".red(),
            (true, false) => "●".blue(),
        }
    };

    let title = &option.title;
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

    format!("{} {} {}", prefix, title, description)
}

fn fmt_select_option2<T>(
    option: &SelectOption<T>,
    focused: bool,
    multiple: bool,
    out: &mut ColoredStrings,
) {
    let prefix = if multiple {
        let prefix = match (option.active, focused) {
            (true, true) => "◉",
            (true, false) => "●",
            _ => "○",
        };

        match (focused, option.active, option.disabled) {
            (true, _, true) => prefix.red(),
            (true, _, false) => prefix.blue(),
            (false, true, _) => prefix.normal(),
            (false, false, _) => prefix.bright_black(),
        }
    } else {
        match (focused, option.disabled) {
            (false, _) => "○".bright_black(),
            (true, true) => "○".red(),
            (true, false) => "●".blue(),
        }
    };

    let title = option.title.to_owned();
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

    out.extend([prefix, " ".into(), title, " ".into(), description]);
}

// endregion: select
