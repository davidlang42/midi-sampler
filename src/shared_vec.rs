use std::sync::Arc;

pub struct SharedVec<T>(Arc<Vec<T>>);

impl<T> SharedVec<T> {
    fn _instances(&self) -> usize {
        Arc::strong_count(&self.0)
    }
}

impl<T> From<Vec<T>> for SharedVec<T> {
    fn from(value: Vec<T>) -> Self {
        Self(Arc::new(value))
    }
}

impl<T> AsRef<[T]> for SharedVec<T> {
    fn as_ref(&self) -> &[T] {
        &self.0
    }
}

impl<T> Clone for SharedVec<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}