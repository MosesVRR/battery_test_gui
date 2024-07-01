
/// Decodes a slice of bytes by removing the prepended 0xB3 and verifying the checksum.
///
/// The checksum is verified as the XOR of every byte in the input slice. If the checksum does not match, the function will return an error.
///
/// # Arguments
///
/// * `bytes` - A slice of bytes to be decoded. It must have at least 2 bytes: the prepended 0xB3 and the checksum.
///
/// # Returns
///
/// A `Result<Vec<u8>, &'static str>` containing the decoded bytes or an error message if the checksum is invalid.
///
/// # Example
///
/// ```
/// let encoded = vec![0xB3, 0x01, 0x02, 0x03, 0xB3 ^ 0x01 ^ 0x02 ^ 0x03];
/// let decoded = decode(&encoded).unwrap();
/// assert_eq!(decoded, vec![0x01, 0x02, 0x03]);
/// ```

const DELIMITER : u8 = 0xB3;


enum Command {
	Ping(PingPayload),
	AssignID(AssignIDPayload),
	RequestData(RequestDataPayload),
	SetStandby,
	SetDischarge,
	SetCharge,
	AnnounceCompletion(AnnounceCompletionPayload)
}

struct PingPayload {
	identification : u8
}

struct AssignIDPayload {
	new_id : u8
}

struct RequestDataPayload {
	battery_temperature : u16,
	bench_temperature : u16,
	load_temperature : u16,
	voltage : u16,
	current : u16,
}


struct AnnounceCompletionPayload {
	flag : u8
}

impl Command {

	fn frame_id(&self) -> u8 {
		match self {
			Command::Ping(_payload) => 0x00,
			Command::AssignID(_payload) => 0x01,
			Command::RequestData(_payload) => 0x02,
			Command::SetStandby => 0x04,
			Command::SetDischarge => 0x05,
			Command::SetCharge => 0x06,
			Command::AnnounceCompletion(_payload) => 0x07
		}
	}

	fn encode(&self) -> Vec<u8> {

		let mut result = vec![DELIMITER, self.frame_id()] ;

		result.extend(match self {

			Command::Ping(payload) => {
				vec![payload.identification]
			},

			Command::AssignID(payload) => {
				vec![payload.new_id]
			},

			Command::RequestData(payload) => {
				vec![
					(payload.battery_temperature >> 8) as u8,		//it is this way to convert the u16 to two u8
					(payload.battery_temperature & 0xFF) as u8,

					(payload.bench_temperature >> 8) as u8,		
					(payload.bench_temperature & 0xFF) as u8,

					(payload.load_temperature >> 8) as u8,		
					(payload.load_temperature & 0xFF) as u8,
					
					(payload.voltage >> 8) as u8,		
					(payload.voltage & 0xFF) as u8,

					(payload.current >> 8) as u8,		
					(payload.current & 0xFF) as u8,
				]
			},

			Command::SetCharge => {
				vec![]
			},
			
			Command::SetDischarge => {
				vec![]
			},
			Command::SetStandby => {
				vec![]
			},
			Command::AnnounceCompletion(payload) => {
				vec![payload.flag]
			}
		});

		let mut checksum = result[0];
		let mut index = 1;

		while index < result.len() {
			checksum ^= result[index];
			index+=1;
		}

		result.push(checksum);
		result

	}
}
/// Encodes a slice of bytes by prepending 0xB3 and appending a checksum.
///
/// The checksum is calculated as the XOR of every byte in the resulting slice, including the prepended 0xB3.
///
/// # Arguments
///
/// * `bytes` - A slice of bytes to be encoded.
///
/// # Returns
///
/// A `Vec<u8>` containing the encoded bytes.
///
/// # Example
///
/// ```
/// let data = vec![0x01, 0x02, 0x03];
/// let encoded = [0xB3, 0x00, 0x05, 0xB3 ^ 0x00 ^ 0x05];
/// assert_eq!(encoded, vec![0xB3, 0x01, 0x02, 0x03, 0xB3 ^ 0x01 ^ 0x02 ^ 0x03]);
/// ``

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_encode() {


		let encoded = vec![0xB3, 0x00, 0x05, 0xB3 ^ 0x00 ^ 0x05];

		let command = Command::Ping(PingPayload{ identification : 0x05 });

		assert_eq!(encoded , command.encode());
		/* 
		let data = vec![0x01, 0x02, 0x03];
		let encoded = encode(&data);
		assert_eq!(encoded, vec![0xB3, 0x01, 0x02, 0x03, 0xB3 ^ 0x01 ^ 0x02 ^ 0x03]);

		let data = vec![0x00, 0xFF, 0x55];
		let encoded = encode(&data);
		assert_eq!(encoded, vec![0xB3, 0x00, 0xFF, 0x55, 0xB3 ^ 0x00 ^ 0xFF ^ 0x55]);
		*/
	}

	#[test]
	fn test_decode() {
		let encoded = vec![0xB3, 0x01, 0x02, 0x03, 0xB3 ^ 0x01 ^ 0x02 ^ 0x03];
		let decoded = decode(&encoded).unwrap();
		assert_eq!(decoded, vec![0x01, 0x02, 0x03]);

		let encoded = vec![0xB3, 0x00, 0xFF, 0x55, 0xB3 ^ 0x00 ^ 0xFF ^ 0x55];
		let decoded = decode(&encoded).unwrap();
		assert_eq!(decoded, vec![0x00, 0xFF, 0x55]);
	}

	#[test]
	fn test_decode_invalid_checksum() {
		let encoded = vec![0xB3, 0x01, 0x02, 0x03, 0x00];
		let result = decode(&encoded);
		assert!(result.is_err());
		assert_eq!(result.err(), Some("Invalid checksum"));
	}

	#[test]
	fn test_decode_too_short() {
		let encoded = vec![0xB3];
		let result = decode(&encoded);
		assert!(result.is_err());
		assert_eq!(result.err(), Some("Input is too short to be valid"));
	}
}
