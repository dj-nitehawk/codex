//! Fork-only `/new [name] ~ prompt` syntax, independent of session startup.

/// Split at the first standalone tilde. Names without a separator stay unchanged.
pub(crate) fn parse(args: &str) -> Result<(Option<String>, Option<String>), &'static str> {
    let separator = args.char_indices().find(|(index, ch)| {
        *ch == '~'
            && args[..*index]
                .chars()
                .next_back()
                .is_none_or(char::is_whitespace)
            && args[*index + 1..]
                .chars()
                .next()
                .is_none_or(char::is_whitespace)
    });
    let Some((index, _)) = separator else {
        return Ok((nonempty(args), None));
    };
    let prompt = args[index + 1..].trim();
    if prompt.is_empty() {
        return Err("Usage: /new [name] ~ <initial prompt>");
    }
    Ok((nonempty(&args[..index]), Some(prompt.to_string())))
}

fn nonempty(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_string())
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn new_session_prompt_syntax() {
        for (input, name, prompt) in [
            ("", None, None),
            (" My chat ", Some("My chat"), None),
            ("~/project", Some("~/project"), None),
            ("a~b", Some("a~b"), None),
            ("name ~prompt", Some("name ~prompt"), None),
            ("~ Explain this", None, Some("Explain this")),
            ("My chat ~ Fix login", Some("My chat"), Some("Fix login")),
            (
                "name\t~\nFirst line\n  second ~ third",
                Some("name"),
                Some("First line\n  second ~ third"),
            ),
            ("~ /review `$HOME` ", None, Some("/review `$HOME`")),
        ] {
            assert_eq!(
                parse(input),
                Ok((name.map(str::to_string), prompt.map(str::to_string)))
            );
        }
    }

    #[test]
    fn new_session_prompt_rejects_empty_prompt() {
        for input in ["~", "name ~", "name ~ \n\t"] {
            assert!(parse(input).is_err(), "{input:?}");
        }
    }
}
