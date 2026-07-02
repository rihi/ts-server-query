use std::collections::HashMap;

use crate::error::QueryError;
use crate::protocol::{bool_field, parse_fields, required_string, required_u64};
use crate::response::Response;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Channel {
    pub cid: u64,
    pub pid: u64,
    pub channel_order: u64,
    pub channel_name: String,
    pub topic: Option<String>,
    pub flags: ChannelFlags,
    pub fields: HashMap<String, String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ChannelFlags {
    pub default: bool,
    pub password: bool,
    pub permanent: bool,
    pub semi_permanent: bool,
}

pub fn parse_channel_list(response: &Response) -> Result<Vec<Channel>, QueryError> {
    let mut channels = Vec::new();

    for line in response.lines() {
        if line.is_empty() {
            continue;
        }

        for item in line.split('|') {
            let fields = parse_fields(item)?;

            channels.push(Channel {
                cid: required_u64(&fields, "cid")?,
                pid: required_u64(&fields, "pid")?,
                channel_order: required_u64(&fields, "channel_order")?,
                channel_name: required_string(&fields, "channel_name")?,
                topic: fields.get("channel_topic").cloned(),
                flags: ChannelFlags {
                    default: bool_field(&fields, "channel_flag_default"),
                    password: bool_field(&fields, "channel_flag_password"),
                    permanent: bool_field(&fields, "channel_flag_permanent"),
                    semi_permanent: bool_field(&fields, "channel_flag_semi_permanent"),
                },
                fields,
            });
        }
    }

    Ok(channels)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::response::Response;

    #[test]
    fn parses_channel_list() {
        let response = Response::new(
            vec![
                "cid=1 pid=0 channel_order=0 channel_name=Default\\sChannel channel_topic=Root channel_flag_default=1 channel_flag_password=0 channel_flag_permanent=1 channel_flag_semi_permanent=0|cid=2 pid=1 channel_order=1 channel_name=Support channel_flag_default=0 channel_flag_password=1 channel_flag_permanent=0 channel_flag_semi_permanent=1".to_owned(),
            ],
            HashMap::new(),
        );

        let channels = parse_channel_list(&response).unwrap();

        assert_eq!(channels.len(), 2);
        assert_eq!(channels[0].cid, 1);
        assert_eq!(channels[0].channel_name, "Default Channel");
        assert_eq!(channels[0].topic.as_deref(), Some("Root"));
        assert!(channels[0].flags.default);
        assert!(channels[1].flags.password);
        assert!(channels[1].flags.semi_permanent);
    }
}
