pub mod package;
pub mod installer;
pub mod library;

pub use package::*;
pub use installer::*;
pub use library::*;

// Re-exportação para facilitar o uso
pub mod prelude {
    pub use super::package::*;
    pub use super::installer::*;
    pub use super::library::*;
}
