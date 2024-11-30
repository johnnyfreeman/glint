pub mod env_var_resolver;
pub mod request_resolver;

pub trait Resolver {
    type Arguments;
    type Error;

    fn resolve(&mut self, arguments: Self::Arguments) -> Result<String, Self::Error>;
}
