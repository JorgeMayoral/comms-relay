use std::collections::HashMap;

use comms::publication::Publication;
use ulid::Ulid;

pub trait PublicationStorage {
    fn add(&mut self, publication: Publication);
    fn get(&self, id: Ulid) -> Option<&Publication>;
    fn list(&self) -> Vec<Publication>;
}

pub type HashMapStorage = HashMap<Ulid, Publication>;

impl PublicationStorage for HashMapStorage {
    fn add(&mut self, publication: Publication) {
        self.insert(publication.id().to_owned(), publication);
    }

    fn get(&self, id: Ulid) -> Option<&Publication> {
        self.get(&id)
    }

    fn list(&self) -> Vec<Publication> {
        self.values().cloned().collect()
    }
}
