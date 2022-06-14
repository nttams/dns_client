use super::dns_types::RCode;
use rand::Rng;

pub struct Header {
    id: u16, // todo: random generate, used to match query and response
    header_flags: HeaderFlags,
    qd_count: u16,
    an_count: u16,
    ns_count: u16,
    ar_count: u16,
}

pub struct HeaderFlags {
    qr: bool,
    op_code: u8,
    aa: bool,
    tc: bool,
    rd: bool,
    ra: bool,
    z: bool,
    ad: bool,
    cd: bool,
    r_code: RCode,
}

impl Header {
    pub fn new() -> Self {
        Self {
            // todo: keep track of these to avoid duplicated
            id: Self::generate_random_id(),
            header_flags: HeaderFlags::new(),
            qd_count: 0,
            an_count: 0,
            ns_count: 0,
            ar_count: 0,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        // dns header has the length of 12 octets
        let mut result: Vec<u8> = vec![0; 12];

        result[0] = (self.id >> 8) as u8;
        result[1] = (self.id >> 0) as u8;

        let header_flags_encoded = self.header_flags.encode();
        result[2] = header_flags_encoded[0];
        result[3] = header_flags_encoded[1];

        result[4] = (self.qd_count >> 8) as u8;
        result[5] = (self.qd_count >> 0) as u8;

        result[6] = (self.an_count >> 8) as u8;
        result[7] = (self.an_count >> 0) as u8;

        result[8] = (self.ns_count >> 8) as u8;
        result[9] = (self.ns_count >> 0) as u8;

        // todo: there's a lot of duplicated code like these
        result[10] = (self.ar_count >> 8) as u8;
        result[11] = (self.ar_count >> 0) as u8;

        result
    }

    // input as slice, len = 12 octets
    // todo: test
    pub fn parse(msg: &[u8], start_pos: usize) -> (Self, u16) {
        assert!(msg.len() > 12);
        let header = Self {
            id: (msg[start_pos] as u16) << 8 | (msg[start_pos + 1] as u16),
            header_flags: HeaderFlags::parse(&msg[start_pos + 2..start_pos + 4]),
            qd_count: (msg[start_pos + 4] as u16) << 8 | (msg[start_pos + 5] as u16),
            an_count: (msg[start_pos + 6] as u16) << 8 | (msg[start_pos + 7] as u16),
            ns_count: (msg[start_pos + 8] as u16) << 8 | (msg[start_pos + 9] as u16),
            ar_count: (msg[start_pos + 10] as u16) << 8 | (msg[start_pos + 11] as u16),
        };
        (header, 12)
    }

    pub fn get_an_count(&self) -> u16 {
        self.an_count
    }
    pub fn get_ns_count(&self) -> u16 {
        self.ns_count
    }
    pub fn get_ar_count(&self) -> u16 {
        self.ar_count
    }

    pub fn inc_qd_count(&mut self) {
        self.qd_count += 1;
    }

    pub fn inc_ar_count(&mut self) {
        self.ar_count += 1;
    }

    pub fn enable_recursion(&mut self) {
        self.header_flags.enable_recursion();
    }

    fn generate_random_id() -> u16 {
        rand::thread_rng().gen_range(1..=65535)
    }
}

impl HeaderFlags {
    pub fn new() -> Self {
        Self {
            qr: false,
            op_code: 0,
            aa: false,
            tc: false,
            rd: false,
            ra: false,
            z: false,
            ad: false,
            cd: false,
            r_code: RCode::NoError,
        }
    }

    // todo: this needs to be tested carefully
    pub fn encode(&self) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::from([0, 0]);

        if self.qr {
            result[0] |= 1 << 7
        };

        assert!(self.op_code < 16);
        result[0] |= self.op_code << 3;

        if self.aa {
            result[0] |= 1 << 2
        };
        if self.tc {
            result[0] |= 1 << 1
        };
        if self.rd {
            result[0] |= 1 << 0
        };

        // second octet
        if self.ra {
            result[1] |= 1 << 7
        };
        if self.z {
            result[1] |= 1 << 6
        };
        if self.ad {
            result[1] |= 1 << 5
        };
        if self.cd {
            result[1] |= 1 << 4
        };

        let r_code_value = self
            .r_code
            .get_value()
            .expect("never see FutureUse in encoding state");

        assert!(r_code_value < 16);
        result[1] |= r_code_value << 0;

        result
    }

    // input as slice, len = 2 octets
    // todo: test
    pub fn parse(msg: &[u8]) -> Self {
        let mut result = Self {
            qr: false,
            op_code: 0,
            aa: false,
            tc: false,
            rd: false,
            ra: false,
            z: false,
            ad: false,
            cd: false,
            r_code: RCode::NoError,
        };

        result.qr = (msg[0] & 0b1000_0000) >> 7 == 1;
        result.op_code = (msg[0] & 0b0111_1000) >> 3;
        result.aa = (msg[0] & 0b0000_0100) >> 2 == 1;
        result.tc = (msg[0] & 0b0000_0010) >> 1 == 1;
        result.rd = (msg[0] & 0b0000_0001) >> 0 == 1;

        // second octet
        result.ra = (msg[1] & 0b1000_0000) >> 7 == 1;
        result.z = (msg[1] & 0b0100_0000) >> 6 == 1;
        result.ad = (msg[1] & 0b0010_0000) >> 5 == 1;
        result.cd = (msg[1] & 0b0001_0000) >> 4 == 1;
        result.r_code = RCode::get_r_code((msg[1] & 0b0000_1111) >> 0);

        result
    }

    pub fn enable_recursion(&mut self) {
        self.rd = true;
    }
}

#[cfg(test)]
mod tests {
    use super::HeaderFlags;
    use super::RCode;

    #[test]
    fn header_flags_encode() {
        // case0, all OFF
        let header_flags = HeaderFlags {
            qr: false,
            op_code: 0,
            aa: false,
            tc: false,
            rd: false,
            ra: false,
            z: false,
            ad: false,
            cd: false,
            r_code: RCode::NoError,
        };

        let encoded_header_flags = header_flags.encode();
        assert!(encoded_header_flags[0] == 0);
        assert!(encoded_header_flags[1] == 0);

        // case1, all ON
        // except RCode::NXDomain (3, 0b0011), not implemented yet for RCODE=15
        let header_flags = HeaderFlags {
            qr: true,
            op_code: 0b1111,
            aa: true,
            tc: true,
            rd: true,
            ra: true,
            z: true,
            ad: true,
            cd: true,
            r_code: RCode::NXDomain,
        };

        let encoded_header_flags = header_flags.encode();
        assert!(encoded_header_flags[0] == 0b1111_1111);
        assert!(encoded_header_flags[1] == 0b1111_0011);
    }

    #[test]
    fn header_flags_parse() {
        let header_flags_encoded = [0, 0];
        let header_flags = HeaderFlags::parse(&header_flags_encoded[..]);

        assert!(header_flags.qr == false);
        assert!(header_flags.op_code == 0);
        assert!(header_flags.aa == false);
        assert!(header_flags.tc == false);
        assert!(header_flags.rd == false);
        assert!(header_flags.ra == false);
        assert!(header_flags.z == false);
        assert!(header_flags.ad == false);
        assert!(header_flags.cd == false);
        assert!(header_flags.r_code == RCode::NoError);
    }
}
