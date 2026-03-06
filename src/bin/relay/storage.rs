use std::collections::HashMap;

use comms::publication::Publication;

pub trait PublicationStorage {
    fn add(&mut self, publication: Publication);
    fn get(&self, id: String) -> Option<&Publication>;
    fn list(&self) -> Vec<Publication>;
}

pub type HashMapStorage = HashMap<String, Publication>;

impl PublicationStorage for HashMapStorage {
    fn add(&mut self, publication: Publication) {
        self.insert(publication.id().to_owned(), publication);
    }

    fn get(&self, id: String) -> Option<&Publication> {
        self.get(&id)
    }

    fn list(&self) -> Vec<Publication> {
        self.values().cloned().collect()
    }
}
