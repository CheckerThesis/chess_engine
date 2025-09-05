pub enum UciCommand {
    Uci,
    IsReady,
    SetOption(String),
    UciNewGame,
    Position(String),
    Go(String),
    Stop,
    Quit,
    Unknown,

    Testing(String),
    Generate
}