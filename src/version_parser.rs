use clap::builder::{StringValueParser, TypedValueParser};
use clap::error::ErrorKind;
use hex::FromHex;

#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
pub struct VersionValueParser {}

impl VersionValueParser {
    pub fn new() -> Self {
        Self {}
    }
}

impl TypedValueParser for VersionValueParser {
    type Value = [u8; 4];

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        TypedValueParser::parse(self, cmd, arg, value.to_owned())
    }

    fn parse(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: std::ffi::OsString,
    ) -> Result<Self::Value, clap::Error> {
        let value = StringValueParser::parse(
            &StringValueParser::new(),
            cmd,
            arg,
            value
        )?;
        let value = <[u8; 4]>::from_hex(value).map_err(|error| {
            clap::Error::raw(
                ErrorKind::InvalidValue,
                format!("{}\n", error)
            )
        })?;
        Ok(value)
    }
}

impl Default for VersionValueParser {
    fn default() -> Self {
        Self::new()
    }
}