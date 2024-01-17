pub use platform_serial::PlatformSerial;

use serial_embedded_hal::*;

pub use serial_embedded_hal::{BaudRate, CharSize, FlowControl, Parity, PortSettings, StopBits};

use core::default::Default;
use lazy_static::lazy_static;
use std::cell::Cell;
use std::sync::Mutex;
use std::sync::Once;

lazy_static! {
    static ref CONFIGURATION: Mutex<Option<PortSettings>> = Mutex::new(None);
    static ref PORT_PATH: Mutex<Option<String>> = Mutex::new(None);
    static ref SERIAL: Mutex<Cell<Option<HiddenSerialParts>>> = Mutex::new(Cell::new(None));
    static ref INIT_ONCE: Once = Once::new();
}

const SERIAL_ERR_MSG: &str = "Serial not initialized";

pub fn configure_serial(port_path: String, configuration: PortSettings) {
    INIT_ONCE.call_once(|| {
        let mut port = PORT_PATH.lock().unwrap();
        *port = Some(port_path.clone());
        let mut config = CONFIGURATION.lock().unwrap();
        *config = Some(configuration);

        let serial = Serial::new(&port_path, &configuration)
            .expect("Could not initialize port with given configuration");

        let (writer, reader) = serial.split();

        let mut serial_ref = SERIAL.lock().unwrap();
        *serial_ref = Cell::new(Some(HiddenSerialParts { reader, writer }));
    });
}

struct HiddenSerialParts {
    pub reader: serial_embedded_hal::Rx,
    pub writer: serial_embedded_hal::Tx,
}

pub struct LinuxSerial;

impl Default for LinuxSerial {
    fn default() -> Self {
        LinuxSerial
    }
}

impl embedded_hal::serial::Read<u8> for LinuxSerial {
    type Error = serial_core::Error;

    fn read(&mut self) -> Result<u8, nb::Error<Self::Error>> {
        SERIAL
            .lock()
            .as_mut()
            .expect(SERIAL_ERR_MSG)
            .get_mut()
            .as_mut()
            .expect(SERIAL_ERR_MSG)
            .reader
            .read()
    }
}

impl embedded_hal::serial::Write<u8> for LinuxSerial {
    type Error = serial_core::Error;

    fn write(&mut self, byte: u8) -> Result<(), nb::Error<Self::Error>> {
        SERIAL
            .lock()
            .as_mut()
            .expect(SERIAL_ERR_MSG)
            .get_mut()
            .as_mut()
            .expect(SERIAL_ERR_MSG)
            .writer
            .write(byte)
    }

    fn flush(&mut self) -> Result<(), nb::Error<Self::Error>> {
        SERIAL
            .lock()
            .as_mut()
            .expect(SERIAL_ERR_MSG)
            .get_mut()
            .as_mut()
            .expect(SERIAL_ERR_MSG)
            .writer
            .flush()
    }
}

impl PlatformSerial<u8> for LinuxSerial {}
