use ui_app::{FromNet, ToNet};

// XXX(RLB) This reader will only work as long as the data in the value of the TLV doesn't have any
// characters that need to be escaped according to SLIP.  Also, it only supports TLVs with up to
// 128 bytes of payload.
pub struct NetRx<'a, Reader> {
    rx: &'a mut Reader,
}

impl<'a, Reader> NetRx<'a, Reader>
where
    Reader: embedded_io_async::Read + 'a,
{
    pub fn new(rx: &'a mut Reader) -> Self {
        Self { rx }
    }

    pub async fn next(&mut self) -> Option<FromNet> {
        let mut type_ = [0u8; 1];
        let mut length = [0u8; 4];
        let mut value = [0u8; 128];

        self.rx.read(&mut type_).await.unwrap();
        self.rx.read(&mut length).await.unwrap();

        let length = u32::from_be_bytes(length) as usize;
        let delimited_value = &mut value[..(length + 1)];
        self.rx.read(delimited_value).await.unwrap();

        match type_[0] {
            0x0f => {
                defmt::assert_eq!(length, 0);
                Some(FromNet::Pong)
            }
            _ => {
                defmt::debug!("skipping TLV of type {:x}", type_[0]);
                None
            }
        }
    }
}

// XXX(RLB) This is just a fixed writer right now, with no generic logic for SLIP or the TLV
// format.  It will need to be generalized.
pub struct NetTx<Writer> {
    tx: Writer,
}

impl<Writer> NetTx<Writer> {
    pub fn new(tx: Writer) -> Self {
        Self { tx }
    }
}

impl<Writer> ui_app::NetTx for NetTx<Writer>
where
    Writer: embedded_io::Write,
{
    fn write(&mut self, to_net: &ToNet) {
        match to_net {
            ToNet::Ping => {
                const PING: u8 = 0x0e;
                const PACKET: &[u8] = &[PING, 0x00, 0x00, 0x00, 0x00, 0xC0];
                self.tx.write(PACKET).unwrap();
            }
        }
    }
}
