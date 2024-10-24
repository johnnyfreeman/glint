pub mod request_resolver;

pub trait Resolver {
    type Arguments;
    type Error;

    fn resolve(&self, arguments: Self::Arguments) -> Result<String, Self::Error>;
}
