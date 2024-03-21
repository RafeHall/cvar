use cvars::{CVarEnum, Value};

#[derive(CVarEnum, Debug, PartialEq)]
enum GameModes {
    #[cvar(alias = "ctf")]
    CaptureTheFlag,
    #[cvar(alias = "tdm")]
    TeamDeathMatch,
    #[cvar(alias = "ffa")]
    FreeForAll,
    #[cvar(alias = "cp")]
    ControlPoint,
}

#[test]
fn test() {
    let r = GameModes::validate("c").unwrap();
    println!("{:#?}", r);
}
