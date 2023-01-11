use core::{marker::PhantomData, ptr::NonNull};

// This is a free list that lives inside the memory it reports about
#[derive(Debug)]
pub struct InlineFreeList<T> {
    head: Link<T>,
    tail: Link<T>,
    len: usize,
    _phantom: PhantomData<T>,
}

type Link<T> = Option<NonNull<Node<T>>>;

#[repr(C)]
struct Node<T> {
    next: Link<T>,
    prev: Link<T>,
    data: T,
}

impl<T> InlineFreeList<T> {
    pub const fn new() -> Self {
        Self {
            head: None,
            tail: None,
            len: 0,
            _phantom: PhantomData,
        }
    }

    pub fn push_head(&mut self, free_addr: usize, data: T) {
        unsafe {
            let new = NonNull::new_unchecked(free_addr as *mut Node<T>);
            *new.as_ptr() = Node {
                data,
                next: None,
                prev: None,
            };

            if let Some(old_head) = self.head {
                (*old_head.as_ptr()).prev = Some(new);
                (*new.as_ptr()).next = Some(old_head);
            } else {
                debug_assert!(self.tail.is_none());
                debug_assert!(self.head.is_none());
                debug_assert!(self.len == 0);

                self.tail = Some(new);
            }

            self.head = Some(new);
            self.len += 1;
        }
    }

    pub fn pop_head(&mut self) -> Option<usize> {
        unsafe {
            self.head.map(|old_head| {
                self.head = (*old_head.as_ptr()).next;

                if let Some(new_head) = self.head {
                    (*new_head.as_ptr()).prev = None;
                } else {
                    debug_assert!(self.len == 1);
                    self.tail = None;
                }

                self.len -= 1;

                old_head.as_ptr() as usize
            })
        }
    }

    pub fn push_tail(&mut self, free_addr: usize, data: T) {
        unsafe {
            let new = NonNull::new_unchecked(free_addr as *mut Node<T>);
            *new.as_ptr() = Node {
                data,
                next: None,
                prev: None,
            };

            if let Some(old_tail) = self.tail {
                (*old_tail.as_ptr()).next = Some(new);
                (*new.as_ptr()).prev = Some(old_tail);
            } else {
                debug_assert!(self.tail.is_none());
                debug_assert!(self.head.is_none());
                debug_assert!(self.len == 0);

                self.head = Some(new);
            }

            self.tail = Some(new);
            self.len += 1;
        }
    }

    pub fn pop_tail(&mut self) -> Option<usize> {
        unsafe {
            self.tail.map(|old_head| {
                let result = old_head.as_ptr() as usize;

                self.head = (*old_head.as_ptr()).prev;

                if let Some(new_head) = self.head {
                    (*new_head.as_ptr()).next = None;
                } else {
                    debug_assert!(self.len == 1);
                    self.head = None;
                }

                self.len -= 1;

                result
            })
        }
    }

    pub fn pop_at_address(&mut self, node_addr: usize) {
        debug_assert!(self.head.is_some());
        debug_assert!(self.tail.is_some());

        let node_ptr = node_addr as *mut Node<T>;

        unsafe {
            let next_node = (*node_ptr).next;
            let prev_node = (*node_ptr).prev;

            if let Some(mut next_node) = next_node {
                next_node.as_mut().prev = prev_node;
            }

            if let Some(mut prev_node) = prev_node {
                prev_node.as_mut().next = next_node;
            }

            if node_ptr == self.head.unwrap().as_ptr() {
                self.head = next_node;
            }

            if node_ptr == self.tail.unwrap().as_ptr() {
                self.tail = prev_node;
            }
        }

        self.len -= 1;
    }

    pub fn pop_on<F>(&mut self, f: F) -> Option<usize>
    where
        F: Fn(&T) -> bool,
    {
        let mut current_node = self.head;

        while let Some(node_ptr) = current_node {
            unsafe {
                if f(&(*node_ptr.as_ptr()).data) {
                    let node_addr = node_ptr.as_ptr() as usize;
                    self.pop_at_address(node_addr);
                    return Some(node_addr);
                } else {
                    current_node = (*node_ptr.as_ptr()).next;
                }
            }
        }

        return None;
    }

    pub fn len(&self) -> usize {
        self.len
    }
}
