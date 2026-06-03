pub mod inplace_vec {
    use std::{mem::MaybeUninit, ops::{Deref, DerefMut, Index, IndexMut}};

  pub struct InplaceVec<T,const SIZE : usize> {
    data : [MaybeUninit<T>;SIZE],
    len : usize
  }
  
  impl<T,const SIZE : usize> InplaceVec<T,SIZE> {
    pub fn new() -> Self {
      InplaceVec{data : [const { MaybeUninit::uninit() };SIZE],len:0}
    }
    pub fn push_back_mut(&mut self, val : T) -> &mut T {
      let ret = self.data[self.len].write(val);
      self.len+=1;
      ret
    }
    pub fn push_back(&mut self, val : T) {
      self.push_back_mut(val);
    }
    pub fn capacity(&self) -> usize {
      SIZE
    }
    pub fn len(&self) -> usize {
      self.len
    }
    pub fn full(&self) -> bool {
      self.len() == self.capacity()
    }
    pub fn clear(&mut self){
      let len = self.len();
      self.len = 0;
      for v in self.data.split_at_mut(len).0 {
        //SAFTEY: len guards what is initialized.
        unsafe {
          v.assume_init_drop();
        }
      }
    }
  }
  impl<T : Clone, const SIZE :usize> InplaceVec<T,SIZE> {
    pub fn fill_up_to(&mut self, val : T, len : usize) {
      if len > self.capacity() {
        panic!("Cannot fill past capacity")
      }
      while self.len() < len {
        self.push_back(val.clone());
      }
    }
    pub fn fill_rest(&mut self, val : T ) {
      self.fill_up_to(val,self.capacity());
    }
  }
  impl<T,const SIZE : usize> Drop for InplaceVec<T,SIZE> {
    fn drop(&mut self) {
      self.clear();
    }
  }

  impl<T,const SIZE : usize> Index<usize> for InplaceVec<T,SIZE> {
    type Output = T;
  
    fn index(&self, index: usize) -> &Self::Output {
      if index>=self.len {
        panic!("index out of bounds");
      }
      //SAFETY: We panic on previous line if it is not initialized.
      unsafe {
        self.data[index].assume_init_ref()
      }
    }
  }

  impl<T,const SIZE : usize> IndexMut<usize> for InplaceVec<T,SIZE> {
  
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
      if index>=self.len {
        panic!("index out of bounds");
      }
      //SAFETY: We panic on previous line if it is not initialized.
      unsafe { 
        self.data[index].assume_init_mut()
      }
    }
  }
  impl<T,const SIZE : usize> Deref for InplaceVec<T,SIZE> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
      //SAFETY: len guards what is initialized.
      unsafe {
        self.data.split_at(self.len()).0.assume_init_ref()
      }
    }
  }
  impl<T,const SIZE : usize> DerefMut for InplaceVec<T,SIZE> {

    fn deref_mut(&mut self) -> &mut Self::Target {
      //SAFETY: len guards what is initialized.
      let len = self.len();
      unsafe {
        self.data.split_at_mut(len).0.assume_init_mut()
      }
    }
  }
   #[cfg(test)]
    mod tests {

use super::*;
        #[test]
        fn push_back_test(){
          let mut v = InplaceVec::<_,10>::new();
          v.push_back(23);
          v.push_back(92);
          assert_eq!(v[0],23);
          assert_eq!(v[1],92);
        }
        #[test]
        fn fill_rest_test(){
          let mut v = InplaceVec::<_,10>::new();
          v.push_back(23);
          v.push_back(92);
          v.fill_rest(-8);
          assert_eq!(v[0],23);
          assert_eq!(v[1],92);
          assert!(v.iter().skip(2).all(|n|*n==-8));
        }
      }
}