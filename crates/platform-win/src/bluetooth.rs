//! Memantau perangkat Bluetooth yang dipasangkan lewat `DeviceWatcher` (WinRT).
//!
//! Setiap perubahan diteruskan mentah ke thread UI sebagai [`Event::Bluetooth`]; keputusan
//! kapan alert tampil ada di `notchi_core::bluetooth`.

use notchi_core::bluetooth::Update;
use windows::Devices::Enumeration::{DeviceInformation, DeviceInformationKind, DeviceInformationUpdate, DeviceWatcher};
use windows_collections::{IIterable, IMapView};
use windows::Foundation::{IReference, TypedEventHandler};
use windows_core::{HSTRING, IInspectable, Interface, Result};

use crate::window::{Event, UiProxy};

const CLASSIC: &str = "{e0cbf06c-cd8b-4647-bb8a-263b43f0f974}";
const LE: &str = "{bb7bb05e-5972-42b5-94fc-76eaa7084d49}";
const CONNECTED: &str = "System.Devices.Aep.IsConnected";
/// Level baterai yang dilaporkan sebagian perangkat (headset, mouse) lewat Windows.
const BATTERY: &str = "{104EA319-6EE2-4701-BD47-8DDBF425BBE5} 2";

pub struct BluetoothWatcher {
    watcher: DeviceWatcher,
}

impl std::fmt::Debug for BluetoothWatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("BluetoothWatcher")
    }
}

fn props_of(map: &IMapView<HSTRING, IInspectable>) -> (Option<bool>, Option<u8>) {
    let get = |key: &str| map.Lookup(&HSTRING::from(key)).ok();
    let connected = get(CONNECTED).and_then(|v| v.cast::<IReference<bool>>().ok()).and_then(|r| r.Value().ok());
    let battery = get(BATTERY).and_then(|v| v.cast::<IReference<u8>>().ok()).and_then(|r| r.Value().ok());
    (connected, battery)
}

impl BluetoothWatcher {
    pub fn start(proxy: UiProxy) -> Result<BluetoothWatcher> {
        let aqs = HSTRING::from(format!(
            "(System.Devices.Aep.ProtocolId:=\"{CLASSIC}\" OR System.Devices.Aep.ProtocolId:=\"{LE}\") AND System.Devices.Aep.IsPaired:=System.StructuredQueryType.Boolean#True"
        ));
        let props: IIterable<HSTRING> = vec![HSTRING::from(CONNECTED), HSTRING::from(BATTERY)].into();
        let watcher = DeviceInformation::CreateWatcherWithKindAqsFilterAndAdditionalProperties(
            &aqs,
            &props,
            DeviceInformationKind::AssociationEndpoint,
        )?;

        let p = proxy.clone();
        watcher.Added(&TypedEventHandler::<DeviceWatcher, DeviceInformation>::new(move |_, info| {
            if let Some(info) = info.as_ref() {
                let (connected, battery) = info.Properties().map(|m| props_of(&m)).unwrap_or((None, None));
                let name = info.Name().ok().map(|n| n.to_string());
                p.post(Event::Bluetooth(Update { id: info.Id()?.to_string(), name, connected, battery }));
            }
            Ok(())
        }))?;
        let p = proxy.clone();
        watcher.Updated(&TypedEventHandler::<DeviceWatcher, DeviceInformationUpdate>::new(move |_, u| {
            if let Some(u) = u.as_ref() {
                let (connected, battery) = u.Properties().map(|m| props_of(&m)).unwrap_or((None, None));
                if connected.is_some() || battery.is_some() {
                    p.post(Event::Bluetooth(Update { id: u.Id()?.to_string(), name: None, connected, battery }));
                }
            }
            Ok(())
        }))?;
        let p = proxy.clone();
        watcher.Removed(&TypedEventHandler::<DeviceWatcher, DeviceInformationUpdate>::new(move |_, u| {
            if let Some(u) = u.as_ref() {
                p.post(Event::BluetoothRemoved(u.Id()?.to_string()));
            }
            Ok(())
        }))?;
        let p = proxy;
        watcher.EnumerationCompleted(&TypedEventHandler::<DeviceWatcher, IInspectable>::new(move |_, _| {
            p.post(Event::BluetoothReady);
            Ok(())
        }))?;
        watcher.Start()?;
        Ok(BluetoothWatcher { watcher })
    }
}

impl Drop for BluetoothWatcher {
    fn drop(&mut self) {
        let _ = self.watcher.Stop();
    }
}
