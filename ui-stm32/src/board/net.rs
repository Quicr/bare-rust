use defmt::Format;
use heapless::String;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use ui_app::{Frame, FromNet, ToNet, FRAME_SIZE, MAX_MESSAGE_LEN};

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
        FromNet::read_tlv(&mut self.rx).await
    }
}

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
        to_net.write_tlv(&mut self.tx);
    }
}

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug, IntoPrimitive, TryFromPrimitive, Format)]
enum PacketTypeToNet {
    Message = 0x09,
    Ping = 0x0e,
}

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug, IntoPrimitive, TryFromPrimitive, Format)]
enum PacketTypeFromNet {
    Message = 0x09,
    Pong = 0x0f,
}

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug, IntoPrimitive, TryFromPrimitive, Format)]
enum ChannelId {
    Ptt = 0,
    PttAi = 1,
    Chat = 2,
    ChatAi = 3,
    Count = 4,
}

trait TlvRead: Sized {
    async fn read_tlv(r: &mut impl embedded_io_async::Read) -> Option<Self>;
}

impl TlvRead for FromNet {
    async fn read_tlv(r: &mut impl embedded_io_async::Read) -> Option<Self> {
        let mut type_length = [0u8; 5];
        r.read_exact(&mut type_length).await.unwrap();

        let Ok(packet_type) = PacketTypeFromNet::try_from(type_length[0]) else {
            defmt::info!("skipping unsupported TLV type {:x}", type_length[0]);
            return None;
        };

        let len = u32::from_be_bytes(type_length[1..].try_into().unwrap()) as usize;

        Some(match packet_type {
            PacketTypeFromNet::Pong => Self::Pong,
            PacketTypeFromNet::Message => {
                let mut channel_id = [0];
                r.read(&mut channel_id).await.unwrap();
                let channel_id = ChannelId::try_from(channel_id[0]).unwrap();

                match channel_id {
                    ChannelId::Ptt => {
                        let mut frame_data = [0u8; 2 * FRAME_SIZE];
                        assert_eq!(len, frame_data.len() + 1);
                        r.read_exact(&mut frame_data).await.unwrap();

                        let mut frame = Frame::default();
                        for (x8, x16) in frame_data.chunks(2).zip(frame.0.iter_mut()) {
                            *x16 = u16::from_be_bytes(x8.try_into().unwrap());
                        }

                        Self::AudioFrame(frame)
                    }
                    ChannelId::Chat => {
                        let mut msg = [0; MAX_MESSAGE_LEN];
                        assert!(len < msg.len());

                        r.read_exact(&mut msg[..len]).await.unwrap();
                        Self::Chat(String::from_iter(msg.into_iter().map(|b| char::from(b))))
                    }

                    _ => unreachable!("Unsupported Channel ID"),
                }
            }
        })
    }
}

trait TlvWrite {
    fn write_tlv(&self, w: &mut impl embedded_io::Write);
}

impl TlvWrite for ToNet {
    fn write_tlv(&self, w: &mut impl embedded_io::Write) {
        match self {
            Self::Ping => {
                defmt::info!("Writing packet of type: {:?}", PacketTypeToNet::Ping);
                w.write(&[PacketTypeToNet::Ping.into(), 0, 0, 0, 0])
                    .unwrap();
            }
            Self::AudioFrame(frame) => {
                defmt::info!(
                    "Writing packet of type: {:?}::{:?}",
                    PacketTypeToNet::Message,
                    ChannelId::Ptt
                );
                let len = (frame.0.len() + 1) as u32;
                let mut header = [0; 6];
                header[0] = PacketTypeToNet::Message.into();
                header[1..5].copy_from_slice(&len.to_be_bytes());
                header[5] = ChannelId::Ptt.into();

                let mut frame_data = [0; 2 * FRAME_SIZE];
                for (x8, x16) in frame_data.chunks_mut(2).zip(frame.0.iter()) {
                    x8.copy_from_slice(&x16.to_be_bytes());
                }

                w.write(&header).unwrap();
                w.write(&frame_data).unwrap();
            }
            Self::Chat(msg) => {
                defmt::info!(
                    "Writing packet of type: {:?}::{:?}",
                    PacketTypeToNet::Message,
                    ChannelId::Chat
                );
                let len = (msg.len() + 1) as u32;
                let mut header = [0; 6];
                header[0] = PacketTypeToNet::Message.into();
                header[1..5].copy_from_slice(&len.to_be_bytes());
                header[5] = ChannelId::Chat.into();

                w.write(&header).unwrap();
                w.write(msg.as_str().as_bytes()).unwrap();
            }
        }
    }
}
