use core::{marker::PhantomData, ptr::NonNull};

use crate::println;

// This is a free list of buddy blocks
#[repr(C)]
pub struct FreeList<'a> {
    head: Option<NonNull<NodeLink<'a>>>,
    _phantom: PhantomData<&'a NodeLink<'a>>,
}

struct NodeLink<'a> {
    next: Option<NonNull<NodeLink<'a>>>,
    prev: Option<NonNull<NodeLink<'a>>>,
    _phantom: PhantomData<&'a NodeLink<'a>>,
}

impl<'a> FreeList<'a> {
    pub fn new_empty() -> Self {
        Self {
            head: None,
            _phantom: PhantomData,
        }
    }

    pub fn pop(&mut self) -> Option<usize> {
        if let Some(mut head) = self.head {
            self.head = unsafe { head.as_mut().next };

            return Some(head.as_ptr() as usize);
        } else {
            return None;
        }
    }

    pub fn remove_at_address(&mut self, node_addr: usize) {
        debug_assert!(self.head.is_some());

        let node_ptr = node_addr as *mut NodeLink;
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
        }
    }

    pub fn insert(&mut self, free_addr: usize) {
        let new_node_ptr = free_addr as *mut NodeLink;

        let new_node_ref = unsafe { &mut *new_node_ptr };
        new_node_ref.next = None;
        new_node_ref.prev = None;

        let new_node = Some(unsafe { NonNull::new_unchecked(new_node_ptr) });

        if let Some(mut old_head) = self.head {
            unsafe {
                old_head.as_mut().prev = new_node;
                new_node_ref.next = Some(NonNull::new_unchecked(old_head.as_ptr()));
            }
        }

        self.head = new_node;
    }
}
