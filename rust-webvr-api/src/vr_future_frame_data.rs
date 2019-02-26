use vr_frame_data::VRFrameData;

#[cfg(not(feature = "ipc"))]
use std::sync::mpsc::{channel, Sender, Receiver};

#[cfg(feature = "ipc")]
use ipc_channel::ipc::{channel as ipc_channel, IpcSender as Sender, IpcReceiver as Receiver};

#[cfg(feature = "ipc")]
fn channel<T>() -> (Sender<T>, Receiver<T>) where
    T: for<'de> serde::de::Deserialize<'de> + serde::ser::Serialize,
{
    ipc_channel().expect("Failed to create IPC channel")
}

#[cfg_attr(feature = "ipc", derive(Serialize, Deserialize))]
enum State<T, U> {
    Resolved(T),
    Blocked(U),
}

#[cfg_attr(feature = "ipc", derive(Serialize, Deserialize))]
pub struct VRFutureFrameData(State<VRFrameData, Receiver<VRFrameData>>);

#[cfg_attr(feature = "ipc", derive(Serialize, Deserialize))]
pub struct VRResolveFrameData(State<(), Sender<VRFrameData>>);

impl VRFutureFrameData {
    pub fn resolved(data: VRFrameData) -> VRFutureFrameData {
        VRFutureFrameData(State::Resolved(data))
    }

    pub fn blocked() -> (VRResolveFrameData, VRFutureFrameData) {
        let (send, recv) = channel();
        (
            VRResolveFrameData(State::Blocked(send)),
            VRFutureFrameData(State::Blocked(recv)),
        )
    }

    pub fn block(self) -> VRFrameData {
        match self {
            VRFutureFrameData(State::Resolved(result)) => result,
            VRFutureFrameData(State::Blocked(recv)) => recv.recv().expect("Failed to get frame data"),
        }
    }
}

impl VRResolveFrameData {
    pub fn resolve(&mut self, data: VRFrameData) -> Result<(), ()> {
        match *self {
            VRResolveFrameData(State::Resolved(())) => return Err(()),
            VRResolveFrameData(State::Blocked(ref send)) => send.send(data).expect("Failed to put frame data"),
        };
        self.0 = State::Resolved(());
        Ok(())
    }
}
