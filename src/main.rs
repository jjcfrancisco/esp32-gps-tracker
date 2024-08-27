pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>;

mod esp32;
mod servo;
mod wifi;

fn main() -> Result<()> {
    esp32::run()?;
    Ok(())
}

