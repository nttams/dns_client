use super::dns_types;
use super::dns_types::{QClass, QType};
use super::domain_name::DomainName;

pub struct Records {
    pub records: Vec<Record>,
}

pub struct Record {
    pub q_name: DomainName,
    pub q_type: u16,
    pub q_class: u16,
    pub ttl: u32,
    rd_length: u16,
    r_data: Vec<u8>,
}

pub struct OptRecord {
    pub record: Record,
    udp_payload_size: u16,
    extended_r_code: u8,
    version: u8,
    do_bit: bool,
    z: u16, //15 bits
}

impl Records {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    pub fn push(&mut self, record: Record) {
        self.records.push(record)
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut rrs = Vec::new();
        for rr in &self.records {
            let mut encoded_rr = rr.encode();
            rrs.append(&mut encoded_rr);
        }
        rrs
    }

    pub fn get_records(&self) -> &Vec<Record> {
        &self.records
    }
}

impl Record {
    pub fn new() -> Self {
        Self {
            q_name: DomainName::new(""),
            q_type: QType::get_value(QType::A).expect("100% sure"),
            q_class: QClass::get_value(QClass::In).expect("100% sure"),
            ttl: 0,
            rd_length: 0,
            r_data: Vec::new(),
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut result = Vec::new();
        let mut domain_name = self.q_name.encode();
        let mut type_and_class = dns_types::encode_type_and_class(self.q_type, self.q_class);

        result.append(&mut domain_name);
        result.append(&mut type_and_class);

        // ttl
        result.push(((self.ttl & 0b_1111_1111_0000_0000_0000_0000_0000_0000) >> 24) as u8);
        result.push(((self.ttl & 0b_0000_0000_1111_1111_0000_0000_0000_0000) >> 16) as u8);
        result.push(((self.ttl & 0b_0000_0000_0000_0000_1111_1111_0000_0000) >> 08) as u8);
        result.push(((self.ttl & 0b_0000_0000_0000_0000_0000_0000_1111_1111) >> 00) as u8);

        // rd_data
        result.push(((self.rd_length & 0b_1111_1111_0000_0000) >> 8) as u8);
        result.push(((self.rd_length & 0b_0000_0000_1111_1111) >> 0) as u8);

        // r_data
        for i in 0..self.rd_length {
            result.push(self.r_data[i as usize]);
        }

        result
    }

    pub fn parse(msg: &[u8], offset: usize) -> (Self, u16) {
        let mut result = Self::new();

        let mut pos = offset;

        let (domain_name, parsed_count) = DomainName::parse(msg, pos);
        pos += parsed_count as usize;
        result.q_name = domain_name;

        let (q_type, q_class, parsed_count) = dns_types::parse_type_and_class(msg, pos);
        pos += parsed_count as usize;
        result.q_type = q_type;
        result.q_class = q_class;

        // ttl
        result.ttl |= (msg[pos] as u32) << 24;
        pos += 1;
        result.ttl |= (msg[pos] as u32) << 16;
        pos += 1;
        result.ttl |= (msg[pos] as u32) << 8;
        pos += 1;
        result.ttl |= (msg[pos] as u32) << 0;
        pos += 1;

        // rd_length
        result.rd_length |= (msg[pos] as u16) << 8;
        pos += 1;
        result.rd_length |= (msg[pos] as u16) << 0;
        pos += 1;

        for _ in 0..result.rd_length {
            result.r_data.push(msg[pos]);
            pos += 1;
        }

        let total_parsed_count: u16 = (pos - offset)
            .try_into()
            .expect("can not happen ever as msg length is controlled");

        (result, total_parsed_count)
    }

    pub fn get_data(&self) -> &Vec<u8> {
        &self.r_data
    }

    pub fn set_q_name(&mut self, domain_name: &str) {
        let name = DomainName::new(domain_name);
        self.q_name = name;
    }

    pub fn set_q_type(&mut self, q_type: QType) {
        let q_type_value = QType::get_value(q_type);
        if let Some(value) = q_type_value {
            self.q_type = value;
        }
        // todo: else case?
    }

    pub fn set_q_class(&mut self, q_class: QClass) {
        let q_class_value = QClass::get_value(q_class);
        if let Some(value) = q_class_value {
            self.q_class = value;
        }
        // todo: else case?
    }
}

impl OptRecord {
    pub fn new(udp_payload_size: u16) -> Self {
        let mut rr = Record::new();
        rr.set_q_name("\0");
        rr.set_q_type(QType::Opt);
        rr.q_class = udp_payload_size;

        OptRecord {
            record: rr,
            udp_payload_size: udp_payload_size,
            extended_r_code: 0,
            version: 0,
            do_bit: false,
            z: 0,
        }
    }
}
