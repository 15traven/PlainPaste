use windows_registry::{
    CURRENT_USER,
    Result as WindowsRegistryResult
};
use windows_result::Error as WindowsError;

const APP_NAME: &str = "PlainPaste";
const AL_REGKEY: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run";
const TASK_MANAGER_OVERRIDE_REGKEY: &str =
    r"SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run";
const TASK_MANAGER_OVERRIDE_ENABLED_VALUE: [u8; 12] = [
    0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
const TASK_MANAGER_OVERRIDE_DISABLED_VALUE: [u8; 12] = [
    0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

pub fn register() -> Result<(), WindowsError> {
    let app_path = std::env::current_exe().unwrap();
    
    CURRENT_USER.create(AL_REGKEY)?.set_string(
        &APP_NAME,
        &app_path.to_str().unwrap()
    )
}

pub fn enable() -> WindowsRegistryResult<()> {
    if let Ok(key) = CURRENT_USER.create(TASK_MANAGER_OVERRIDE_REGKEY) {
        key.set_bytes(
            &APP_NAME, 
            windows_registry::Type::Bytes, 
            &TASK_MANAGER_OVERRIDE_ENABLED_VALUE
        )?;
    }

    Ok(())
}

pub fn disable() -> WindowsRegistryResult<()> {
    if let Ok(key) = CURRENT_USER.create(TASK_MANAGER_OVERRIDE_REGKEY) {
        key.set_bytes(
            &APP_NAME, 
            windows_registry::Type::Bytes, 
            &TASK_MANAGER_OVERRIDE_DISABLED_VALUE
        )?;
    }

    Ok(())    
}

pub fn is_enabled() -> WindowsRegistryResult<bool> {
    let value = CURRENT_USER
        .open(TASK_MANAGER_OVERRIDE_REGKEY)?
        .get_value(&APP_NAME)?;

    if value[0] == 2 {
        Ok(true)
    } else {
        Ok(false)
    }
}