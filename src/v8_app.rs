use std::path::PathBuf;
use std::fs::File;
use std::io::Read;
use exe::{PE, Arch};
use std::env;

/// Перечень возможных приложений 1С. Необходим для поиска пути к данным приложениям при использовании
/// V8Finder
pub enum V8AppType {
    /// Толстый клиент
    ThickClient,
    /// Тонкий клиент
    ThinClient,
    /// Клиент удаленного администрирования
    RAC,
    /// Сервер удаленного администрирования
    RAS,
    /// Сервер отладки
    DBGS,
    /// Сервер приложений
    AppServer,
    /// Сервер хранилища
    RepositoryServer,
    /// Автономный сервер
    IBSRV,
    /// Утилита администрирования
    IBCMD,
}

impl V8AppType {
    /// Возвращает текстовое имя соответствующего вида приложения 1С
    pub fn value(&self) -> &str {
        match *self {
            V8AppType::ThickClient => "1cv8",
            V8AppType::ThinClient => "1cv8c",
            V8AppType::RAC => "rac",
            V8AppType::RAS => "ras",
            V8AppType::DBGS => "dbgs",
            V8AppType::AppServer => "ragent",
            V8AppType::RepositoryServer => "crserver",
            V8AppType::IBSRV => "ibsrv",
            V8AppType::IBCMD => "ibcmd",
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
/// Перечень разрядностей платформы 1С
pub enum V8Arch {
    /// 32-х разрядная
    X86,
    /// 64-х разрядная
    X64,
}

impl V8Arch {
    /// Осуществляет попытку определения разрядности платформы 1С. Логика определения различается в
    /// зависимости от текущей ОС:
    /// * Windows - по файлу 1cv8s.exe находящегося в папке bin. Читается его PE
    /// [сигнатура](https://docs.microsoft.com/en-us/windows/win32/debug/pe-format#machine-types)
    /// * Linux - по пути платформы. 1С автоматически устанавливает платформу в папки i386 или x86_64, соответственно
    /// разрядность определяется по наличию одной из подстрок в пути;
    /// * macOS - всегда 64-х разрядная.
    ///
    /// Если по каким-то причинам метод не сможет определить разрядность самостоятельно,
    /// по умолчанию присваивается разрядность 32
    pub fn from_path(v8_path: &PathBuf) -> V8Arch {
        let current_os = env::consts::OS;
        return match current_os {
            "windows" => V8Arch::v8_arch_from_exe(&v8_path.join(get_v8s_suffix())),
            "linux" => V8Arch::v8_arch_from_linux_path(v8_path),
            _ => V8Arch::X64
        }
    }

    fn v8_arch_from_linux_path(path: &PathBuf) -> V8Arch {
        let str_path = path.to_str().unwrap_or("");
        if str_path.contains("i386") {
            V8Arch::X86
        } else {
            V8Arch::X64
        }
    }

    fn v8_arch_from_exe(path_to_exe: &PathBuf) -> V8Arch {
        if path_to_exe.exists() {
            let exe_file = File::open(path_to_exe);
            if let Ok(mut exe_file) = exe_file {
                let mut buf = Vec::new();
                let _ = exe_file.read_to_end(&mut buf);

                let pe_file = PE::new_disk(buf.as_slice());
                let v8_arch = pe_file.get_arch();
                return match v8_arch {
                    Ok(Arch::X64) => V8Arch::X64,
                    Ok(Arch::X86) => V8Arch::X86,
                    Err(_) => V8Arch::X86,
                };
            }
        }

        V8Arch::X86
    }
}

fn get_v8s_suffix() -> &'static str {
    let current_os = env::consts::OS;
    match current_os {
        "windows" => r"bin\1cv8s.exe",
        _ => "1cv8s",
    }
}
