pub mod statbin;
pub mod invbin;
pub mod botbin;
pub mod dump;
pub mod reserialize;
pub mod diff;
pub mod zip;
mod build_stuff;
pub mod patch;
pub mod langbin;
pub mod techbin;
pub mod botmgmt;
pub mod chatbroadcast;

pub trait ProcelioCLITool {
    fn command(&self) -> &'static str;

    fn usage(&self);

    fn tool(&self, args: Vec<String>);
}