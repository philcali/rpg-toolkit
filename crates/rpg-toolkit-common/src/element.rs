use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Element {
    Fire,
    Ice,
    Lightning,
    Wind,
    Earth,
    Light,
    Dark,
}

impl Element {
    pub fn all() -> &'static [Element] {
        &[
            Element::Fire,
            Element::Ice,
            Element::Lightning,
            Element::Wind,
            Element::Earth,
            Element::Light,
            Element::Dark,
        ]
    }
}
