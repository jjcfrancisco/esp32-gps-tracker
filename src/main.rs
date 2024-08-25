pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>;

mod esp32;

fn main() -> Result<()> {
    _ = esp32::test();
    Ok(())
}

