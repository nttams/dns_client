pub enum QType {
    A,
    Aaaa,
    Opt,
    FutureUse,
}

pub enum QClass {
    In,
    FutureUse,
}

#[derive(PartialEq)] // todo: learn this
pub enum RCode {
    NoError,
    FormErr,
    ServFail,
    NXDomain,
    FutureUse,
}

impl QType {
    pub fn get_value(q_type: Self) -> Option<u16> {
        match q_type {
            QType::A => Some(1),
            QType::Aaaa => Some(28),
            QType::Opt => Some(41),
            _ => None,
        }
    }

    pub fn get_q_type(value: u16) -> Self {
        match value {
            1 => QType::A,
            28 => QType::Aaaa,
            41 => QType::Opt,
            _ => QType::FutureUse,
        }
    }
}

impl QClass {
    pub fn get_value(q_class: Self) -> Option<u16> {
        match q_class {
            QClass::In => Some(1),
            _ => None,
        }
    }

    pub fn get_q_class(value: u16) -> Self {
        match value {
            1 => QClass::In,
            _ => QClass::FutureUse,
        }
    }
}

impl RCode {
    pub fn get_value(&self) -> Option<u8> {
        match self {
            RCode::NoError => Some(0),
            RCode::FormErr => Some(1),
            RCode::ServFail => Some(2),
            RCode::NXDomain => Some(3),
            RCode::FutureUse => None,
        }
    }

    pub fn get_r_code(value: u8) -> Self {
        // r_code value has only 4 bits.
        assert!(value < 16);
        match value {
            0 => RCode::NoError,
            1 => RCode::FormErr,
            2 => RCode::ServFail,
            3 => RCode::NXDomain,
            _ => RCode::FutureUse,
        }
    }
}

pub fn parse_type_and_class(msg: &[u8], offset: usize) -> (u16, u16, u16) {
    let q_type = (msg[offset + 0] as u16) << 8 | (msg[offset + 1] as u16);
    let q_class = (msg[offset + 2] as u16) << 8 | (msg[offset + 3] as u16);

    (q_type, q_class, 4)
}

pub fn encode_type_and_class(q_type: u16, q_class: u16) -> Vec<u8> {
    let mut result = Vec::new();

    result.push((q_type >> 8) as u8);
    result.push((q_type >> 0) as u8);
    result.push((q_class >> 8) as u8);
    result.push((q_class >> 0) as u8);

    result
}
