use super::{HandlerError, HandlerErrorKind, MutationBackend};
use crate::mutation::plan::CapturedRepresentation;

pub struct WindowsSettingStore;

#[cfg(windows)]
mod implementation {
    use std::io;

    use winreg::{
        RegKey,
        enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WRITE},
    };

    use super::{CapturedRepresentation, HandlerError, HandlerErrorKind, MutationBackend};

    const ADVANCED_PATH: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced";
    const WIDGETS_VALUE: &str = "TaskbarDa";
    const TASK_VIEW_VALUE: &str = "ShowTaskViewButton";
    const SHOW_DESKTOP_VALUE: &str = "TaskbarSd";
    const WINDOWS_EXPLORER_POLICY_PATH: &str = r"Software\Policies\Microsoft\Windows\Explorer";
    const EXPLORER_POLICY_PATH: &str =
        r"Software\Microsoft\Windows\CurrentVersion\Policies\Explorer";
    const WIDGETS_POLICY_PATH: &str = r"Software\Policies\Microsoft\Dsh";
    const WIDGETS_POLICY_VALUE: &str = "AllowNewsAndInterests";
    const TASK_VIEW_POLICY_VALUE: &str = "HideTaskViewButton";
    const TASKBAR_SETTINGS_POLICY_VALUE: &str = "NoSetTaskbar";

    fn read_fixed(value_name: &'static str) -> Result<CapturedRepresentation, HandlerError> {
        let current_user = RegKey::predef(HKEY_CURRENT_USER);
        let key = current_user
            .open_subkey_with_flags(ADVANCED_PATH, KEY_READ)
            .map_err(|error| map_read_error(error, "open the fixed taskbar key"))?;
        match key.get_value::<u32, _>(value_name) {
            Ok(value) => Ok(CapturedRepresentation::Dword(value)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                Ok(CapturedRepresentation::Missing)
            }
            Err(error) => Err(map_read_error(error, "read the fixed taskbar DWORD")),
        }
    }

    fn write_fixed(
        value_name: &'static str,
        state: &CapturedRepresentation,
    ) -> Result<(), HandlerError> {
        let current_user = RegKey::predef(HKEY_CURRENT_USER);
        let key = current_user
            .open_subkey_with_flags(ADVANCED_PATH, KEY_READ | KEY_WRITE)
            .map_err(|error| map_error(error, "open the existing fixed taskbar key for writing"))?;
        match state {
            CapturedRepresentation::Dword(value @ 0..=1) => key
                .set_value(value_name, value)
                .map_err(|error| map_error(error, "write the fixed taskbar DWORD")),
            CapturedRepresentation::Missing => match key.delete_value(value_name) {
                Ok(()) => Ok(()),
                Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
                Err(error) => Err(map_error(error, "restore the fixed taskbar value absence")),
            },
            CapturedRepresentation::Dword(_) => Err(HandlerError::new(
                HandlerErrorKind::InvalidRepresentation,
                "The handler refused an out-of-range taskbar DWORD.",
            )),
        }
    }

    fn read_optional_dword(
        root: &RegKey,
        path: &'static str,
        value_name: &'static str,
    ) -> Result<Option<u32>, HandlerError> {
        let key = match root.open_subkey_with_flags(path, KEY_READ) {
            Ok(key) => key,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(map_read_error(error, "open a fixed policy key")),
        };
        match key.get_value::<u32, _>(value_name) {
            Ok(value @ 0..=1) => Ok(Some(value)),
            Ok(_) => Err(HandlerError::new(
                HandlerErrorKind::InvalidRepresentation,
                "A fixed taskbar policy contained an unsupported DWORD.",
            )),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(map_read_error(error, "read a fixed taskbar policy DWORD")),
        }
    }

    fn policy_is_configured(
        path: &'static str,
        value_name: &'static str,
    ) -> Result<bool, HandlerError> {
        let current_user = RegKey::predef(HKEY_CURRENT_USER);
        let local_machine = RegKey::predef(HKEY_LOCAL_MACHINE);
        Ok(
            read_optional_dword(&current_user, path, value_name)?.is_some()
                || read_optional_dword(&local_machine, path, value_name)?.is_some(),
        )
    }

    fn taskbar_settings_are_blocked() -> Result<bool, HandlerError> {
        let current_user = RegKey::predef(HKEY_CURRENT_USER);
        let local_machine = RegKey::predef(HKEY_LOCAL_MACHINE);
        Ok(read_optional_dword(
            &current_user,
            EXPLORER_POLICY_PATH,
            TASKBAR_SETTINGS_POLICY_VALUE,
        )? == Some(1)
            || read_optional_dword(
                &local_machine,
                EXPLORER_POLICY_PATH,
                TASKBAR_SETTINGS_POLICY_VALUE,
            )? == Some(1))
    }

    fn map_error(error: io::Error, action: &str) -> HandlerError {
        HandlerError::new(
            if error.kind() == io::ErrorKind::NotFound {
                HandlerErrorKind::MissingRepresentation
            } else {
                HandlerErrorKind::WriteFailed
            },
            format!(
                "Could not {action}; Windows error category: {:?}.",
                error.kind()
            ),
        )
    }

    fn map_read_error(error: io::Error, action: &str) -> HandlerError {
        HandlerError::new(
            if error.kind() == io::ErrorKind::NotFound {
                HandlerErrorKind::MissingRepresentation
            } else {
                HandlerErrorKind::ReadFailed
            },
            format!(
                "Could not {action}; Windows error category: {:?}.",
                error.kind()
            ),
        )
    }

    impl MutationBackend for super::WindowsSettingStore {
        fn read_widgets(&self) -> Result<CapturedRepresentation, HandlerError> {
            read_fixed(WIDGETS_VALUE)
        }

        fn widgets_externally_managed(&self) -> Result<bool, HandlerError> {
            let local_machine = RegKey::predef(HKEY_LOCAL_MACHINE);
            Ok(
                read_optional_dword(&local_machine, WIDGETS_POLICY_PATH, WIDGETS_POLICY_VALUE)?
                    .is_some()
                    || taskbar_settings_are_blocked()?,
            )
        }

        fn write_widgets(&self, state: &CapturedRepresentation) -> Result<(), HandlerError> {
            write_fixed(WIDGETS_VALUE, state)
        }

        fn read_task_view(&self) -> Result<CapturedRepresentation, HandlerError> {
            read_fixed(TASK_VIEW_VALUE)
        }

        fn task_view_externally_managed(&self) -> Result<bool, HandlerError> {
            Ok(
                policy_is_configured(WINDOWS_EXPLORER_POLICY_PATH, TASK_VIEW_POLICY_VALUE)?
                    || taskbar_settings_are_blocked()?,
            )
        }

        fn write_task_view(&self, state: &CapturedRepresentation) -> Result<(), HandlerError> {
            write_fixed(TASK_VIEW_VALUE, state)
        }

        fn read_show_desktop(&self) -> Result<CapturedRepresentation, HandlerError> {
            read_fixed(SHOW_DESKTOP_VALUE)
        }

        fn show_desktop_externally_managed(&self) -> Result<bool, HandlerError> {
            taskbar_settings_are_blocked()
        }

        fn write_show_desktop(&self, state: &CapturedRepresentation) -> Result<(), HandlerError> {
            write_fixed(SHOW_DESKTOP_VALUE, state)
        }
    }
}

#[cfg(not(windows))]
impl MutationBackend for WindowsSettingStore {
    fn read_widgets(&self) -> Result<CapturedRepresentation, HandlerError> {
        Err(unsupported())
    }
    fn widgets_externally_managed(&self) -> Result<bool, HandlerError> {
        Err(unsupported())
    }
    fn write_widgets(&self, _: &CapturedRepresentation) -> Result<(), HandlerError> {
        Err(unsupported())
    }
    fn read_task_view(&self) -> Result<CapturedRepresentation, HandlerError> {
        Err(unsupported())
    }
    fn task_view_externally_managed(&self) -> Result<bool, HandlerError> {
        Err(unsupported())
    }
    fn write_task_view(&self, _: &CapturedRepresentation) -> Result<(), HandlerError> {
        Err(unsupported())
    }
    fn read_show_desktop(&self) -> Result<CapturedRepresentation, HandlerError> {
        Err(unsupported())
    }
    fn show_desktop_externally_managed(&self) -> Result<bool, HandlerError> {
        Err(unsupported())
    }
    fn write_show_desktop(&self, _: &CapturedRepresentation) -> Result<(), HandlerError> {
        Err(unsupported())
    }
}

#[cfg(not(windows))]
fn unsupported() -> HandlerError {
    HandlerError::new(
        HandlerErrorKind::UnsupportedPlatform,
        "Mutation alpha handlers are available only on Windows.",
    )
}
