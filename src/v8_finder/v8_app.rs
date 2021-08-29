enum V8AppType {
    Application,
    ThinClient,
    RAC,
    DBGS,
    AppServer,
    RepositoryServer,
    IBSRV,
    IBCMD,
}

impl V8AppType {
    fn value(&self) -> &str {
        match *self {
            V8AppType::Application => "1cv8",
            V8AppType::ThinClient => "1cv8c",
            V8AppType::RAC => "rac",
            V8AppType::DBGS => "dbgs",
            V8AppType::AppServer => "ragent",
            V8AppType::RepositoryServer => "crserver",
            V8AppType::IBSRV => "ibsrv",
            V8AppType::IBCMD => "ibcmd",
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum V8Arch {
    X86,
    X64,
}

impl V8Arch {
    fn value(&self) -> &str {
        match *self {
            V8Arch::X86 => "x86",
            V8Arch::X64 => "x64",
        }
    }
}
