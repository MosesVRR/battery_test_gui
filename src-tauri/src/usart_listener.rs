use std::io::Read;
use std::time::Duration;
use serialport::SerialPort;

pub fn usart_listen(port_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let baud_rate = 115200;
    let timeout = Duration::from_millis(1000);

    let mut port = serialport::new(port_name, baud_rate)
        .timeout(timeout)
        .open()?;

    let mut buffer: Vec<u8> = vec![0; 1024];
    loop {
        match port.read(buffer.as_mut_slice()) {
            Ok(bytes_read) => {
                println!("Read {} byte(s): {:?}", bytes_read, &buffer[..bytes_read]);
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                // Handle timeout error if necessary
            }
            Err(e) => {
                eprintln!("Error reading from serial port: {:?}", e);
                return Err(Box::new(e));
            }
        }
    }
}
