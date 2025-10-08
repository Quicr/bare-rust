use num_enum::{IntoPrimitive, TryFromPrimitive};
use ui_app::{FromNet, ToNet, FRAME_SIZE};

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
                let packet = Ping;
                packet.write(&mut self.tx);
            }

            ToNet::AudioFrame(frame) => {
                let mut frame_data = [0u8; 2 * FRAME_SIZE];

                for (x8, x16) in frame_data.chunks_mut(2).zip(frame.0.iter()) {
                    x8.copy_from_slice(&x16.to_be_bytes());
                }

                let packet = Message {
                    channel_id: ChannelId::Ptt,
                    body: MediaMessage {
                        is_last: false,
                        payload: &frame_data,
                    },
                };

                packet.write(&mut self.tx);
            }

            ToNet::Chat(chat) => {
                let packet = Message {
                    channel_id: ChannelId::Ptt,
                    body: ChatMessage {
                        payload: &chat.as_bytes(),
                    },
                };

                packet.write(&mut self.tx);
            }

            to_net => {
                defmt::info!("skipping tx of {:?}", to_net);
                // TODO write other message types
            }
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug, IntoPrimitive, TryFromPrimitive)]
enum PacketType {
    Ping = 0x03,
    Message = 0x09,
}

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug, IntoPrimitive, TryFromPrimitive)]
enum MessageType {
    Media = 0x01,
    AIRequest = 0x02,
    AIResponse = 0x03,
    Chat = 0x04,
}

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug, IntoPrimitive, TryFromPrimitive)]
enum ChannelId {
    Ptt = 0,
    PttAi = 1,
    Chat = 2,
    ChatAi = 3,
    Count = 4,
}

trait Tlv {
    const TYPE: PacketType;
    fn len(&self) -> usize;
    fn write_payload(&self, w: &mut impl embedded_io::Write);

    fn write(&self, w: &mut impl embedded_io::Write) {
        let len = self.len() as u32;
        w.write(&[Self::TYPE.into()]).unwrap();
        w.write(&len.to_be_bytes()).unwrap();
        self.write_payload(w);
    }
}

struct Ping;

impl Tlv for Ping {
    const TYPE: PacketType = PacketType::Ping;

    fn len(&self) -> usize {
        0
    }

    fn write_payload(&self, _w: &mut impl embedded_io::Write) {}
}

trait MessageBody {
    const MESSAGE_TYPE: MessageType;
    fn len(&self) -> usize;
    fn write(&self, w: &mut impl embedded_io::Write);
}

struct MediaMessage<'a> {
    is_last: bool,
    payload: &'a [u8],
}

impl<'a> MessageBody for MediaMessage<'a> {
    const MESSAGE_TYPE: MessageType = MessageType::Media;

    fn len(&self) -> usize {
        // is_last + payload_len + payload
        1 + 4 + self.payload.len()
    }

    fn write(&self, w: &mut impl embedded_io::Write) {
        let len = self.payload.len() as u32;
        w.write(&[self.is_last.into()]).unwrap();
        w.write(&len.to_be_bytes()).unwrap();
        w.write(self.payload).unwrap();
    }
}

struct ChatMessage<'a> {
    payload: &'a [u8],
}

impl<'a> MessageBody for ChatMessage<'a> {
    const MESSAGE_TYPE: MessageType = MessageType::Chat;

    fn len(&self) -> usize {
        // payload_len + payload
        4 + self.payload.len()
    }

    fn write(&self, w: &mut impl embedded_io::Write) {
        let len = self.payload.len() as u32;
        w.write(&len.to_be_bytes()).unwrap();
        w.write(self.payload).unwrap();
    }
}

struct Message<B: MessageBody> {
    channel_id: ChannelId,
    body: B,
}

impl<B: MessageBody> Tlv for Message<B> {
    const TYPE: PacketType = PacketType::Message;

    fn len(&self) -> usize {
        // ChannelId + MessageType + Body
        2 + self.body.len()
    }

    fn write_payload(&self, w: &mut impl embedded_io::Write) {
        w.write(&[self.channel_id.into(), B::MESSAGE_TYPE.into()])
            .unwrap();
        self.body.write(w);
    }
}
