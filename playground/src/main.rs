struct MyIterator<'a, T> {
    slice: &'a [T],
}

impl<'a, T> Iterator for MyIterator<'a, T> {
    type Item = &'a T; // a reference to one of the slice's elements

    fn next(&mut self) -> Option<Self::Item> {
        let element = self.slice.get(0);
        if element.is_some() {
            self.slice = &self.slice[1..];
        }
        
        element
    }
}

//struct MyMutableIterator<'a, T> {
//    slice: &'a mut [T],
//}
//
//impl<'a, T> Iterator for MyMutableIterator<'a, T> {
//    type Item = &'a mut T; // a reference to one of the slice's elements
//
//    fn next(&mut self) -> Option<Self::Item> {
//        todo!();
//    }
//}

//struct MyMutableIterator<'iter, T> {
//    slice: &'iter mut [T],
//}
//
//impl<'iter, T> Iterator for MyMutableIterator<'iter, T> {
//    type Item = &'iter mut T; // a reference to one of the slice's elements
//
//    fn next<'next>(&'next mut self) -> Option<Self::Item> {
//        let element = self.slice.get_mut(0);
//        //self.slice = &mut self.slice[1..];
//
//        element
//    }
//}

fn main() {
    println!("hello");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let collection = vec![1, 2, 3, 4];
        let wrapper = MyIterator {
            slice: &collection[..],
        };

        for (index, elem) in wrapper.enumerate() {
            assert_eq!(*elem, collection[index]);
        }

//        let mut collection = vec![1, 2, 3, 4];
//        let wrapper = MyMutableIterator {
//            slice: &mut collection[..],
//        };
//
//        for (index, elem) in wrapper.enumerate() {
//            *elem = *elem + 1;
//        }
//
//        assert_eq!(collection.get(0), Some(&2));
    }
}
