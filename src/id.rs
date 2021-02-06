const ID_SIZE: usize = 16; // Arbitrary
pub type Identifier = [char; ID_SIZE];

fn id(s: &str) -> Identifier {
    let mut id = [char::default(); ID_SIZE];

    for (i, c) in s.chars().take(ID_SIZE).enumerate() {
        id[i] = c;
    }

    id
}

#[derive(PartialEq, Copy, Clone)]
pub struct Id {
    id: Identifier,
}

impl std::fmt::Debug for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::iter::FromIterator;
        let id = String::from_iter(&self.id);

        f.write_str(&id)
    }
}

impl From<&str> for Id {
    fn from(s: &str) -> Self {
        Self { id: id(s) }
    }
}
