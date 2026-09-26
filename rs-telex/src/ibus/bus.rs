use std::error::Error;
use zbus::Connection;
use zbus::connection::Builder;

use super::address::get_ibus_address;
use super::factory::IBusFactory;
use super::types::{
    COMPONENT_NAME, IBUS_FACTORY_PATH, IBUS_INTERFACE, IBUS_PATH, IBUS_SERVICE, make_component,
};

pub struct IBusClient {
    pub connection: Connection,
}

impl IBusClient {
    pub async fn connect() -> Result<Self, Box<dyn Error + Send + Sync>> {
        let address = get_ibus_address()
            .ok_or_else(|| "Could not locate IBus session socket address".to_string())?;

        println!("Connecting to IBus daemon at {}", address);
        let connection = Builder::address(address.as_str())?.build().await?;

        Ok(Self { connection })
    }

    /// Register the component description directly with the running IBus daemon
    pub async fn register_component(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let comp_val = make_component();

        let reply = self
            .connection
            .call_method(
                Some(IBUS_SERVICE),
                IBUS_PATH,
                Some(IBUS_INTERFACE),
                "RegisterComponent",
                &(comp_val,),
            )
            .await?;
        let () = reply.body().deserialize()?;

        println!(
            "Successfully registered component '{}' with IBus daemon",
            COMPONENT_NAME
        );
        Ok(())
    }

    /// Run the IBus engine daemon service
    pub async fn run_service(self, _register: bool) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.connection.request_name(COMPONENT_NAME).await?;
        super::log::log_info(&format!(
            "IBusClient connected, requested bus name '{}'",
            COMPONENT_NAME
        ));

        self.connection
            .object_server()
            .at(IBUS_FACTORY_PATH, IBusFactory)
            .await?;
        super::log::log_info(&format!("Exported IBusFactory at {}", IBUS_FACTORY_PATH));

        if let Err(e) = self.register_component().await {
            super::log::log_info(&format!("Notice: Component registration reported: {}", e));
        }

        super::log::log_info("rs-telex IBus engine is active and ready.");
        std::future::pending::<()>().await;
        Ok(())
    }
}
