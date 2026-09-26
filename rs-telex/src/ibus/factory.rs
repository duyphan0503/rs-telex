use std::sync::atomic::{AtomicUsize, Ordering};
use zbus::interface;
use zbus::object_server::ObjectServer;
use zvariant::OwnedObjectPath;

use super::engine::IBusEngineService;

static ENGINE_COUNTER: AtomicUsize = AtomicUsize::new(1);

pub struct IBusFactory;

#[interface(name = "org.freedesktop.IBus.Factory")]
impl IBusFactory {
    /// Called by IBus daemon when an engine instance is created.
    async fn create_engine(
        &self,
        #[zbus(object_server)] server: &ObjectServer,
        _name: &str,
    ) -> zbus::fdo::Result<OwnedObjectPath> {
        let id = ENGINE_COUNTER.fetch_add(1, Ordering::SeqCst);
        let path_str = format!("/org/freedesktop/IBus/Engine/{}", id);
        let path = OwnedObjectPath::try_from(path_str)
            .map_err(|e: zvariant::Error| zbus::fdo::Error::Failed(e.to_string()))?;

        let engine = IBusEngineService::new();
        server.at(&path, engine).await?;

        super::log::log_info(&format!(
            "IBusFactory::create_engine called for engine='{}', assigned path='{}'",
            _name,
            path.as_str()
        ));
        Ok(path)
    }

    /// Returns metadata information array
    async fn get_info(&self) -> Vec<String> {
        vec![
            "rs-telex".to_string(),
            "vi".to_string(),
            "ibus-rs-telex".to_string(),
            "duyphan0503".to_string(),
            "MIT".to_string(),
        ]
    }
}
