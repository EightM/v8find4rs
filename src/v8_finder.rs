use itertools::Itertools;

use crate::v8_app::V8Arch;
use crate::v8_finder::v8_platform::V8Platform;
use log::error;

mod v8_dir;
mod v8_platform;

/// Определяет приоритетность поиска платформы 1С. Работает аналогично варианту описанному в документации
/// к платформе
pub enum SearchPriority {
    /// Ищется максимально доступная версия 32-х разрядной платформы.
    X32,
    /// Ищется максимально доступная версия 64-х разрядной платформы.
    X64,
    /// Ищется 32-х разрядная платформа, но если существует 64-х разрядная платформа более высокой версии,
    /// то будет возвращена именно она.
    X32_64,
    /// Ищется 64-х разрядная платформа, но если существует 32-х разрядная платформа более высокой
    /// версии, то будет возвращена именно она.
    X64_32,
}


/// Структура для поиска установленных версий платформы 1С.
/// Пример использования:
///```no_run
/// use v8find4rs::v8_finder::{V8Finder, SearchPriority};
///
/// let finder: V8Finder = V8Finder::new();
/// let platform = finder.get_platform("8.3.14", SearchPriority::X32);
/// ```
pub struct V8Finder {
    platforms: Vec<V8Platform>,
}

impl V8Finder {
    /// Создание нового экземпляра. Поиск всех доступных платформ осуществляется сразу при создании объекта.
    /// В случае возникновения каких-то ошибок при создании объекта, будет осуществлена запись в канал `error`,
    /// и возвращен объект с пустым вектором доступных платформ
    pub fn new() -> Self {
        let platforms = V8Platform::v8_platforms();
        match platforms {
            Ok(platforms) => {
                V8Finder {
                    platforms,
                }
            }
            Err(err) => {
                error!("{}", err);
                V8Finder {platforms: Vec::new()}
            }
        }
    }

    /// Осуществляет фильтрацию хранящихся в V8Finder найденных платформ в поисках одной максимально новой
    /// версии
    /// * version - может быть в трех вариантах: "8.3", "8.3.3" и "8.3.3.1234"
    /// * search_priority - см. [SearchPriority]
    pub fn get_platform(&self, version: &str, search_priority: SearchPriority) -> Option<&V8Platform> {
        let full_version: Vec<_> = version.split(".").collect_vec();
        return match full_version.len() {
            // 8.3
            2 => {
                let generation: u32 = full_version[1].parse().unwrap();
                let filtered_platforms: Vec<_> = self.get_platforms_by_filter(
                    generation, None, None);

                V8Finder::max_platform_by_search_priority(filtered_platforms, search_priority)
            }
            // 8.3.3
            3 => {
                let generation: u32 = full_version[1].parse().unwrap();
                let version: u32 = full_version[2].parse().unwrap();
                let filtered_platforms = self.get_platforms_by_filter(
                    generation, Some(version), None);

                V8Finder::max_platform_by_search_priority(filtered_platforms, search_priority)
            }
            //8.3.3.1234
            4 => {
                let generation: u32 = full_version[1].parse().unwrap();
                let version: u32 = full_version[2].parse().unwrap();
                let build: u32 = full_version[3].parse().unwrap();

                let filtered_platforms = self.get_platforms_by_filter(
                    generation, Some(version), Some(build));

                V8Finder::max_platform_by_search_priority(filtered_platforms, search_priority)
            }
            _ => None
        };
    }

    fn get_platforms_by_filter(&self, generation: u32, version: Option<u32>, build: Option<u32>) -> Vec<&V8Platform> {
        let mut filtered_platforms = self.platforms.iter().rev()
            .filter(|platform| platform.generation == generation).collect_vec();

        if version.is_some() {
            filtered_platforms = filtered_platforms.iter()
                .copied()
                .filter(|platform| platform.version == version.unwrap()).collect_vec();
        }

        if build.is_some() {
            filtered_platforms = filtered_platforms.iter()
                .copied()
                .filter(|platform| platform.build == build.unwrap())
                .collect_vec();
        }

        filtered_platforms
    }

    fn max_platform_by_search_priority(filtered_platforms: Vec<&V8Platform>,
                                       search_priority: SearchPriority) -> Option<&V8Platform> {
        let max_x32 = filtered_platforms.iter()
            .copied()
            .filter(|platform| platform.arch == V8Arch::X86)
            .max();

        let max_x64 = filtered_platforms.iter()
            .copied()
            .filter(|platform| platform.arch == V8Arch::X64)
            .max();

        match search_priority {
            SearchPriority::X32 => max_x32,
            SearchPriority::X64 => max_x64,
            SearchPriority::X32_64 => max_x64.max(max_x32),
            SearchPriority::X64_32 => max_x32.max(max_x64),
        }
    }
}