//////////////////
///// TO DO //////
//////////////////
// - vérifier [async]
// - ajouter crate nécessaire
// - coder les différentes fonctions
//////////////////


//The driver for the ethernet is already implemented in the OS, then we just use TcpStream to ask and tell for protocol.rs
use tokio::net::TcpStream;

// structure to manage the ethernet interface
pub struct EthernetInterface{
    logger: Logger,
    stream: TcpStream,
}

impl EthernetInterface {

    //using arc mutex to share the ethernet interface between threads
    pub fn into_arc_mutex(self) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(self))
    }

    // create a new instance of ethernet driver
    pub async fn connect(ip:&str, port: &u16)-> Result<Self,Error> {

    }   
    
}

#[async_trait]
impl BytesDialogProtocol for EthernetInterface {
    // tell fonction to send data without receiving
    async fn tell(&mut self, command: Bytes) -> Result<(), Error> {

    }
    // ask fonction to send data and expecting data back
    async fn ask(&mut self, command: Bytes) -> Result<Bytes, Error> {

    }
}
