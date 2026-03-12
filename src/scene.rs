#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scene {
    MainMenu,
    Settings,
    MatchIntro,
    RoundPlaying,
    RoundResolve,
    MatchOver,
}
