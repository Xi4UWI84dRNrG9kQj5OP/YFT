pub trait PredecessorSet<T> {
    fn predecessor(&self, number: T) -> Option<T>;
    fn insert(&mut self, element: T);
    fn delete(&mut self, element: T);
    fn successor(&self, number: T) -> Option<T>;
    // Optional
    fn minimum(&self) -> Option<T>;
    fn maximum(&self) -> Option<T>;
    fn contains(&self, number: T) -> bool;
}