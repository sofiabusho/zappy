//! Server command-line parsing (S01).
//!
//! Contract (RQ17 / AQ01): the server accepts
//! `-p <port> -x <width> -y <height> -n <team> [<team> ...] -c <nb> [-t <t>]`.
//! All flags are required except `-t`, which defaults to [`DEFAULT_T`] (100).
//! Missing or invalid arguments produce a [`CliError`]; the binary prints
//! [`USAGE`] and exits non-zero (see `src/main.rs`).

use std::fmt;

/// Default time-unit divider when `-t` is omitted (RQ10).
pub const DEFAULT_T: u32 = 100;

/// Usage text, matching the sample in `docs/raw/audit.md` (AQ01) line-for-line
/// so auditors see the expected `./server` output.
pub const USAGE: &str = concat!(
    " Usage: ./server -p <port> -x <width> -y <height> -n <team> [<team>] [<team>] ... -c <nb> [-t <t>]\n",
    " -p port number\n",
    " -x world width\n",
    " -y world height\n",
    " -n team_name_1 team_name_2 ...\n",
    " -c number of clients authorized at the beginning of the game\n",
    " -t [100] time unit divider (the greater t is, the faster the game will go)\n",
);

/// Fully-parsed, validated server configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// TCP port to listen on (1..=65535).
    pub port: u16,
    /// World width in tiles (> 0).
    pub width: u32,
    /// World height in tiles (> 0).
    pub height: u32,
    /// Team names in the order given; at least one, all distinct and non-empty.
    pub teams: Vec<String>,
    /// Authorized clients per team at game start (> 0).
    pub clients_per_team: u32,
    /// Time-unit divider `t` (> 0); defaults to [`DEFAULT_T`].
    pub t: u32,
}

/// Reasons CLI parsing can fail. `Display` is a short, human-readable line
/// printed to stderr above the usage text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliError {
    /// No arguments at all (bare `./server`): show usage without an error line.
    NoArgs,
    /// A flag that expects a value was the last token.
    MissingValue(String),
    /// A flag's value could not be parsed or was out of range.
    InvalidValue { flag: String, value: String },
    /// `-n` was given with no team names following it.
    NoTeams,
    /// The same team name was listed more than once.
    DuplicateTeam(String),
    /// A required flag was never supplied.
    MissingFlag(&'static str),
    /// The same single-value flag appeared twice.
    DuplicateFlag(String),
    /// An unrecognized flag or stray positional argument.
    Unknown(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::NoArgs => write!(f, "error: no arguments given"),
            CliError::MissingValue(flag) => write!(f, "error: missing value for {flag}"),
            CliError::InvalidValue { flag, value } => {
                write!(f, "error: invalid value '{value}' for {flag}")
            }
            CliError::NoTeams => write!(f, "error: -n requires at least one team name"),
            CliError::DuplicateTeam(name) => write!(f, "error: duplicate team name '{name}'"),
            CliError::MissingFlag(flag) => write!(f, "error: missing required flag {flag}"),
            CliError::DuplicateFlag(flag) => write!(f, "error: flag {flag} given more than once"),
            CliError::Unknown(tok) => write!(f, "error: unknown argument '{tok}'"),
        }
    }
}

impl std::error::Error for CliError {}

/// Pull the value token following `flag` at position `i`, advancing `i` past it.
/// A following token that itself looks like a flag counts as missing.
fn take_value<'a, S: AsRef<str>>(
    args: &'a [S],
    i: &mut usize,
    flag: &str,
) -> Result<&'a str, CliError> {
    let next = args.get(*i + 1).map(AsRef::as_ref);
    match next {
        Some(v) if !is_flag(v) => {
            *i += 1;
            Ok(v)
        }
        _ => Err(CliError::MissingValue(flag.to_string())),
    }
}

/// A token is a flag if it starts with `-` and is not a lone `-`.
fn is_flag(tok: &str) -> bool {
    tok.starts_with('-') && tok != "-"
}

fn parse_u32(flag: &str, value: &str) -> Result<u32, CliError> {
    match value.parse::<u32>() {
        Ok(n) if n > 0 => Ok(n),
        _ => Err(CliError::InvalidValue {
            flag: flag.to_string(),
            value: value.to_string(),
        }),
    }
}

fn set_once(slot: &mut Option<u32>, flag: &str, value: u32) -> Result<(), CliError> {
    if slot.is_some() {
        return Err(CliError::DuplicateFlag(flag.to_string()));
    }
    *slot = Some(value);
    Ok(())
}

/// Parse validated [`Config`] from CLI tokens (already stripped of argv[0]).
///
/// # Errors
/// Returns a [`CliError`] describing the first problem encountered: an unknown
/// flag, a missing/invalid value, a missing required flag, or a bad team list.
pub fn parse<S: AsRef<str>>(args: &[S]) -> Result<Config, CliError> {
    if args.is_empty() {
        return Err(CliError::NoArgs);
    }

    let mut port: Option<u32> = None;
    let mut width: Option<u32> = None;
    let mut height: Option<u32> = None;
    let mut clients: Option<u32> = None;
    let mut t: Option<u32> = None;
    let mut teams: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let tok = args[i].as_ref();
        match tok {
            "-p" => {
                let raw = take_value(args, &mut i, "-p")?;
                let n = raw.parse::<u16>().ok().filter(|&n| n != 0).ok_or_else(|| {
                    CliError::InvalidValue {
                        flag: "-p".to_string(),
                        value: raw.to_string(),
                    }
                })?;
                if port.is_some() {
                    return Err(CliError::DuplicateFlag("-p".to_string()));
                }
                port = Some(u32::from(n));
            }
            "-x" => set_once(
                &mut width,
                "-x",
                parse_u32("-x", take_value(args, &mut i, "-x")?)?,
            )?,
            "-y" => set_once(
                &mut height,
                "-y",
                parse_u32("-y", take_value(args, &mut i, "-y")?)?,
            )?,
            "-c" => set_once(
                &mut clients,
                "-c",
                parse_u32("-c", take_value(args, &mut i, "-c")?)?,
            )?,
            "-t" => set_once(
                &mut t,
                "-t",
                parse_u32("-t", take_value(args, &mut i, "-t")?)?,
            )?,
            "-n" => {
                if !teams.is_empty() {
                    return Err(CliError::DuplicateFlag("-n".to_string()));
                }
                // Consume every following non-flag token as a team name.
                while let Some(name) = args.get(i + 1).map(AsRef::as_ref) {
                    if is_flag(name) {
                        break;
                    }
                    if name.is_empty() {
                        return Err(CliError::InvalidValue {
                            flag: "-n".to_string(),
                            value: String::new(),
                        });
                    }
                    if teams.iter().any(|t| t == name) {
                        return Err(CliError::DuplicateTeam(name.to_string()));
                    }
                    teams.push(name.to_string());
                    i += 1;
                }
                if teams.is_empty() {
                    return Err(CliError::NoTeams);
                }
            }
            other => return Err(CliError::Unknown(other.to_string())),
        }
        i += 1;
    }

    Ok(Config {
        port: port.ok_or(CliError::MissingFlag("-p"))? as u16,
        width: width.ok_or(CliError::MissingFlag("-x"))?,
        height: height.ok_or(CliError::MissingFlag("-y"))?,
        teams: {
            if teams.is_empty() {
                return Err(CliError::MissingFlag("-n"));
            }
            teams
        },
        clients_per_team: clients.ok_or(CliError::MissingFlag("-c"))?,
        t: t.unwrap_or(DEFAULT_T),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(tokens: &[&str]) -> Vec<String> {
        tokens.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn parses_full_valid_line() {
        let args = v(&[
            "-p", "8080", "-x", "10", "-y", "10", "-c", "5", "-n", "team1", "team2", "-t", "100",
        ]);
        let cfg = parse(&args).expect("should parse");
        assert_eq!(cfg.port, 8080);
        assert_eq!(cfg.width, 10);
        assert_eq!(cfg.height, 10);
        assert_eq!(cfg.clients_per_team, 5);
        assert_eq!(cfg.teams, vec!["team1".to_string(), "team2".to_string()]);
        assert_eq!(cfg.t, 100);
    }

    #[test]
    fn t_defaults_to_100_when_omitted() {
        let args = v(&["-p", "4242", "-x", "5", "-y", "6", "-c", "3", "-n", "solo"]);
        let cfg = parse(&args).expect("should parse");
        assert_eq!(cfg.t, DEFAULT_T);
        assert_eq!(cfg.t, 100);
    }

    #[test]
    fn flags_may_appear_in_any_order() {
        let args = v(&[
            "-n", "a", "b", "-t", "10", "-c", "2", "-y", "8", "-x", "7", "-p", "9000",
        ]);
        let cfg = parse(&args).expect("should parse");
        assert_eq!(cfg.port, 9000);
        assert_eq!(cfg.width, 7);
        assert_eq!(cfg.height, 8);
        assert_eq!(cfg.t, 10);
        assert_eq!(cfg.teams, vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn single_team_is_valid() {
        let args = v(&["-p", "1", "-x", "1", "-y", "1", "-c", "1", "-n", "only"]);
        assert_eq!(parse(&args).unwrap().teams, vec!["only".to_string()]);
    }

    #[test]
    fn empty_args_is_no_args() {
        let empty: Vec<String> = Vec::new();
        assert_eq!(parse(&empty), Err(CliError::NoArgs));
    }

    #[test]
    fn missing_required_flag_errors() {
        // No -p.
        let args = v(&["-x", "10", "-y", "10", "-c", "5", "-n", "team"]);
        assert_eq!(parse(&args), Err(CliError::MissingFlag("-p")));
    }

    #[test]
    fn missing_teams_errors() {
        let args = v(&["-p", "8080", "-x", "10", "-y", "10", "-c", "5"]);
        assert_eq!(parse(&args), Err(CliError::MissingFlag("-n")));
    }

    #[test]
    fn dash_n_with_no_names_errors() {
        let args = v(&["-p", "8080", "-x", "10", "-y", "10", "-c", "5", "-n"]);
        assert_eq!(parse(&args), Err(CliError::NoTeams));
    }

    #[test]
    fn dash_n_immediately_before_flag_errors() {
        let args = v(&["-n", "-p", "8080", "-x", "10", "-y", "10", "-c", "5"]);
        assert_eq!(parse(&args), Err(CliError::NoTeams));
    }

    #[test]
    fn missing_value_errors() {
        let args = v(&["-p"]);
        assert_eq!(parse(&args), Err(CliError::MissingValue("-p".to_string())));
    }

    #[test]
    fn value_that_is_a_flag_counts_as_missing() {
        let args = v(&["-p", "-x", "10", "-y", "10", "-c", "5", "-n", "team"]);
        assert_eq!(parse(&args), Err(CliError::MissingValue("-p".to_string())));
    }

    #[test]
    fn non_numeric_value_errors() {
        let args = v(&["-p", "abc", "-x", "10", "-y", "10", "-c", "5", "-n", "team"]);
        assert_eq!(
            parse(&args),
            Err(CliError::InvalidValue {
                flag: "-p".to_string(),
                value: "abc".to_string()
            })
        );
    }

    #[test]
    fn zero_and_out_of_range_values_error() {
        let zero_width = v(&["-p", "8080", "-x", "0", "-y", "10", "-c", "5", "-n", "team"]);
        assert_eq!(
            parse(&zero_width),
            Err(CliError::InvalidValue {
                flag: "-x".to_string(),
                value: "0".to_string()
            })
        );

        // Port 0 is reserved; port above u16 range is invalid.
        let big_port = v(&[
            "-p", "70000", "-x", "10", "-y", "10", "-c", "5", "-n", "team",
        ]);
        assert_eq!(
            parse(&big_port),
            Err(CliError::InvalidValue {
                flag: "-p".to_string(),
                value: "70000".to_string()
            })
        );
    }

    #[test]
    fn unknown_flag_errors() {
        let args = v(&[
            "-z", "1", "-p", "8080", "-x", "10", "-y", "10", "-c", "5", "-n", "team",
        ]);
        assert_eq!(parse(&args), Err(CliError::Unknown("-z".to_string())));
    }

    #[test]
    fn duplicate_single_value_flag_errors() {
        let args = v(&[
            "-p", "8080", "-p", "9090", "-x", "10", "-y", "10", "-c", "5", "-n", "team",
        ]);
        assert_eq!(parse(&args), Err(CliError::DuplicateFlag("-p".to_string())));
    }

    #[test]
    fn duplicate_team_name_errors() {
        let args = v(&[
            "-p", "8080", "-x", "10", "-y", "10", "-c", "5", "-n", "team", "team",
        ]);
        assert_eq!(
            parse(&args),
            Err(CliError::DuplicateTeam("team".to_string()))
        );
    }

    #[test]
    fn usage_text_matches_audit_lines() {
        // Guard against drift from docs/raw/audit.md (AQ01).
        for line in [
            " Usage: ./server -p <port> -x <width> -y <height> -n <team> [<team>] [<team>] ... -c <nb> [-t <t>]",
            " -p port number",
            " -x world width",
            " -y world height",
            " -n team_name_1 team_name_2 ...",
            " -c number of clients authorized at the beginning of the game",
            " -t [100] time unit divider (the greater t is, the faster the game will go)",
        ] {
            assert!(USAGE.contains(line), "usage missing line: {line}");
        }
    }
}
