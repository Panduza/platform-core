use crate::{BooleanAttServer, Class, Error, JsonAttServer};

#[derive(Clone)]
pub enum Element {
    Class(Class),
    AsBoolean(BooleanAttServer),
    AsJson(JsonAttServer),
}

impl Element {
    /// Request attribute server enablement
    ///
    pub async fn change_enablement(&mut self, enabled: bool) -> Result<(), Error> {
        match self {
            Element::Class(_class) => Ok(()),
            Element::AsBoolean(att_server) => att_server.change_enablement(enabled).await,
            Element::AsJson(att_server) => att_server.change_enablement(enabled).await,
        }
    }

    /// Request attribute server enablement
    ///
    pub async fn enable(&mut self) -> Result<(), Error> {
        self.change_enablement(true).await
    }

    /// Request attribute server disablement
    ///
    pub async fn disable(&mut self) -> Result<(), Error> {
        self.change_enablement(false).await
    }
}
