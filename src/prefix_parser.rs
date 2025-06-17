#[derive(Debug, Clone)]
pub struct Arguments<'a> {
    current_iter: std::str::SplitWhitespace<'a>,
    remaining_slice_for_remainder: &'a str,
}

impl<'a> Arguments<'a> {
    pub fn new(args_str: &'a str) -> Self {
        Arguments {
            current_iter: args_str.split_whitespace(),
            remaining_slice_for_remainder: args_str,
        }
    }

    pub fn remainder(&self) -> &'a str {
        self.remaining_slice_for_remainder.trim_start()
    }
}

impl<'a> Iterator for Arguments<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let arg = self.current_iter.next()?;

        let effective_remainder_start = self.remaining_slice_for_remainder.trim_start();

        if let Some(pos) = effective_remainder_start.find(arg) {
            if pos == 0 {
                let leading_whitespace_len = effective_remainder_start.as_ptr() as usize
                    - self.remaining_slice_for_remainder.as_ptr() as usize;
                let advance_by = leading_whitespace_len + arg.len();

                if advance_by <= self.remaining_slice_for_remainder.len() {
                    self.remaining_slice_for_remainder =
                        &self.remaining_slice_for_remainder[advance_by..];
                } else {
                    self.remaining_slice_for_remainder = "";
                }
            } else {
                self.remaining_slice_for_remainder = "";
            }
        } else {
            self.remaining_slice_for_remainder = "";
        }

        Some(arg)
    }
}

#[derive(Debug, PartialEq)]
pub struct ParsedCommand<'a> {
    pub command: &'a str,
    args_part: &'a str,
}

impl<'a> ParsedCommand<'a> {
    pub fn arguments(&self) -> Arguments<'a> {
        Arguments::new(self.args_part)
    }
}

pub fn parse<'a>(message: &'a str, prefix: &str) -> Option<ParsedCommand<'a>> {
    if !message.starts_with(prefix) {
        return None;
    }

    let content_after_prefix = message.strip_prefix(prefix)?;
    let trimmed_content = content_after_prefix.trim_start();

    if trimmed_content.is_empty() {
        return None;
    }

    let mut parts = trimmed_content.splitn(2, char::is_whitespace);
    let command = parts.next().unwrap();
    let args_part = parts.next().unwrap_or("").trim_end();

    Some(ParsedCommand { command, args_part })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_command() {
        let result = parse("!echo hello world", "!").unwrap();
        assert_eq!(result.command, "echo");

        let mut args = result.arguments();
        assert_eq!(args.next(), Some("hello"));
        assert_eq!(args.remainder(), "world");
        assert_eq!(args.next(), Some("world"));
        assert_eq!(args.remainder(), "");
        assert_eq!(args.next(), None);
        assert_eq!(args.remainder(), "");
    }

    #[test]
    fn test_parse_command_with_extra_spaces() {
        let result = parse("!play  song  title with spaces  ", "!").unwrap();
        assert_eq!(result.command, "play");

        let mut args = result.arguments();
        assert_eq!(args.next(), Some("song"));
        assert_eq!(args.remainder(), "title with spaces");
        assert_eq!(args.next(), Some("title"));
        assert_eq!(args.remainder(), "with spaces");
        assert_eq!(args.next(), Some("with"));
        assert_eq!(args.remainder(), "spaces");
        assert_eq!(args.next(), Some("spaces"));
        assert_eq!(args.remainder(), "");
        assert_eq!(args.next(), None);
    }

    #[test]
    fn test_parse_command_prefix_with_spaces_after() {
        let result = parse("!  spaced_cmd arg1 arg2", "!").unwrap();
        assert_eq!(result.command, "spaced_cmd");
        let mut args = result.arguments();
        assert_eq!(args.next(), Some("arg1"));
        assert_eq!(args.remainder(), "arg2");
        assert_eq!(args.next(), Some("arg2"));
        assert_eq!(args.remainder(), "");
        assert_eq!(args.next(), None);
    }

    #[test]
    fn test_parse_no_args() {
        let result = parse("!kick", "!").unwrap();
        assert_eq!(result.command, "kick");
        let mut args = result.arguments();
        assert_eq!(args.next(), None);
        assert_eq!(args.remainder(), "");
    }

    #[test]
    fn test_parse_no_args_with_spaces() {
        let result = parse("!kick  ", "!").unwrap();
        assert_eq!(result.command, "kick");
        let mut args = result.arguments();
        assert_eq!(args.next(), None);
        assert_eq!(args.remainder(), "");
    }

    #[test]
    fn test_no_prefix() {
        assert!(parse("echo hello world", "!").is_none());
    }

    #[test]
    fn test_wrong_prefix() {
        assert!(parse("$echo hello world", "!").is_none());
    }

    #[test]
    fn test_only_prefix() {
        assert!(parse("!", "!").is_none());
    }

    #[test]
    fn test_prefix_and_spaces() {
        assert!(parse("!   ", "!").is_none());
    }

    #[test]
    fn test_empty_message() {
        assert!(parse("", "!").is_none());
    }

    #[test]
    fn test_arguments_iterator_multiple_calls() {
        let parsed = parse("!cmd arg1 arg2 arg3", "!").unwrap();

        let mut args1 = parsed.arguments();
        assert_eq!(args1.next(), Some("arg1"));
        assert_eq!(args1.remainder(), "arg2 arg3");
        assert_eq!(args1.next(), Some("arg2"));
        assert_eq!(args1.remainder(), "arg3");

        let mut args2 = parsed.arguments();
        assert_eq!(args2.next(), Some("arg1"));
        assert_eq!(args2.remainder(), "arg2 arg3");
        assert_eq!(args2.next(), Some("arg2"));
        assert_eq!(args2.remainder(), "arg3");
        assert_eq!(args2.next(), Some("arg3"));
        assert_eq!(args2.remainder(), "");
        assert_eq!(args2.next(), None);
    }

    #[test]
    fn test_remainder_before_next() {
        let parsed = parse("!cmd arg1 arg2 arg3", "!").unwrap();
        let args = parsed.arguments();
        assert_eq!(args.remainder(), "arg1 arg2 arg3");
    }

    #[test]
    fn test_remainder_after_all_next() {
        let parsed = parse("!cmd arg1", "!").unwrap();
        let mut args = parsed.arguments();
        assert_eq!(args.next(), Some("arg1"));
        assert_eq!(args.remainder(), "");
        assert_eq!(args.next(), None);
        assert_eq!(args.remainder(), "");
    }

    #[test]
    fn test_user_example() {
        let parsed = parse("!echo hello im here!", "!").unwrap();
        assert_eq!(parsed.command, "echo");
        let mut args = parsed.arguments();
        assert_eq!(args.next(), Some("hello"));
        assert_eq!(args.remainder(), "im here!");
        assert_eq!(args.next(), Some("im"));
        assert_eq!(args.remainder(), "here!");
        assert_eq!(args.next(), Some("here!"));
        assert_eq!(args.remainder(), "");
        assert_eq!(args.next(), None);
    }

    #[test]
    fn test_args_str_with_internal_multiple_spaces() {
        let parsed = parse("!cmd  first   second  ", "!").unwrap();
        assert_eq!(parsed.command, "cmd");
        let mut args = parsed.arguments();
        assert_eq!(args.next(), Some("first"));
        assert_eq!(args.remainder(), "second");
        assert_eq!(args.next(), Some("second"));
        assert_eq!(args.remainder(), "");
        assert_eq!(args.next(), None);
    }
}
