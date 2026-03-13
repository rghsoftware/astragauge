use std::collections::VecDeque;

pub struct RingBuffer<T> {
  capacity: usize,
  inner: VecDeque<T>,
}

impl<T> RingBuffer<T> {
  #[must_use]
  pub fn new(capacity: usize) -> Self {
    assert!(capacity > 0, "RingBuffer capacity must be greater than 0");
    Self {
      capacity,
      inner: VecDeque::with_capacity(capacity),
    }
  }

  pub fn push(&mut self, item: T) {
    if self.inner.len() >= self.capacity {
      self.inner.pop_front(); // Evict oldest
    }
    self.inner.push_back(item);
  }

  pub fn iter(&self) -> impl Iterator<Item = &T> {
    self.inner.iter()
  }

  pub fn len(&self) -> usize {
    self.inner.len()
  }

  pub fn is_empty(&self) -> bool {
    self.inner.is_empty()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_respects_capacity() {
    let mut buffer = RingBuffer::new(3);
    buffer.push(1);
    buffer.push(2);
    buffer.push(3);
    buffer.push(4); // Should evict 1
    assert_eq!(buffer.len(), 3);
    let items: Vec<_> = buffer.iter().copied().collect();
    assert_eq!(items, vec![2, 3, 4]);
  }

  #[test]
  fn test_fifo_eviction() {
    let mut buffer = RingBuffer::new(2);
    buffer.push(1);
    buffer.push(2);
    buffer.push(3); // Evicts 1
    buffer.push(4); // Evicts 2
    let items: Vec<_> = buffer.iter().copied().collect();
    assert_eq!(items, vec![3, 4]);
  }

  #[test]
  #[should_panic(expected = "RingBuffer capacity must be greater than 0")]
  fn test_zero_capacity_panics() {
    let _ = RingBuffer::<i32>::new(0);
  }
}
