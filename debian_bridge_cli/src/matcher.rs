use clap::ArgMatches;

pub struct CommandMatcher<'a> {
    matches: &'a ArgMatches<'a>,
}

impl<'a> CommandMatcher<'a> {
    pub fn new(matches: &'a ArgMatches<'a>) -> Self {
        CommandMatcher { matches }
    }

    pub fn is_option_present<T, S>(&self, command: T, option: S) -> bool
    where
        T: Into<String>,
        S: Into<String>,
    {
        let command = command.into();
        let option = option.into();

        self.matches
            .subcommand_matches(&command)
            .expect(format!("No command '{}' presented", command).as_str())
            .is_present(option)
    }

    pub fn get_argument<T, S>(&self, command: T, arg: S) -> Option<String>
    where
        T: Into<String>,
        S: Into<String>,
    {
        let command = command.into();
        let arg = arg.into();

        self.matches
            .subcommand_matches(&command)
            .expect(format!("No command '{}' presented", command).as_str())
            .value_of(&arg)
            .map(|s| s.to_string())
    }
}
