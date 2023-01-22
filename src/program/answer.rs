use dialoguer::{theme::ColorfulTheme, Select};

#[derive(Debug, PartialEq, Eq)]
pub enum Answer {
    No,
    Yes,
}

impl From<usize> for Answer {
    fn from(network: usize) -> Self {
        match network {
            0 => Answer::No,
            1 => Answer::Yes,
            _ => panic!("Wrong answer"),
        }
    }
}

pub fn approve_action(prompt: &str, selected: usize) -> bool {
    let selected = if selected > 1 { 0 } else { selected };
    let answer: Answer = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .default(selected)
        .item("No")
        .item("Yes")
        .interact()
        .unwrap()
        .into();

    answer.eq(&Answer::Yes)
}
