//! Status daya dari `GetSystemPowerStatus` (PRD §3.2).

use notchi_core::power::PowerReading;
use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};

pub fn read() -> PowerReading {
    let mut s = SYSTEM_POWER_STATUS::default();
    // SAFETY: struktur keluaran valid selama pemanggilan.
    let ok = unsafe { GetSystemPowerStatus(&mut s) }.is_ok();
    if !ok {
        return PowerReading { has_battery: false, ac_online: true, percent: None };
    }
    // BatteryFlag 128 = tanpa baterai sistem, 255 = tidak diketahui.
    let has_battery = s.BatteryFlag != 128 && s.BatteryFlag != 255;
    PowerReading {
        has_battery,
        ac_online: s.ACLineStatus == 1,
        percent: (s.BatteryLifePercent <= 100).then_some(s.BatteryLifePercent),
    }
}

/// Battery Saver menyala.
pub fn battery_saver() -> bool {
    let mut s = SYSTEM_POWER_STATUS::default();
    // SAFETY: struktur keluaran valid selama pemanggilan.
    unsafe { GetSystemPowerStatus(&mut s) }.is_ok() && s.SystemStatusFlag == 1
}
