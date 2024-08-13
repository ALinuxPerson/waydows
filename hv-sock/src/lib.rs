#[cfg(windows)]
pub mod host_registry {
    use std::mem;
    use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
    use getset::Getters;
    use uuid::Uuid;
    use windows_registry::{Key, KeyIterator};

    pub const HIVE: &Key = windows_registry::LOCAL_MACHINE;
    pub const KEY: &'static str = "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Virtualization\\GuestCommunicationServices";
    pub const ELEMENT_NAME: &'static str = "ElementName";

    pub struct ServiceData {
        pub uuid: Uuid,
        pub element_name: String,
    }

    #[derive(Getters)]
    #[getset(get = "pub")]
    pub struct Service {
        data: ServiceData,
        key: Key,
    }
    
    impl Service {
        pub fn set_element_name(&mut self, to: String) -> windows_registry::Result<String> {
            self.key.set_string(ELEMENT_NAME, &to)?;
            Ok(mem::replace(&mut self.data.element_name, to))
        }
    }

    pub struct HostRegistry {
        key: Key,
        lock: Option<RwLock<()>>,
    }

    impl HostRegistry {
        fn from_key(key: Key) -> Self {
            Self { key, lock: Some(RwLock::new(())) }
        }
        
        pub fn open() -> windows_registry::Result<Self> {
            Ok(Self::from_key(HIVE.open(KEY)?))
        }

        pub fn create() -> windows_registry::Result<Self> {
            Ok(Self::from_key(HIVE.create(KEY)?))
        }
    }

    impl HostRegistry {
        fn from_key_no_lock(key: Key) -> Self {
            Self { key, lock: None }
        }

        pub fn open_no_lock() -> windows_registry::Result<Self> {
            Ok(Self::from_key_no_lock(HIVE.open(KEY)?))
        }

        pub fn create_no_lock() -> windows_registry::Result<Self> {
            Ok(Self::from_key_no_lock(HIVE.create(KEY)?))
        }
    }

    impl HostRegistry {
        pub fn key(&self) -> &Key {
            &self.key
        }
    }
    
    impl HostRegistry {
        pub fn lock(&mut self, lock: bool) {
            if lock {
                if self.lock.is_none() {
                    self.lock = Some(RwLock::new(()))
                }
            } else {
                self.lock = None
            }
        }
        
        fn read(&self) -> Option<RwLockReadGuard<()>> {
            self.lock.as_ref().map(|l| l.read().unwrap())
        }
        
        fn read_with<R>(&self, f: impl FnOnce(&Key) -> R) -> R {
            let _guard = self.read();
            f()
        }

        fn write(&self) -> Option<RwLockWriteGuard<()>> {
            self.lock.as_ref().map(|l| l.write().unwrap())
        }
        
        fn write_with<R>(&self, f: impl FnOnce(&Key) -> R) -> R {
            let _guard = self.write();
            f()
        }
    }

    impl HostRegistry {
        pub fn register(&self, service: ServiceData) -> windows_registry::Result<Service> {
            let key = self.key.create(&service.uuid.to_string())?;
            self.write_with(|key| key.set_string(ELEMENT_NAME, &service.element_name))?;
            Ok(Service { data: service, key })
        }

        pub fn delete(&self, uuid: Uuid) -> windows_registry::Result<()> {
            self.write_with(|key| key.remove_tree(&uuid.to_string()))?;
            Ok(())
        }

        pub fn get(&self, uuid: Uuid) -> windows_registry::Result<Service> {
            let key = self.read_with(|key| self.key.open(&uuid.to_string()))?;
            let element_name = key.get_string(ELEMENT_NAME)?;

            Ok(Service { data: ServiceData { uuid, element_name }, key })
        }

        pub fn rename(&self, from: Uuid, to: Uuid) -> windows_registry::Result<Service> {
            let element_name = self.get(from)?.data.element_name;
            self.delete(from)?;
            self.register(ServiceData { uuid: to, element_name })
        }
    }
    
    impl HostRegistry {
        pub fn iter(&self) -> windows_registry::Result<Iter> {
            Ok(Iter { host_registry: self, keys: self.key.keys()? })
        }
    }

    pub struct Iter<'hr> {
        host_registry: &'hr HostRegistry,
        keys: KeyIterator<'hr>,
    }

    impl<'hr> Iterator for Iter<'hr> {
        type Item = windows_registry::Result<Service>;

        fn next(&mut self) -> Option<Self::Item> {
            self.keys.next().map(|k| self.host_registry.get(k.parse().unwrap()))
        }
    }
}

#[cfg(windows)]
mod listener {
    pub struct HyperVSocketListener;
}

mod stream {
    pub struct HyperVSocketStream;
}
